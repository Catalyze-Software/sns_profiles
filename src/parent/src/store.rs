use std::{cell::RefCell, collections::HashMap};

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{
    api::{call, time},
    id,
};

use ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        canister_type::CanisterType,
        filter_type::FilterType,
        sort_type::SortDirection,
        wasm_version_type::WasmVersion,
    },
    helpers::{
        canister_helper::{Canister, CanisterID, CanisterSettings, InstallCodeMode},
        error_helper::api_error,
        logger_helper::add_log,
        paging_helper::get_paged_data,
        serialize_helper::deserialize,
    },
    models::{
        canister_models::ScalableCanisterDetails,
        logger_models::{LogType, PostLog},
        paged_response_models::PagedResponse,
        wasm_models::WasmDetails,
    },
};

use shared::profile_models::{ProfileFilter, ProfileResponse, ProfileSort};

#[derive(CandidType, Clone, Deserialize)]
pub struct ScalableMetaData {
    pub name: String,
    pub canister_count: usize,
    pub has_child_wasm: bool,
    pub cycles: u64,
    pub used_data: u64,
    pub parent: Principal,
    pub updated_at: u64,
    pub created_at: u64,
}

#[derive(CandidType, Clone, Deserialize)]
pub struct ScalableData {
    // The name of the scalable canister (ex; users)
    pub name: String,
    // The child canisters that are used for storing the scalable data
    pub canisters: HashMap<Principal, ScalableCanisterDetails>,
    // The parent canister
    pub parent: Principal,
    // The wasm details that need to be installed on the child canisters
    pub child_wasm_data: WasmDetails,
    // updated_at record
    pub updated_at: u64,
    // created_at record
    pub created_at: u64,
}

impl Default for ScalableData {
    fn default() -> Self {
        ScalableData {
            canisters: HashMap::new(),
            name: String::default(),
            child_wasm_data: Default::default(),
            parent: Principal::anonymous(),
            updated_at: time(),
            created_at: time(),
        }
    }
}

thread_local! {
    pub static DATA: RefCell<ScalableData> = RefCell::new(ScalableData::default());
}
impl ScalableData {
    // Method to retrieve an available canister to write updates to
    pub fn get_available_canister(caller: Principal) -> Result<ScalableCanisterDetails, String> {
        let canister = DATA.with(|v| {
            v.borrow()
                .canisters
                .iter()
                // filter out self in case this method is called by a child canister
                .filter(|(_, c)| c.principal != caller)
                .find(|(_, c)| c.is_available)
                .map(|(_, details)| details.clone())
        });

        match canister {
            None => Err("No available canister found".to_string()),
            Some(c) => Ok(c),
        }
    }

    // Methods to retrieve all the canisters
    pub fn get_canisters() -> Vec<ScalableCanisterDetails> {
        let canisters: Vec<ScalableCanisterDetails> = DATA.with(|v| {
            v.borrow()
                .canisters
                .iter()
                .map(|(_, details)| details.clone())
                .collect()
        });
        return canisters;
    }

    // Method used on the init function to spawn a child canister when the parent canister is installed
    pub async fn initialize_first_child_canister() -> () {
        // check if the child wasm is present
        if DATA.with(|data| data.borrow().child_wasm_data.bytes.len()) == 0 {
            return;
        }

        // check if there is already a child canister
        if DATA.with(|v| v.borrow().canisters.len() != 0) {
            return;
        }

        // spawn empty canister
        let new_canister = Self::spawn_empty_canister().await;
        let _ = match new_canister {
            Err(err) => Err(err),
            Ok(new_canister_principal) => {
                // Install child canister
                let installed_canister = Self::_install_child_canister(
                    Self::get_name(),
                    new_canister_principal,
                    InstallCodeMode::Install,
                )
                .await;
                match installed_canister {
                    Err(err) => Err(err),
                    Ok(new_installed_canister_principal) => Ok(new_installed_canister_principal),
                }
            }
        };
    }

    // Method used called by child canister once full (inter-canister call)
    pub async fn close_child_canister_and_spawn_sibling(
        caller: Principal,
        last_entry_id: u64,
        entry: Vec<u8>,
    ) -> Result<Principal, ApiError> {
        let inputs = Some(vec![format!("last_entry_id - {:?}", &last_entry_id)]);

        // check if the child wasm is present
        if DATA.with(|v| v.borrow().child_wasm_data.bytes.len() == 0) {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "NO_WASM_SPECIFIED",
                "There is no foundation WASM uploaded",
                &Self::get_name(),
                "close_child_canister_and_spawn_sibling",
                inputs,
            ));
        }

        // check if the caller is known to this canister
        let caller_canister = DATA.with(|v| v.borrow().canisters.get(&caller).cloned());
        match caller_canister {
            None => Err(api_error(
                ApiErrorType::BadRequest,
                "UNKNOWN_CANISTER",
                "The caller principal isnt known to this canister",
                &Self::get_name(),
                "close_child_canister_and_spawn_sibling",
                inputs,
            )),
            Some(mut _caller_canister) => {
                // spawn empty canister
                let new_canister = Self::spawn_empty_canister().await;
                match new_canister {
                    Err(err) => Err(err),
                    Ok(new_canister_principal) => {
                        // Install child canister
                        let installed_canister = Self::_install_child_canister(
                            Self::get_name(),
                            new_canister_principal,
                            InstallCodeMode::Install,
                        )
                        .await;
                        match installed_canister {
                            Err(err) => Err(err),
                            Ok(new_installed_canister_principal) => {
                                // update the caller canister
                                _caller_canister.is_available = false;
                                _caller_canister.entry_range = (0, Some(last_entry_id));

                                DATA.with(|v| {
                                    v.borrow_mut()
                                        .canisters
                                        .insert(_caller_canister.principal, _caller_canister)
                                });

                                // send the entry to the new canister
                                let call_result: Result<(Result<(), ApiError>,), _> = call::call(
                                    new_installed_canister_principal,
                                    "add_entry_by_parent",
                                    (entry,),
                                )
                                .await;

                                match call_result {
                                    Err(err) => Err(api_error(
                                        ApiErrorType::BadRequest,
                                        "FAILED_TO_STORE_DATA",
                                        err.1.as_str(),
                                        &Self::get_name(),
                                        "close_child_canister_and_spawn_sibling",
                                        inputs,
                                    )),
                                    Ok(_) => Ok(new_installed_canister_principal),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Method used to upgrade the child canister
    pub async fn upgrade_child_canister(
        canister_principal: Principal,
    ) -> Result<ScalableCanisterDetails, ApiError> {
        let inputs = Some(vec![format!(
            "canister_principal - {}",
            &canister_principal.to_string()
        )]);

        let data = DATA.with(|v| v.borrow().clone());
        match data.canisters.get(&canister_principal).cloned() {
            None => Err(api_error(
                ApiErrorType::NotFound,
                "NO_CHILDREN",
                "There are no child canisters found",
                &Self::get_name(),
                "upgrade_scalable_canister",
                inputs,
            )),
            Some(mut _child_canister) => {
                // check if the version of the wasm is different then the new version
                if &data.child_wasm_data.wasm_version == &_child_canister.wasm_version {
                    return Err(api_error(
                        ApiErrorType::BadRequest,
                        "CANISTER_UP_TO_DATE",
                        "The latest WASM version is already installed",
                        &Self::get_name(),
                        "upgrade_scalable_canister",
                        inputs,
                    ));
                }

                let canister = Canister::from(_child_canister.principal);
                // upgrade the child canister
                let upgrade_result = canister
                    .install_code(
                        InstallCodeMode::Upgrade,
                        data.child_wasm_data.bytes.clone(),
                        (),
                    )
                    .await;
                match upgrade_result {
                    Err(err) => Err(api_error(
                        ApiErrorType::BadRequest,
                        "UPGRADE_FAILED",
                        &err.1.as_str(),
                        &Self::get_name(),
                        "upgrade_scalable_canister",
                        inputs,
                    )),
                    Ok(_) => {
                        // update child wasm version
                        _child_canister.wasm_version = data.child_wasm_data.wasm_version;

                        DATA.with(|v| {
                            v.borrow_mut()
                                .canisters
                                .insert(canister_principal, _child_canister.clone())
                        });
                        Ok(_child_canister)
                    }
                }
            }
        }
    }

    // Method used to spawn an empty canister (not installed)
    async fn spawn_empty_canister() -> Result<Principal, ApiError> {
        // Set canister settings
        let canister_settings = CanisterSettings {
            controllers: Some(vec![id()]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
        };

        // Create canister with predefined amount of cycles
        let new_canister = Canister::create(Some(canister_settings), 2_000_000_000_000).await;
        match new_canister {
            Err(err) => Err(api_error(
                ApiErrorType::BadRequest,
                "CANISTER_NOT_CREATED",
                err.1.as_str(),
                &Self::get_name(),
                "_spawn_empty_canister",
                None,
            )),
            Ok(_canister) => {
                let new_canister_principal = CanisterID::from(_canister);
                let canister_data = ScalableCanisterDetails {
                    principal: new_canister_principal,
                    wasm_version: WasmVersion::None,
                    canister_type: CanisterType::Empty,
                    is_available: true,
                    entry_range: (0, None),
                };

                // Store child canister data on the parent
                DATA.with(|v| {
                    v.borrow_mut()
                        .canisters
                        .insert(new_canister_principal, canister_data)
                });
                Ok(new_canister_principal)
            }
        }
    }

    // Install the child canister
    async fn _install_child_canister(
        name: String,
        canister_principal: Principal,
        install_code_mode: InstallCodeMode,
    ) -> Result<Principal, ApiError> {
        let inputs = Some(vec![format!("name - {}", &name.to_string())]);

        let data = DATA.with(|v| v.borrow().clone());
        if data.child_wasm_data.bytes.len() == 0 {
            return Err(api_error(
                ApiErrorType::BadRequest,
                "NO_WASM_SPECIFIED",
                "There is no foundation WASM uploaded",
                &Self::get_name(),
                "install_child_canister",
                inputs,
            ));
        }

        let install_canister = Canister::from(canister_principal)
            .install_code(
                install_code_mode,
                data.child_wasm_data.bytes,
                (id(), name, data.canisters.iter().len()),
            )
            .await;

        match install_canister {
            Err(err) => Err(api_error(
                ApiErrorType::NotFound,
                "CANISTER_INSTALL_FAILED",
                err.1.as_str(),
                &Self::get_name(),
                "_install_child_canister",
                inputs,
            )),
            Ok(_) => {
                let new_child_details = ScalableCanisterDetails {
                    principal: canister_principal,
                    wasm_version: data.child_wasm_data.wasm_version.clone(),
                    is_available: true,
                    canister_type: CanisterType::ScalableChild,
                    entry_range: (0, None),
                };

                DATA.with(|v| {
                    v.borrow_mut()
                        .canisters
                        .insert(canister_principal, new_child_details)
                });
                Ok(canister_principal)
            }
        }
    }

    // Method used to upgrade all the child canister
    pub async fn upgrade_children() {
        let data = DATA.with(|data| data.borrow().clone());
        for child in data.canisters {
            if child.1.wasm_version != data.child_wasm_data.wasm_version {
                match ScalableData::upgrade_child_canister(child.0.clone()).await {
                    Ok(_details) => add_log(PostLog {
                        log_type: LogType::Info,
                        description: "Profile child canister successfully upgraded".to_string(),
                        source: "upgrade_children".to_string(),
                        data: format!("{:?}", _details),
                    }),
                    Err(err) => add_log(PostLog {
                        log_type: LogType::Error,
                        description: "Profile child canister not upgraded".to_string(),
                        source: "upgrade_children".to_string(),
                        data: format!("{:?}", err),
                    }),
                };
            }
        }
    }

    // Get the child WASM
    pub fn get_child_wasm_data(
        old_store: &ScalableData,
        version: u64,
    ) -> Result<WasmDetails, String> {
        // Get the WASM from the file system
        let bytes = include_bytes!("../../../wasm/child.wasm.gz").to_vec();

        // Check if the wasm bytes are empty
        if bytes.is_empty() {
            return Err("No WASM found, skipping child WASM update".to_string());
        }

        // Check if the WASM is the same as the previous one
        if old_store.child_wasm_data.bytes == bytes {
            return Err("WASM is the same, skipping child WASM update".to_string());
        }

        // Create the WASM details
        let details = WasmDetails {
            label: "child_profile_canister".to_string(),
            bytes,
            wasm_type: CanisterType::ScalableChild,
            wasm_version: WasmVersion::Version(version),
            updated_at: time(),
            created_at: old_store.child_wasm_data.created_at,
        };

        Ok(details)
    }

    pub async fn get_child_canister_data(
        limit: usize,
        page: usize,
        filters: Vec<ProfileFilter>,
        filter_type: FilterType,
        sort: ProfileSort,
    ) -> PagedResponse<ProfileResponse> {
        let canisters: Vec<Principal> = DATA.with(|data| {
            data.borrow()
                .canisters
                .clone()
                .into_iter()
                .map(|c| c.1.principal.clone())
                .collect()
        });

        let mut profiles: Vec<ProfileResponse> = vec![];
        for canister in canisters {
            let mut canister_data =
                Self::get_filtered_child_data(canister, &filters, &filter_type).await;
            profiles.append(&mut canister_data);
        }

        let ordered_profiles = Self::_get_ordered_profiles(profiles, sort);
        get_paged_data(ordered_profiles, limit, page)
    }

    async fn get_filtered_child_data(
        canister_principal: Principal,
        filters: &Vec<ProfileFilter>,
        filter_type: &FilterType,
    ) -> Vec<ProfileResponse> {
        let (mut bytes, (_, last)) =
            Self::get_chunked_child_data(canister_principal, filters, filter_type, 0, None).await;

        if last > 1 {
            for i in 1..last + 1 {
                let (mut _bytes, _) =
                    Self::get_chunked_child_data(canister_principal, filters, filter_type, i, None)
                        .await;
                bytes.append(&mut _bytes);
            }
        }

        match deserialize::<Vec<ProfileResponse>>(bytes.clone()) {
            Ok(_res) => _res,
            Err(_err) => {
                ic_cdk::println!("Error: {}", _err);
                vec![]
            }
        }
    }

    fn _get_ordered_profiles(
        mut profiles: Vec<ProfileResponse>,
        sort: ProfileSort,
    ) -> Vec<ProfileResponse> {
        use ProfileSort::*;
        use SortDirection::*;
        match sort {
            Username(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.username.cmp(&b.username)),
                Desc => profiles.sort_by(|a, b| b.username.cmp(&a.username)),
            },
            DisplayName(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.display_name.cmp(&b.display_name)),
                Desc => profiles.sort_by(|a, b| b.display_name.cmp(&a.display_name)),
            },
            FirstName(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.first_name.cmp(&b.first_name)),
                Desc => profiles.sort_by(|a, b| b.first_name.cmp(&a.first_name)),
            },
            LastName(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.last_name.cmp(&b.last_name)),
                Desc => profiles.sort_by(|a, b| b.last_name.cmp(&a.last_name)),
            },
            Email(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.email.cmp(&b.email)),
                Desc => profiles.sort_by(|a, b| b.email.cmp(&a.email)),
            },
            City(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.city.cmp(&b.city)),
                Desc => profiles.sort_by(|a, b| b.city.cmp(&a.city)),
            },
            Country(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.country.cmp(&b.country)),
                Desc => profiles.sort_by(|a, b| b.country.cmp(&a.country)),
            },
            CreatedOn(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.created_on.cmp(&b.created_on)),
                Desc => profiles.sort_by(|a, b| b.created_on.cmp(&a.created_on)),
            },
            UpdatedOn(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.updated_on.cmp(&b.updated_on)),
                Desc => profiles.sort_by(|a, b| b.updated_on.cmp(&a.updated_on)),
            },
            StateOrProvince(direction) => match direction {
                Asc => profiles.sort_by(|a, b| a.state_or_province.cmp(&b.state_or_province)),
                Desc => profiles.sort_by(|a, b| b.state_or_province.cmp(&a.state_or_province)),
            },
        };
        profiles
    }

    async fn get_chunked_child_data(
        canister_principal: Principal,
        filters: &Vec<ProfileFilter>,
        filter_type: &FilterType,
        chunk: usize,
        max_bytes_per_chunk: Option<usize>,
    ) -> (Vec<u8>, (usize, usize)) {
        let _max_bytes_per_chunk = max_bytes_per_chunk.unwrap_or(2_000_000);
        let result: Result<(Vec<u8>, (usize, usize)), _> = call::call(
            canister_principal,
            "get_chunked_data",
            (filters, filter_type, chunk, _max_bytes_per_chunk),
        )
        .await;

        match result {
            Ok(_res) => _res,
            _ => (vec![], (0, 0)),
        }
    }

    fn get_name() -> String {
        DATA.with(|v| v.borrow().name.clone())
    }
}
