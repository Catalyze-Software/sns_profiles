use std::collections::HashMap;

use candid::Principal;
use ic_catalyze_notifications::models::{Environment, FriendRequestNotificationData};
use ic_catalyze_notifications::store::Notification;
use ic_cdk::api::{call, time};
use ic_cdk::id;
use ic_scalable_canister::store::Data;

use ic_scalable_canister::ic_scalable_misc::helpers::serialize_helper::serialize;
use ic_scalable_canister::ic_scalable_misc::models::identifier_model::Identifier;
use ic_scalable_canister::ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        application_role_type::ApplicationRole,
        asset_type::Asset,
        sort_type::SortDirection,
    },
    helpers::{error_helper::api_error, paging_helper::get_paged_data},
    models::paged_response_models::PagedResponse,
};

use serde_json::json;
use shared::profile_models::{
    DocumentDetails, FriendRequest, FriendRequestResponse, PostProfile, PostWallet, Profile,
    ProfileFilter, ProfileResponse, ProfileSort, RelationType, UpdateProfile, Wallet,
    WalletResponse,
};

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    {DefaultMemoryImpl, StableBTreeMap, StableCell},
};

use std::cell::RefCell;

use crate::IDENTIFIER_KIND;

use super::validation::{validate_post_profile, validate_update_profile};

type Memory = VirtualMemory<DefaultMemoryImpl>;

pub static DATA_MEMORY_ID: MemoryId = MemoryId::new(0);
pub static ENTRIES_MEMORY_ID: MemoryId = MemoryId::new(1);
pub static FRIEND_REQUESTS_MEMORY_ID: MemoryId = MemoryId::new(2);

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

        pub static STABLE_DATA: RefCell<StableCell<Data, Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(DATA_MEMORY_ID)),
                Data::default(),
            ).expect("failed")
        );

        pub static ENTRIES: RefCell<StableBTreeMap<String, Profile, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(ENTRIES_MEMORY_ID)),
            )
        );

        pub static FRIEND_REQUEST: RefCell<StableBTreeMap<u64, FriendRequest, Memory>> = RefCell::new(
            StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(FRIEND_REQUESTS_MEMORY_ID)),
            )
        );
}

pub struct Store;

impl Store {
    // Method to add a profile to the data store
    pub async fn add_profile(
        caller: Principal,
        post_profile: PostProfile,
        member_canister: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("post_profile - {:?}", &post_profile),
        ]);

        // Check if the user has already registered a profile
        match Self::_get_profile_from_caller(caller) {
            // If the user has already registered a profile, return an error
            Some(_) => Err(api_error(
                ApiErrorType::BadRequest,
                "ALREADY_REGISTERED",
                "User profile already registered",
                STABLE_DATA
                    .with(|data| Data::get_name(data.borrow().get()))
                    .as_str(),
                "add_profile",
                inputs,
            )),
            // If the user has not registered a profile, continue and validate the post_profile method argument
            None => match validate_post_profile(post_profile.clone()) {
                Err(err) => Err(err),
                Ok(_) => {
                    // Check if the username is already taken
                    if Self::_has_user_name(
                        &ENTRIES.with(|entries| Data::get_entries(entries)),
                        &post_profile.username,
                    ) {
                        return Err(api_error(
                            ApiErrorType::BadRequest,
                            "USERNAME_TAKEN",
                            "Username already taken",
                            STABLE_DATA
                                .with(|data| Data::get_name(data.borrow().get()))
                                .as_str(),
                            "add_profile",
                            inputs,
                        ));
                    }

                    let empty = "".to_string();

                    // Create a new profile object and set the post profile values
                    let profile = Profile {
                        principal: caller,
                        username: post_profile.username,
                        display_name: post_profile.display_name,
                        application_role: ApplicationRole::default(),
                        first_name: post_profile.first_name,
                        last_name: post_profile.last_name,
                        privacy: post_profile.privacy,
                        about: empty.clone(),
                        email: empty.clone(),
                        date_of_birth: 0,
                        city: empty.clone(),
                        state_or_province: empty.clone(),
                        country: empty.clone(),
                        profile_image: Asset::None,
                        banner_image: Asset::None,
                        skills: vec![],
                        interests: vec![],
                        causes: vec![],
                        website: empty,
                        wallets: HashMap::new(),
                        starred: HashMap::new(),
                        relations: HashMap::new(),
                        code_of_conduct: DocumentDetails {
                            approved_version: 0,
                            approved_date: 0,
                        },
                        extra: post_profile.extra,
                        updated_on: time(),
                        created_on: time(),
                        member_identifier: Principal::anonymous(),
                        privacy_policy: None,
                        terms_of_service: None,
                    };
                    // Add the new profile to the data store and pass in the "kind" as a third parameter to generate a identifier
                    let add_entry_result = STABLE_DATA.with(|data| {
                        ENTRIES.with(|entries| {
                            Data::add_entry(
                                data,
                                entries,
                                profile.clone(),
                                Some(IDENTIFIER_KIND.to_string()),
                            )
                        })
                    });

                    // Check if the profile was added to the data store successfully
                    match add_entry_result {
                        // The profile was not added to the data store because the canister is at capacity
                        Err(err) => match err {
                            ApiError::CanisterAtCapacity(message) => {
                                let _data = STABLE_DATA.with(|v| v.borrow().get().clone());
                                // Spawn a sibling canister and pass the profile data to it
                                match Data::spawn_sibling(&_data, profile).await {
                                    Ok(_) => Err(ApiError::CanisterAtCapacity(message)),
                                    Err(err) => Err(err),
                                }
                            }
                            _ => Err(err),
                        },
                        Ok((identifier, mut profile)) => {
                            // Create a new member entry on the specified member canister
                            let member_result: Result<(Result<Principal, ApiError>,), _> =
                                call::call(
                                    member_canister,
                                    "create_empty_member",
                                    (caller, identifier),
                                )
                                .await;
                            match member_result {
                                Ok(_result) => match _result.0 {
                                    Ok(_member_identifier) => {
                                        // Update the profile with the member identifier
                                        STABLE_DATA.with(|data| {
                                            profile.member_identifier = _member_identifier;
                                            let _ = ENTRIES.with(|entries| {
                                                Data::update_entry(
                                                    data,
                                                    entries,
                                                    identifier,
                                                    profile.clone(),
                                                )
                                            });
                                        });
                                        Ok(Self::_map_profile_to_profile_response(
                                            identifier.to_string(),
                                            profile,
                                        ))
                                    }
                                    Err(err) => Err(err),
                                },
                                Err(err) => Err(api_error(
                                    ApiErrorType::Unexpected,
                                    "ICC_MEMBER_CREATION_FAILED",
                                    err.1.as_str(),
                                    STABLE_DATA
                                        .with(|data| Data::get_name(data.borrow().get()))
                                        .as_str(),
                                    "add_profile",
                                    inputs,
                                )),
                            }
                        }
                    }
                }
            },
        }
    }

    // Method to update a profile in the data store
    pub fn update_profile(
        caller: Principal,
        update_profile: UpdateProfile,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("update_profile - {:?}", &update_profile),
        ]);
        // get the profile from the data store
        match Self::_get_profile_from_caller(caller) {
            // If the profile does not exist, return an error
            None => Err(Self::_profile_not_found_error("update_profile", inputs)),
            // If the profile exists, continue and validate the update_profile method argument
            Some((_identifier, mut profile)) => {
                match validate_update_profile(update_profile.clone()) {
                    Err(err) => Err(err),
                    Ok(_) => {
                        // Check if the email is not empty
                        let email = match update_profile.email {
                            None => "".to_string(),
                            Some(_email) => _email,
                        };
                        // Check if it is not the same as the current email and the email is already taken
                        if email != "" {
                            if profile.email != email
                                && ENTRIES.with(|entries| {
                                    Self::_has_email(&Data::get_entries(entries), &email)
                                })
                            {
                                return Err(api_error(
                                    ApiErrorType::BadRequest,
                                    "EMAIL_TAKEN",
                                    "Email already taken",
                                    STABLE_DATA
                                        .with(|data| Data::get_name(data.borrow().get()))
                                        .as_str(),
                                    "update_profile",
                                    inputs,
                                ));
                            }
                        }

                        // update profile fields
                        profile.display_name = update_profile.display_name;
                        profile.first_name = update_profile.first_name;
                        profile.last_name = update_profile.last_name;
                        profile.privacy = update_profile.privacy;
                        profile.about = update_profile.about;
                        profile.email = email;
                        profile.date_of_birth = update_profile.date_of_birth;
                        profile.city = update_profile.city;
                        profile.state_or_province = update_profile.state_or_province;
                        profile.country = update_profile.country;
                        profile.profile_image = update_profile.profile_image;
                        profile.banner_image = update_profile.banner_image;
                        profile.skills = update_profile.skills;
                        profile.interests = update_profile.interests;
                        profile.causes = update_profile.causes;
                        profile.website = update_profile.website;
                        profile.extra = update_profile.extra;
                        profile.updated_on = time();

                        // update the profile in the data store
                        match STABLE_DATA.with(|data| {
                            ENTRIES.with(|entries| {
                                Data::update_entry(data, entries, _identifier, profile)
                            })
                        }) {
                            Err(err) => Err(err),
                            Ok((identifier, profile)) => {
                                Ok(Self::_map_profile_to_profile_response(
                                    identifier.to_string(),
                                    profile,
                                ))
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn add_wallet(caller: Principal, wallet: PostWallet) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("wallet - {:?}", &wallet),
        ]);

        // get the profile from the data store
        match Self::_get_profile_from_caller(caller) {
            // If the profile does not exist, return an error
            None => Err(Self::_profile_not_found_error("add_wallet", inputs)),
            // If the profile exists, continue
            Some((_identifier, mut _profile)) => {
                // Add the wallet to the profile, insert overwrites if the wallet already exists
                _profile.wallets.insert(
                    wallet.principal,
                    Wallet {
                        is_primary: false,
                        provider: wallet.provider,
                    },
                );
                // Update the profile in the data store
                STABLE_DATA
                    .with(|data| {
                        ENTRIES.with(|entries| {
                            Data::update_entry(data, entries, _identifier, _profile)
                        })
                    })
                    .map_or_else(
                        |err| Err(err),
                        |result| {
                            Ok(Self::_map_profile_to_profile_response(
                                result.0.to_string(),
                                result.1,
                            ))
                        },
                    )
            }
        }
    }

    // Method to set a wallet as primary
    pub fn set_wallet_as_primary(caller: Principal, wallet_principal: Principal) -> Result<(), ()> {
        // get the profile from the data store
        if let Some((_identifier, mut _profile)) = Store::_get_profile_from_caller(caller) {
            // Check if the wallet exists
            if _profile.wallets.get(&wallet_principal).is_none() {
                return Err(());
            }

            // Set all wallets to not primary
            for (_wallet_principal, mut _wallet) in _profile.wallets.iter_mut() {
                _wallet.is_primary = false;
            }
            // Set the wallet as primary
            _profile
                .wallets
                .get_mut(&wallet_principal)
                .unwrap()
                .is_primary = true;

            // Update the profile in the data store
            if let Ok(_) = STABLE_DATA.with(|data| {
                ENTRIES.with(|entries| Data::update_entry(data, entries, _identifier, _profile))
            }) {
                return Ok(());
            } else {
                return Err(());
            }
        } else {
            return Err(());
        }
    }

    // Method to remove a wallet from a profile
    pub fn remove_wallet(
        caller: Principal,
        wallet_principal: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("wallet - {:?}", &wallet_principal),
        ]);

        // get the profile from the data store
        match Self::_get_profile_from_caller(caller) {
            // If the profile does not exist, return an error
            None => Err(Self::_profile_not_found_error("remove_wallet", inputs)),
            // If the profile exists, continue
            Some((_identifier, mut _profile)) => {
                // Check if the wallet exists
                if let None = _profile.wallets.get(&wallet_principal) {
                    return Err(api_error(
                        ApiErrorType::NotFound,
                        "WALLET_NOT_FOUND",
                        "Wallet not found",
                        STABLE_DATA
                            .with(|data| Data::get_name(data.borrow().get()))
                            .as_str(),
                        "remove_wallet",
                        inputs,
                    ));
                }
                // Remove the wallet from the profile
                _profile.wallets.remove(&wallet_principal);

                // Update the profile in the data store
                STABLE_DATA
                    .with(|data| {
                        ENTRIES.with(|entries| {
                            Data::update_entry(data, entries, _identifier, _profile)
                        })
                    })
                    .map_or_else(
                        |err| Err(err),
                        |result| {
                            Ok(Self::_map_profile_to_profile_response(
                                result.0.to_string(),
                                result.1,
                            ))
                        },
                    )
            }
        }
    }

    // Method to add a starred identifier to a profile
    pub fn add_starred(
        caller: Principal,
        starred_identifier: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller.to_string()),
            format!("identifier - {:?}", &starred_identifier.to_string()),
        ]);
        // decode the identifier
        let (_, _, kind) = Identifier::decode(&starred_identifier);
        // check if the identifier is valid to use as a starred identifier
        if !vec!["grp".to_string(), "tsk".to_string(), "evt".to_string()].contains(&kind) {
            return Err(api_error(
                ApiErrorType::NotFound,
                "INVALID TYPE",
                format!("'{}' is not supported", kind).as_str(),
                STABLE_DATA
                    .with(|data| Data::get_name(data.borrow().get()))
                    .as_str(),
                "add_starred",
                inputs,
            ));
        }

        // get the profile from the data store
        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error("add_starred", inputs)),
            // If the profile exists, continue
            Some((_identifier, mut _profile)) => {
                // Add the starred identifier to the profile
                _profile.starred.insert(starred_identifier, kind);

                STABLE_DATA
                    .with(|data| {
                        ENTRIES.with(|entries| {
                            Data::update_entry(data, entries, _identifier, _profile)
                        })
                    })
                    .map_or_else(
                        |err| Err(err),
                        |result| {
                            Ok(Self::_map_profile_to_profile_response(
                                result.0.to_string(),
                                result.1,
                            ))
                        },
                    )
            }
        }
    }

    // Method to remove a starred identifier from a profile
    pub fn remove_starred(
        caller: Principal,
        starred_identifier: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("identifier - {:?}", &starred_identifier),
        ]);

        // get the profile from the data store
        match Self::_get_profile_from_caller(caller) {
            // If the profile does not exist, return an error
            None => Err(Self::_profile_not_found_error("remove_starred", inputs)),
            // If the profile exists, continue
            Some((_identifier, mut _profile)) => {
                // Check if the starred identifier exists
                if let None = _profile.starred.get(&starred_identifier) {
                    return Err(api_error(
                        ApiErrorType::NotFound,
                        "STARRED_NOT_FOUND",
                        "Starred identifier not found",
                        STABLE_DATA
                            .with(|data| Data::get_name(data.borrow().get()))
                            .as_str(),
                        "remove_starred",
                        inputs,
                    ));
                }

                // Remove the starred identifier from the profile
                _profile.starred.remove(&starred_identifier);
                // Update the profile in the data store
                STABLE_DATA
                    .with(|data| {
                        ENTRIES.with(|entries| {
                            Data::update_entry(data, entries, _identifier, _profile)
                        })
                    })
                    .map_or_else(
                        |err| Err(err),
                        |result| {
                            Ok(Self::_map_profile_to_profile_response(
                                result.0.to_string(),
                                result.1,
                            ))
                        },
                    )
            }
        }
    }

    // Method to get all starred identifiers of a specific type
    pub fn get_starred(caller: Principal, kind: String) -> Vec<Principal> {
        // get the profile from the data store
        let profile = Self::_get_profile_from_caller(caller);
        // If the profile exists, continue
        if let Some((_principal, _profile)) = profile {
            // Create a vector to hold the starred identifiers
            let mut starred = vec![];
            // Iterate through the starred identifiers in the profile
            _profile
                .starred
                .into_iter()
                .for_each(|(_starred_identifier, _kind)| {
                    // If the kind matches the kind passed in, add it to the vector
                    if _kind == kind {
                        starred.push(_starred_identifier)
                    }
                });
            return starred;
        };
        return vec![];
    }

    // Method to add a relation to a profile
    pub fn add_relation(
        caller: Principal,
        relation_type: RelationType,
        relation_identifier: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller.to_string()),
            format!("relation_type - {:?}", &relation_type.to_string()),
            format!(
                "relation_identifier - {:?}",
                &relation_identifier.to_string()
            ),
        ]);

        // decode the identifier
        let (_, _, kind) = Identifier::decode(&relation_identifier);
        // check if the identifier is valid to use as a relation identifier
        if &kind != &IDENTIFIER_KIND.to_string() {
            return Err(api_error(
                ApiErrorType::NotFound,
                "INVALID TYPE",
                format!("'{}' is not supported", kind).as_str(),
                STABLE_DATA
                    .with(|data| Data::get_name(data.borrow().get()))
                    .as_str(),
                "add_relation",
                inputs,
            ));
        }

        // get the profile from the data store
        match Self::_get_profile_from_caller(caller) {
            // If the profile does not exist, return an error
            None => Err(Self::_profile_not_found_error("add_relation", inputs)),
            // If the profile exists, continue
            Some((_identifier, mut _profile)) => {
                // Add the relation to the profile, if existing it will be overwritten
                _profile
                    .relations
                    .insert(relation_identifier, relation_type.to_string());

                // Update the profile in the data store
                STABLE_DATA
                    .with(|data| {
                        ENTRIES.with(|entries| {
                            Data::update_entry(data, entries, _identifier, _profile)
                        })
                    })
                    .map_or_else(
                        |err| Err(err),
                        |result| {
                            Ok(Self::_map_profile_to_profile_response(
                                result.0.to_string(),
                                result.1,
                            ))
                        },
                    )
            }
        }
    }

    // Method to get the relations of a profile by type
    pub fn get_relations(caller: Principal, relation_type: RelationType) -> Vec<Principal> {
        // Create a vector to hold the relations
        let mut relations = vec![];

        // get the profile from the data store
        let profile = Self::_get_profile_from_caller(caller);
        // If the profile exists, continue
        if let Some((_principal, _profile)) = profile {
            // Iterate through the relations in the profile
            _profile
                .relations
                .into_iter()
                .for_each(|(_relation_identifier, _relation_type)| {
                    // If the relation type matches the relation type passed in, add it to the vector
                    if _relation_type == relation_type.to_string() {
                        relations.push(_relation_identifier);
                    }
                });
            return relations;
        };
        return vec![];
    }

    // Method to get the profile of the caller
    pub fn get_profile_by_user_principal(
        principal: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        // get the profile from the data store
        match Self::_get_profile_from_caller(principal) {
            // If the profile does not exist, return an error
            None => Err(Self::_profile_not_found_error(
                "get_profile_by_user_principal",
                None,
            )),
            // If the profile exists, continue
            Some((_identifier, profile)) => Ok(Self::_map_profile_to_profile_response(
                _identifier.to_string(),
                profile,
            )),
        }
    }

    // Method to get the profile by an identifier
    pub fn get_profile_by_identifier(identifier: Principal) -> Result<ProfileResponse, ApiError> {
        // get the profile from the data store
        match STABLE_DATA
            .with(|data| ENTRIES.with(|entries| Data::get_entry(data, entries, identifier)))
        {
            // If the profile does not exist, return an error
            Err(err) => Err(err),
            // If the profile exists, continue
            Ok((_identifier, profile)) => Ok(Self::_map_profile_to_profile_response(
                _identifier.to_string(),
                profile,
            )),
        }
    }

    // Method to get profiles by a list of principals
    pub fn get_profiles_by_user_principal(principals: Vec<Principal>) -> Vec<ProfileResponse> {
        // get the profiles from the data store
        let fetched_profiles = ENTRIES.with(|entries| Data::get_entries(entries));

        // filter the profiles by the principals passed in
        principals
            .into_iter()
            .filter_map(|principal| {
                fetched_profiles
                    .iter()
                    // filter the profiles by the principal
                    .find(|f| f.1.principal == principal)
                    .map(|(_identifier, profile)| {
                        Self::_map_profile_to_profile_response(_identifier.clone(), profile.clone())
                    })
            })
            .collect()
    }

    // Method to get profiles by a list of identifiers
    pub fn get_profiles_by_identifier(profile_identifiers: Vec<Principal>) -> Vec<ProfileResponse> {
        // create a vector to hold the profiles
        let mut profiles: Vec<ProfileResponse> = vec![];

        // filter the profiles by the principals passed in
        for identifier in profile_identifiers {
            // get the profile from the data store
            if let Ok((_identifier, profile)) = STABLE_DATA
                .with(|data| ENTRIES.with(|entries| Data::get_entry(data, entries, identifier)))
            {
                // add the profile to the vector
                profiles.push(Self::_map_profile_to_profile_response(
                    _identifier.to_string(),
                    profile,
                ));
            }
        }

        profiles
    }

    // Method to set the approved code of conduct version for a profile
    pub fn approve_code_of_conduct(caller: Principal, version: u64) -> Result<bool, ApiError> {
        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error(
                "approve_code_of_conduct",
                None,
            )),
            Some((_identifier, mut _existing)) => {
                _existing.code_of_conduct = DocumentDetails {
                    approved_version: version,
                    approved_date: time(),
                };

                let _ = STABLE_DATA.with(|data| {
                    ENTRIES
                        .with(|entries| Data::update_entry(data, entries, _identifier, _existing))
                });
                Ok(true)
            }
        }
    }

    pub fn approve_privacy_policy(caller: Principal, version: u64) -> Result<bool, ApiError> {
        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error(
                "approve_privacy_policy",
                None,
            )),
            Some((_identifier, mut _existing)) => {
                _existing.privacy_policy = Some(DocumentDetails {
                    approved_version: version,
                    approved_date: time(),
                });

                let _ = STABLE_DATA.with(|data| {
                    ENTRIES
                        .with(|entries| Data::update_entry(data, entries, _identifier, _existing))
                });
                Ok(true)
            }
        }
    }

    pub fn approve_terms_of_service(caller: Principal, version: u64) -> Result<bool, ApiError> {
        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error(
                "approve_terms_of_service",
                None,
            )),
            Some((_identifier, mut _existing)) => {
                _existing.terms_of_service = Some(DocumentDetails {
                    approved_version: version,
                    approved_date: time(),
                });

                let _ = STABLE_DATA.with(|data| {
                    ENTRIES
                        .with(|entries| Data::update_entry(data, entries, _identifier, _existing))
                });
                Ok(true)
            }
        }
    }

    pub fn get_paged_profiles_by_identifier(
        identifiers: Vec<Principal>,
        limit: usize,
        page: usize,
        filters: Vec<ProfileFilter>,
        sort: ProfileSort,
    ) -> PagedResponse<ProfileResponse> {
        // create a vector to hold the profiles
        let mut profiles: Vec<ProfileResponse> = vec![];

        STABLE_DATA.with(|data| {
            // filter the profiles by the identifiers passed in
            identifiers.into_iter().for_each(|identifier| {
                if let Ok((_identifier, _profile)) =
                    ENTRIES.with(|entries| Data::get_entry(&data, entries, identifier))
                {
                    // add the profile to the vector
                    profiles.push(Self::_map_profile_to_profile_response(
                        _identifier.to_string(),
                        _profile,
                    ))
                };
            });
            // filter the profiles by the filters passed in
            let filtered_profiles = Self::_get_filtered_profiles(profiles, filters);
            // sort the profiles by the sort passed in
            let ordered_profiles = Self::_get_ordered_profiles(filtered_profiles, sort);
            // return the paged profiles
            get_paged_data(ordered_profiles, limit, page)
        })
    }

    pub fn get_paged_profiles_by_principal(
        principals: Vec<Principal>,
        limit: usize,
        page: usize,
        filters: Vec<ProfileFilter>,
        sort: ProfileSort,
    ) -> PagedResponse<ProfileResponse> {
        // create a vector to hold the profiles
        let mut profiles: Vec<ProfileResponse> = vec![];
        ENTRIES.with(|entries| {
            // get profiles from the data store
            let all_profiles = Data::get_entries(entries);
            // filter the profiles by the principals passed in
            principals.into_iter().for_each(|p| {
                if let Some((_identifier, _profile)) =
                    all_profiles.iter().find(|(_, _p)| _p.principal == p)
                {
                    // add the profile to the vector
                    profiles.push(Self::_map_profile_to_profile_response(
                        _identifier.clone(),
                        _profile.clone(),
                    ));
                };
            });
            // filter the profiles by the filters passed in
            let filtered_profiles = Self::_get_filtered_profiles(profiles, filters);
            // sort the profiles by the sort passed in
            let ordered_profiles = Self::_get_ordered_profiles(filtered_profiles, sort);
            // return the paged profiles
            get_paged_data(ordered_profiles, limit, page)
        })
    }

    fn _has_user_name(profiles: &Vec<(String, Profile)>, username: &String) -> bool {
        let profile = profiles
            .iter()
            .find(|(_, profile)| &profile.username == username);
        match profile {
            None => false,
            Some(_) => true,
        }
    }

    // Method to check if a profile exists by email
    fn _has_email(profiles: &Vec<(String, Profile)>, email: &String) -> bool {
        let profile = profiles.iter().find(|(_, profile)| &profile.email == email);
        match profile {
            None => false,
            Some(_) => true,
        }
    }

    // Method to order profiles by a sort
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

    // Method to filter profiles by a filter
    fn _get_filtered_profiles(
        mut profiles: Vec<ProfileResponse>,
        filters: Vec<ProfileFilter>,
    ) -> Vec<ProfileResponse> {
        for filter in filters {
            use ProfileFilter::*;
            match filter {
                Username(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.username.contains(&value))
                        .cloned()
                        .collect();
                }
                DisplayName(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.display_name.contains(&value))
                        .cloned()
                        .collect();
                }
                FirstName(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.first_name.contains(&value))
                        .cloned()
                        .collect();
                }
                LastName(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.last_name.contains(&value))
                        .cloned()
                        .collect();
                }
                Email(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.email.contains(&value))
                        .cloned()
                        .collect();
                }
                City(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.city.contains(&value))
                        .cloned()
                        .collect();
                }
                Country(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.country.contains(&value))
                        .cloned()
                        .collect();
                }
                Skill(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.skills.contains(&value))
                        .cloned()
                        .collect();
                }
                Interest(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.interests.contains(&value))
                        .cloned()
                        .collect();
                }
                Cause(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.causes.contains(&value))
                        .cloned()
                        .collect();
                }
                UpdatedOn(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| {
                            profile.updated_on >= value.start_date
                                && profile.updated_on <= value.end_date
                        })
                        .cloned()
                        .collect();
                }
                CreatedOn(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| {
                            profile.created_on >= value.start_date
                                && profile.created_on <= value.end_date
                        })
                        .cloned()
                        .collect();
                }
                StateOrProvince(value) => {
                    profiles = profiles
                        .iter()
                        .filter(|profile| profile.state_or_province == value)
                        .cloned()
                        .collect();
                }
            }
        }
        profiles
    }

    // Method to map a profile to a profile response
    fn _map_profile_to_profile_response(identifier: String, profile: Profile) -> ProfileResponse {
        ProfileResponse {
            identifier: Principal::from_text(identifier).unwrap_or(Principal::anonymous()),
            principal: profile.principal,
            username: profile.username,
            display_name: profile.display_name,
            application_role: profile.application_role,
            first_name: profile.first_name,
            last_name: profile.last_name,
            privacy: profile.privacy,
            about: profile.about,
            email: profile.email,
            date_of_birth: profile.date_of_birth,
            city: profile.city,
            state_or_province: profile.state_or_province,
            country: profile.country,
            profile_image: profile.profile_image,
            banner_image: profile.banner_image,
            skills: profile.skills,
            interests: profile.interests,
            causes: profile.causes,
            website: profile.website,
            code_of_conduct: profile.code_of_conduct,
            wallets: profile
                .wallets
                .into_iter()
                .map(|(k, v)| WalletResponse {
                    provider: v.provider,
                    principal: k,
                    is_primary: v.is_primary,
                })
                .collect(),
            extra: profile.extra,
            updated_on: profile.updated_on,
            created_on: profile.created_on,
            member_identifier: profile.member_identifier,
            privacy_policy: profile.privacy_policy,
            terms_of_service: profile.terms_of_service,
        }
    }

    //  Method to get a profile from a caller
    fn _get_profile_from_caller(caller: Principal) -> Option<(Principal, Profile)> {
        let profiles = ENTRIES.with(|entries| Data::get_entries(entries));
        if let Some(reponse) = profiles
            .into_iter()
            .find(|(_identifier, _profile)| _profile.principal == caller)
        {
            return Some((
                Principal::from_text(reponse.0).unwrap_or(Principal::anonymous()),
                reponse.1,
            ));
        } else {
            return None;
        }
    }

    // default profile_not_found error
    fn _profile_not_found_error(method_name: &str, inputs: Option<Vec<String>>) -> ApiError {
        api_error(
            ApiErrorType::NotFound,
            "PROFILE_NOT_FOUND",
            "Profile not found",
            STABLE_DATA
                .with(|data| Data::get_name(data.borrow().get()))
                .as_str(),
            method_name,
            inputs,
        )
    }

    // Used for composite_query calls from the parent canister
    //
    // Method to get filtered profiles serialized and chunked
    pub fn get_chunked_data(
        filters: Vec<ProfileFilter>,
        chunk: usize,
        max_bytes_per_chunk: usize,
    ) -> (Vec<u8>, (usize, usize)) {
        let profiles = ENTRIES.with(|data| Data::get_entries(data));
        // get profiles for filtering
        let mapped_profiles: Vec<ProfileResponse> = profiles
            .iter()
            .map(|(_identifier, _profile_data)| {
                Self::_map_profile_to_profile_response(_identifier.clone(), _profile_data.clone())
            })
            .collect();

        // filter profiles
        let filtered_profiles = Self::_get_filtered_profiles(mapped_profiles, filters);
        if let Ok(bytes) = serialize(&filtered_profiles) {
            // Check if the bytes of the serialized profiles are greater than the max bytes per chunk specified as an argument
            if bytes.len() >= max_bytes_per_chunk {
                // Get the start and end index of the bytes to be returned
                let start = chunk * max_bytes_per_chunk;
                let end = (chunk + 1) * (max_bytes_per_chunk);

                // Get the bytes to be returned, if the end index is greater than the length of the bytes, return the remaining bytes
                let response = if end >= bytes.len() {
                    bytes[start..].to_vec()
                } else {
                    bytes[start..end].to_vec()
                };

                // Determine the max number of chunks that can be returned, a float is used because the number of chunks can be a decimal in this step
                let mut max_chunks: f64 = 0.00;
                if max_bytes_per_chunk < bytes.len() {
                    max_chunks = (bytes.len() / max_bytes_per_chunk) as f64;
                }

                // return the response and start and end chunk index, the end chunk index is calculated by rounding up the max chunks
                return (response, (chunk, max_chunks.ceil() as usize));
            }
            // if the bytes of the serialized profiles are less than the max bytes per chunk specified as an argument, return the bytes and start and end chunk index as 0
            return (bytes, (0, 0));
        } else {
            // if the profiles cant be serialized return an empty vec and start and end chunk index as 0
            return (vec![], (0, 0));
        }
    }

    pub fn add_friend_request(
        requested_by: Principal,
        to: Principal,
        message: String,
    ) -> Result<FriendRequestResponse, ApiError> {
        FRIEND_REQUEST.with(|r| {
            let mut requests = r.borrow_mut();

            // If the requester puts out a second friend request for the user
            if requests
                .iter()
                .any(|(_, r)| r.requested_by == requested_by && r.to == to)
            {
                return Err(api_error(
                    ApiErrorType::BadRequest,
                    "ALREADY_REQUESTED",
                    "You already sent a friend request to this user",
                    STABLE_DATA
                        .with(|data| Data::get_name(data.borrow().get()))
                        .as_str(),
                    "friend_request",
                    None,
                ));
            }

            // if the "to" has already sent a request to the "requested_by"
            if requests
                .iter()
                .any(|(_, r)| r.requested_by == to && r.to == requested_by)
            {
                return Err(api_error(
                    ApiErrorType::BadRequest,
                    "PENDING_REQUEST",
                    "The invited user already send you a friend request",
                    STABLE_DATA
                        .with(|data| Data::get_name(data.borrow().get()))
                        .as_str(),
                    "friend_request",
                    None,
                ));
            }

            let id = requests.last_key_value().map(|(k, _)| k + 1).unwrap_or(0);

            let request = FriendRequest {
                requested_by,
                message: message.clone(),
                to,
                created_at: time(),
            };

            requests.insert(id.clone(), request.clone());

            let display_name = Self::get_profile_by_user_principal(requested_by)
                .map_or("unknown".to_string(), |p| p.display_name);

            let metadata = json!({
                "receivedBy": display_name,
                "receivedByPrincipal": requested_by.to_string(),
                "message": message,
                "isProcessed": false,
            });

            Self::send_notification().friend_request_notification(
                requested_by.clone(),
                FriendRequestNotificationData {
                    friend_request_id: id.clone(),
                    from: requested_by.clone(),
                    to,
                    accepted: None,
                },
                vec![to.clone()],
                metadata.to_string(),
            );

            Ok(FriendRequestResponse {
                id,
                requested_by,
                message: request.message.clone(),
                to,
                created_at: request.created_at,
            })
        })
    }

    pub fn get_friend_requests(caller: Principal) -> Vec<FriendRequestResponse> {
        FRIEND_REQUEST.with(|r| {
            let requests = r.borrow();

            requests
                .iter()
                .filter(|(_, r)| r.requested_by == caller || r.to == caller)
                .map(|(k, v)| FriendRequestResponse {
                    id: k.clone(),
                    requested_by: v.requested_by,
                    message: v.message.clone(),
                    to: v.to,
                    created_at: v.created_at,
                })
                .collect()
        })
    }

    pub fn accept_friend_request(caller: Principal, id: u64) -> Result<bool, String> {
        FRIEND_REQUEST.with(|r| {
            let mut requests = r.borrow_mut();

            if let Some(request) = requests.get(&id) {
                if request.to != caller {
                    return Err("Request not found".to_string());
                }
                let profiles = ENTRIES.with(|data| Data::get_entries(data));

                let mut caller_profile = profiles
                    .iter()
                    .find(|(_, p)| p.principal == request.to)
                    .unwrap()
                    .clone();

                caller_profile
                    .1
                    .relations
                    .insert(request.requested_by, RelationType::Friend.to_string());

                let mut to_profile = profiles
                    .iter()
                    .find(|(_, p)| p.principal == request.requested_by)
                    .unwrap()
                    .clone();

                to_profile
                    .1
                    .relations
                    .insert(request.to, RelationType::Friend.to_string());

                ENTRIES.with(|entries| {
                    let _ = STABLE_DATA.with(|data| {
                        let _ = Data::update_entry(
                            data,
                            entries,
                            Principal::from_text(caller_profile.0).unwrap(),
                            caller_profile.1,
                        );
                        let _ = Data::update_entry(
                            data,
                            entries,
                            Principal::from_text(to_profile.0.clone()).unwrap(),
                            to_profile.1.clone(),
                        );
                    });
                });
                requests.remove(&id);

                let display_name = Self::get_profile_by_user_principal(caller)
                    .map_or("unknown".to_string(), |p| p.display_name);

                let metadata = json!({
                    "acceptedBy": display_name,
                    "acceptedByPrincipal": caller.to_string(),
                });

                Self::send_notification().friend_request_notification(
                    request.requested_by.clone(),
                    FriendRequestNotificationData {
                        friend_request_id: id.clone(),
                        from: request.requested_by.clone(),
                        to: request.to.clone(),
                        accepted: Some(true),
                    },
                    vec![request.requested_by],
                    metadata.to_string(),
                );
                return Ok(true);
            }

            Err("Request not found".to_string())
        })
    }

    pub fn remove_friend(caller: Principal, to_remove: Principal) -> Result<bool, String> {
        let profiles = ENTRIES.with(|data| Data::get_entries(data));

        let mut caller_profile = profiles
            .iter()
            .find(|(_, p)| p.principal == caller)
            .unwrap()
            .clone();

        caller_profile.1.relations.remove(&to_remove);

        let mut to_remove_profile = profiles
            .iter()
            .find(|(_, p)| p.principal == to_remove)
            .unwrap()
            .clone();

        to_remove_profile.1.relations.remove(&caller);

        ENTRIES.with(|entries| {
            STABLE_DATA.with(|data| {
                let _ = Data::update_entry(
                    data,
                    entries,
                    Principal::from_text(caller_profile.0).unwrap(),
                    caller_profile.1,
                );
                let _ = Data::update_entry(
                    data,
                    entries,
                    Principal::from_text(to_remove_profile.0).unwrap(),
                    to_remove_profile.1,
                );
            });
        });

        Self::send_notification().friend_remove_notification(to_remove, "{}".to_string());

        Ok(true)
    }

    pub fn clear_relations(caller: Principal) -> bool {
        let profiles = ENTRIES.with(|data| Data::get_entries(data));

        let mut caller_profile = profiles
            .iter()
            .find(|(_, p)| p.principal == caller)
            .unwrap()
            .clone();

        caller_profile.1.relations.clear();

        ENTRIES.with(|entries| {
            STABLE_DATA.with(|data| {
                let _ = Data::update_entry(
                    data,
                    entries,
                    Principal::from_text(caller_profile.0).unwrap(),
                    caller_profile.1,
                );
            });
        });

        true
    }

    pub fn decline_friend_request(caller: Principal, id: u64) -> Result<bool, String> {
        FRIEND_REQUEST.with(|r| {
            let mut requests = r.borrow_mut();

            if let Some(request) = requests.get(&id) {
                if request.to == caller {
                    let display_name = Self::get_profile_by_user_principal(caller)
                        .map_or("unknown".to_string(), |p| p.display_name);

                    let metadata = json!({
                        "declinedBy": display_name,
                        "declinedByPrincipal": caller.to_string(),
                    });

                    Self::send_notification().friend_request_notification(
                        request.requested_by.clone(),
                        FriendRequestNotificationData {
                            friend_request_id: id.clone(),
                            from: request.requested_by.clone(),
                            to: request.to.clone(),
                            accepted: Some(false),
                        },
                        vec![request.requested_by],
                        metadata.to_string(),
                    );

                    requests.remove(&id);
                    return Ok(true);
                }
            }

            Err("Request not found".to_string())
        })
    }

    pub fn remove_friend_request(caller: Principal, id: u64) -> Result<bool, String> {
        FRIEND_REQUEST.with(|r| {
            let mut requests = r.borrow_mut();

            if let Some(request) = requests.get(&id) {
                if request.requested_by == caller {
                    requests.remove(&id);
                    return Ok(true);
                }
            }

            Err("Request not found".to_string())
        })
    }

    pub fn block_user(caller: Principal, to_block: Principal) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller.to_string()),
            format!("relation_identifier - {:?}", &to_block.to_string()),
        ]);

        // get the profile from the data store
        match Self::_get_profile_from_caller(caller) {
            // If the profile does not exist, return an error
            None => Err(Self::_profile_not_found_error("block_user", inputs)),
            // If the profile exists, continue
            Some((_identifier, mut _profile)) => {
                // Add the relation to the profile, if existing it will be overwritten
                _profile
                    .relations
                    .insert(to_block, RelationType::Blocked.to_string());

                // Update the profile in the data store
                ENTRIES
                    .with(|entries| {
                        STABLE_DATA
                            .with(|data| Data::update_entry(data, entries, _identifier, _profile))
                    })
                    .map_or_else(
                        |err| Err(err),
                        |result| {
                            Ok(Self::_map_profile_to_profile_response(
                                result.0.to_string(),
                                result.1,
                            ))
                        },
                    )
            }
        }
    }

    pub fn unblock_user(
        caller: Principal,
        to_unblock: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller.to_string()),
            format!("relation_identifier - {:?}", &to_unblock.to_string()),
        ]);

        // get the profile from the data store
        match Self::_get_profile_from_caller(caller) {
            // If the profile does not exist, return an error
            None => Err(Self::_profile_not_found_error("block_user", inputs)),
            // If the profile exists, continue
            Some((_identifier, mut _profile)) => {
                // Add the relation to the profile, if existing it will be overwritten
                _profile.relations.remove(&to_unblock);

                // Update the profile in the data store
                ENTRIES
                    .with(|entries| {
                        STABLE_DATA
                            .with(|data| Data::update_entry(data, entries, _identifier, _profile))
                    })
                    .map_or_else(
                        |err| Err(err),
                        |result| {
                            Ok(Self::_map_profile_to_profile_response(
                                result.0.to_string(),
                                result.1,
                            ))
                        },
                    )
            }
        }
    }

    fn get_environment() -> Option<Environment> {
        let canister_id = id().to_string();
        if canister_id == "4vy4w-gaaaa-aaaap-aa4pa-cai".to_string() {
            return Some(Environment::Production);
        }
        if canister_id == "5ycyv-iiaaa-aaaap-abgia-cai" {
            return Some(Environment::Staging);
        } else {
            // if its development or marketing
            return Some(Environment::Development);
        }
    }

    fn send_notification() -> Notification {
        let environment = Self::get_environment().expect("Environment not found");
        Notification::new(environment)
    }
}
