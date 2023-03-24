use std::collections::HashMap;

use candid::Principal;
use ic_cdk::api::{call, time};
use ic_scalable_canister::store::Data;

use ic_scalable_misc::enums::filter_type::FilterType;
use ic_scalable_misc::helpers::serialize_helper::serialize;
use ic_scalable_misc::models::identifier_model::Identifier;
use ic_scalable_misc::{
    enums::{
        api_error_type::{ApiError, ApiErrorType},
        application_role_type::ApplicationRole,
        asset_type::Asset,
        sort_type::SortDirection,
        whitelist_rights_type::WhitelistRights,
    },
    helpers::{error_helper::api_error, paging_helper::get_paged_data},
    models::paged_response_models::PagedResponse,
};

use shared::profile_models::{
    CodeOfConductDetails, PostProfile, PostWallet, Profile, ProfileFilter, ProfileResponse,
    ProfileSort, RelationType, UpdateProfile, Wallet, WalletResponse,
};

use std::cell::RefCell;

use super::validation::{validate_post_profile, validate_update_profile};

thread_local! {
    pub static DATA: RefCell<Data<Profile>> = RefCell::new(Data::default());
}

pub struct Store;

impl Store {
    pub async fn add_profile(
        caller: Principal,
        post_profile: PostProfile,
        member_canister: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("post_profile - {:?}", &post_profile),
        ]);

        match Self::_get_profile_from_caller(caller) {
            Some(_) => Err(api_error(
                ApiErrorType::BadRequest,
                "ALREADY_REGISTERED",
                "User profile already registered",
                DATA.with(|data| Data::get_name(data)).as_str(),
                "add_profile",
                inputs,
            )),
            None => match validate_post_profile(post_profile.clone()) {
                Err(err) => Err(err),
                Ok(_) => {
                    if Self::_has_user_name(
                        &DATA.with(|data| Data::get_entries(data)),
                        &post_profile.username,
                    ) {
                        return Err(api_error(
                            ApiErrorType::BadRequest,
                            "USERNAME_TAKEN",
                            "Username already taken",
                            DATA.with(|data| Data::get_name(data)).as_str(),
                            "add_profile",
                            inputs,
                        ));
                    }

                    let empty = "".to_string();

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
                        code_of_conduct: CodeOfConductDetails {
                            approved_version: 0,
                            approved_date: 0,
                        },
                        extra: post_profile.extra,
                        updated_on: time(),
                        created_on: time(),
                        member_identifier: Principal::anonymous(),
                    };
                    let new_entry = DATA.with(|data| {
                        Data::add_entry(data, profile.clone(), Some("pfe".to_string()))
                    });

                    match new_entry {
                        Err(err) => match err {
                            ApiError::CanisterAtCapacity(message) => {
                                let _data = DATA.with(|v| v.borrow().clone());
                                match Data::spawn_sibling(_data, profile).await {
                                    Ok(_) => Err(ApiError::CanisterAtCapacity(message)),
                                    Err(err) => Err(err),
                                }
                            }
                            _ => Err(err),
                        },
                        Ok((identifier, mut profile)) => {
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
                                        DATA.with(|data| {
                                            profile.member_identifier = _member_identifier;
                                            let _ = Data::update_entry(
                                                data,
                                                identifier,
                                                profile.clone(),
                                            );
                                        });
                                        Ok(Self::_map_profile_to_profile_response(
                                            identifier, profile,
                                        ))
                                    }
                                    Err(err) => Err(err),
                                },
                                Err(err) => Err(api_error(
                                    ApiErrorType::Unexpected,
                                    "ICC_MEMBER_CREATION_FAILED",
                                    err.1.as_str(),
                                    DATA.with(|data| Data::get_name(data)).as_str(),
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

    pub fn update_profile(
        caller: Principal,
        update_profile: UpdateProfile,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("update_profile - {:?}", &update_profile),
        ]);
        DATA.with(|data| match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error("update_profile", inputs)),
            Some((_identifier, profile)) => match validate_update_profile(update_profile.clone()) {
                Err(err) => Err(err),
                Ok(_) => {
                    let email = match update_profile.email {
                        None => "".to_string(),
                        Some(_email) => _email,
                    };

                    if email != "" {
                        if profile.email != email
                            && Self::_has_email(&Data::get_entries(data), &email)
                        {
                            return Err(api_error(
                                ApiErrorType::BadRequest,
                                "EMAIL_TAKEN",
                                "Email already taken",
                                DATA.with(|data| Data::get_name(data)).as_str(),
                                "update_profile",
                                inputs,
                            ));
                        }
                    }

                    let updated_profile = Profile {
                        principal: caller,
                        username: profile.username.clone(),
                        display_name: update_profile.display_name,
                        application_role: profile.application_role.clone(),
                        first_name: update_profile.first_name,
                        last_name: update_profile.last_name,
                        privacy: update_profile.privacy,
                        about: update_profile.about,
                        email,
                        date_of_birth: update_profile.date_of_birth,
                        city: update_profile.city,
                        state_or_province: update_profile.state_or_province,
                        country: update_profile.country,
                        profile_image: update_profile.profile_image,
                        banner_image: update_profile.banner_image,
                        skills: update_profile.skills,
                        interests: update_profile.interests,
                        causes: update_profile.causes,
                        website: update_profile.website,
                        code_of_conduct: profile.code_of_conduct.clone(),
                        wallets: profile.wallets.clone(),
                        starred: profile.starred.clone(),
                        relations: profile.relations.clone(),
                        extra: update_profile.extra,
                        updated_on: time(),
                        created_on: profile.created_on,
                        member_identifier: profile.member_identifier,
                    };
                    match DATA
                        .with(|data| Data::update_entry(data, _identifier, updated_profile.clone()))
                    {
                        Err(err) => Err(err),
                        Ok((identifier, profile)) => {
                            Ok(Self::_map_profile_to_profile_response(identifier, profile))
                        }
                    }
                }
            },
        })
    }

    pub fn add_wallet(caller: Principal, wallet: PostWallet) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("wallet - {:?}", &wallet),
        ]);

        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error("add_wallet", inputs)),
            Some((_identifier, mut _profile)) => {
                _profile.wallets.insert(
                    wallet.principal,
                    Wallet {
                        is_primary: false,
                        provider: wallet.provider,
                    },
                );
                DATA.with(|data| Data::update_entry(data, _identifier, _profile))
                    .map_or_else(
                        |err| Err(err),
                        |result| Ok(Self::_map_profile_to_profile_response(result.0, result.1)),
                    )
            }
        }
    }

    pub fn set_wallet_as_primary(caller: Principal, wallet_principal: Principal) -> Result<(), ()> {
        if let Some((_identifier, mut _profile)) = Store::_get_profile_from_caller(caller) {
            for (_wallet_principal, mut _wallet) in _profile.wallets.iter_mut() {
                _wallet.is_primary = false;
            }
            _profile
                .wallets
                .get_mut(&wallet_principal)
                .unwrap()
                .is_primary = true;
            if let Ok(_) = DATA.with(|data| Data::update_entry(data, _identifier, _profile)) {
                return Ok(());
            } else {
                return Err(());
            }
        } else {
            return Err(());
        }
    }

    pub fn remove_wallet(
        caller: Principal,
        wallet_principal: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("wallet - {:?}", &wallet_principal),
        ]);

        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error("remove_wallet", inputs)),
            Some((_identifier, mut _profile)) => {
                _profile.wallets.remove(&wallet_principal);

                DATA.with(|data| Data::update_entry(data, _identifier, _profile))
                    .map_or_else(
                        |err| Err(err),
                        |result| Ok(Self::_map_profile_to_profile_response(result.0, result.1)),
                    )
            }
        }
    }

    pub fn add_starred(
        caller: Principal,
        starred_identifier: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller.to_string()),
            format!("identifier - {:?}", &starred_identifier.to_string()),
        ]);

        let (_, _, kind) = Identifier::decode(&starred_identifier);
        if !vec!["grp".to_string(), "tsk".to_string(), "evt".to_string()].contains(&kind) {
            return Err(api_error(
                ApiErrorType::NotFound,
                "INVALID TYPE",
                format!("'{}' is not supported", kind).as_str(),
                DATA.with(|data| Data::get_name(data)).as_str(),
                "add_starred",
                inputs,
            ));
        }

        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error("add_starred", inputs)),
            Some((_identifier, mut _profile)) => {
                _profile.starred.insert(starred_identifier, kind);

                DATA.with(|data| Data::update_entry(data, _identifier, _profile))
                    .map_or_else(
                        |err| Err(err),
                        |result| Ok(Self::_map_profile_to_profile_response(result.0, result.1)),
                    )
            }
        }
    }

    pub fn remove_starred(
        caller: Principal,
        starred_identifier: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("identifier - {:?}", &starred_identifier),
        ]);

        let (_, _, kind) = Identifier::decode(&starred_identifier);
        if !vec!["grp".to_string(), "tsk".to_string(), "evt".to_string()].contains(&kind) {
            return Err(api_error(
                ApiErrorType::NotFound,
                "INVALID TYPE",
                format!("'{}' is not supported", kind).as_str(),
                DATA.with(|data| Data::get_name(data)).as_str(),
                "remove_starred",
                inputs,
            ));
        }

        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error("remove_starred", inputs)),
            Some((_identifier, mut _profile)) => {
                _profile.starred.remove(&starred_identifier);
                DATA.with(|data| Data::update_entry(data, _identifier, _profile))
                    .map_or_else(
                        |err| Err(err),
                        |result| Ok(Self::_map_profile_to_profile_response(result.0, result.1)),
                    )
            }
        }
    }

    pub fn get_starred(caller: Principal, kind: String) -> Vec<Principal> {
        let profile = Self::_get_profile_from_caller(caller);
        if let Some((_principal, _profile)) = profile {
            let mut starred = vec![];
            _profile
                .starred
                .into_iter()
                .for_each(|(_starred_identifier, _kind)| {
                    if _kind == kind {
                        starred.push(_starred_identifier)
                    }
                });
            return starred;
        };
        return vec![];
    }

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

        let (_, _, kind) = Identifier::decode(&relation_identifier);
        if &kind != &"pfe".to_string() {
            return Err(api_error(
                ApiErrorType::NotFound,
                "INVALID TYPE",
                format!("'{}' is not supported", kind).as_str(),
                DATA.with(|data| Data::get_name(data)).as_str(),
                "add_relation",
                inputs,
            ));
        }

        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error("add_relation", inputs)),
            Some((_identifier, mut _profile)) => {
                _profile
                    .relations
                    .insert(relation_identifier, relation_type.to_string());

                DATA.with(|data| Data::update_entry(data, _identifier, _profile))
                    .map_or_else(
                        |err| Err(err),
                        |result| Ok(Self::_map_profile_to_profile_response(result.0, result.1)),
                    )
            }
        }
    }

    pub fn get_relations(caller: Principal, relation_type: RelationType) -> Vec<Principal> {
        let profile = Self::_get_profile_from_caller(caller);
        if let Some((_principal, _profile)) = profile {
            let mut relations = vec![];
            _profile
                .relations
                .into_iter()
                .for_each(|(_relation_identifier, _relation_type)| {
                    if _relation_type == relation_type.to_string() {
                        relations.push(_relation_identifier);
                    }
                });
            return relations;
        };
        return vec![];
    }

    pub fn remove_relation(
        caller: Principal,
        relation_identifier: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        let inputs = Some(vec![
            format!("principal - {:?}", &caller),
            format!("relation_identifier - {:?}", &relation_identifier),
        ]);

        let (_, _, kind) = Identifier::decode(&relation_identifier);
        if &kind != &"pfe".to_string() {
            return Err(api_error(
                ApiErrorType::NotFound,
                "INVALID TYPE",
                format!("'{}' is not supported", kind).as_str(),
                DATA.with(|data| Data::get_name(data)).as_str(),
                "remove_starred",
                inputs,
            ));
        }

        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error("remove_relation", inputs)),
            Some((_identifier, mut _profile)) => {
                _profile.relations.remove(&relation_identifier);

                DATA.with(|data| Data::update_entry(data, _identifier, _profile))
                    .map_or_else(
                        |err| Err(err),
                        |result| Ok(Self::_map_profile_to_profile_response(result.0, result.1)),
                    )
            }
        }
    }

    pub fn get_profile_by_user_principal(
        principal: Principal,
    ) -> Result<ProfileResponse, ApiError> {
        match Self::_get_profile_from_caller(principal) {
            None => Err(Self::_profile_not_found_error(
                "get_profile_by_user_principal",
                None,
            )),
            Some((_identifier, profile)) => {
                Ok(Self::_map_profile_to_profile_response(_identifier, profile))
            }
        }
    }

    pub fn get_profile_by_identifier(identifier: Principal) -> Result<ProfileResponse, ApiError> {
        match DATA.with(|data| Data::get_entry(data, identifier)) {
            Err(err) => Err(err),
            Ok((_identifier, profile)) => {
                Ok(Self::_map_profile_to_profile_response(_identifier, profile))
            }
        }
    }

    pub fn get_profiles_by_user_principal(principals: Vec<Principal>) -> Vec<ProfileResponse> {
        let fetched_profiles = DATA.with(|data| Data::get_entries(data));

        principals
            .into_iter()
            .filter_map(|principal| {
                fetched_profiles
                    .iter()
                    .find(|f| f.1.principal == principal)
                    .map(|(_identifier, profile)| {
                        Self::_map_profile_to_profile_response(_identifier.clone(), profile.clone())
                    })
            })
            .collect()
    }

    pub fn get_profiles_by_identifier(profile_identifiers: Vec<Principal>) -> Vec<ProfileResponse> {
        let mut profiles: Vec<ProfileResponse> = vec![];

        for identifier in profile_identifiers {
            if let Ok((_identifier, profile)) = DATA.with(|data| Data::get_entry(data, identifier))
            {
                profiles.push(Self::_map_profile_to_profile_response(_identifier, profile));
            }
        }

        profiles
    }

    pub fn approve_code_of_conduct(caller: Principal, version: u64) -> Result<bool, ApiError> {
        match Self::_get_profile_from_caller(caller) {
            None => Err(Self::_profile_not_found_error(
                "approve_code_of_conduct",
                None,
            )),
            Some((_identifier, mut _existing)) => {
                _existing.code_of_conduct = CodeOfConductDetails {
                    approved_version: version,
                    approved_date: time(),
                };

                let _ = DATA.with(|data| Data::update_entry(data, _identifier, _existing));
                Ok(true)
            }
        }
    }

    pub fn get_paged_profiles_by_id(
        identifiers: Vec<Principal>,
        limit: usize,
        page: usize,
        filters: Vec<ProfileFilter>,
        sort: ProfileSort,
    ) -> PagedResponse<ProfileResponse> {
        let mut profiles: Vec<ProfileResponse> = vec![];
        DATA.with(|data| {
            identifiers.into_iter().for_each(|identifier| {
                if let Ok((_identifier, _profile)) = Data::get_entry(&data, identifier) {
                    profiles.push(Self::_map_profile_to_profile_response(
                        _identifier,
                        _profile,
                    ))
                };
            });
            let filtered_profiles = Self::_get_filtered_profiles(profiles, filters);
            let ordered_profiles = Self::_get_ordered_profiles(filtered_profiles, sort);
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
        let mut profiles: Vec<ProfileResponse> = vec![];
        DATA.with(|data| {
            let all_profiles = Data::get_entries(data);
            principals.into_iter().for_each(|p| {
                if let Some((_identifier, _profile)) =
                    all_profiles.iter().find(|(_, _p)| _p.principal == p)
                {
                    profiles.push(Self::_map_profile_to_profile_response(
                        _identifier.clone(),
                        _profile.clone(),
                    ));
                };
            });
            let filtered_profiles = Self::_get_filtered_profiles(profiles, filters);
            let ordered_profiles = Self::_get_ordered_profiles(filtered_profiles, sort);
            get_paged_data(ordered_profiles, limit, page)
        })
    }

    pub fn update_application_role(
        principal: Principal,
        application_role: ApplicationRole,
    ) -> Result<ProfileResponse, ApiError> {
        match Self::_get_profile_from_caller(principal) {
            None => Err(Self::_profile_not_found_error(
                "update_application_role",
                None,
            )),
            Some((_, mut _existing)) => {
                _existing.application_role = application_role;
                _existing.updated_on = time();
                match DATA.with(|data| Data::update_entry(data, principal, _existing)) {
                    Err(err) => Err(err),
                    Ok((identifier, profile)) => {
                        Ok(Self::_map_profile_to_profile_response(identifier, profile))
                    }
                }
            }
        }
    }

    pub fn get_application_role(principal: Principal) -> Result<ApplicationRole, ApiError> {
        match Self::_get_profile_from_caller(principal) {
            None => Err(Self::_profile_not_found_error(
                "update_application_role",
                None,
            )),
            Some((_identifier, _profile)) => Ok(_profile.application_role),
        }
    }

    fn _has_user_name(profiles: &Vec<(Principal, Profile)>, username: &String) -> bool {
        let profile = profiles
            .iter()
            .find(|(_, profile)| &profile.username == username);
        match profile {
            None => false,
            Some(_) => true,
        }
    }

    fn _has_email(profiles: &Vec<(Principal, Profile)>, email: &String) -> bool {
        let profile = profiles.iter().find(|(_, profile)| &profile.email == email);
        match profile {
            None => false,
            Some(_) => true,
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

    fn _map_profile_to_profile_response(
        identifier: Principal,
        profile: Profile,
    ) -> ProfileResponse {
        ProfileResponse {
            identifier,
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
        }
    }

    fn _get_profile_from_caller(caller: Principal) -> Option<(Principal, Profile)> {
        let profiles = DATA.with(|data| Data::get_entries(data));
        profiles
            .into_iter()
            .find(|(_identifier, _profile)| _profile.principal == caller)
    }

    fn _profile_not_found_error(method_name: &str, inputs: Option<Vec<String>>) -> ApiError {
        api_error(
            ApiErrorType::NotFound,
            "PROFILE_NOT_FOUND",
            "Profile not found",
            DATA.with(|data| Data::get_name(data)).as_str(),
            method_name,
            inputs,
        )
    }

    pub fn get_chunked_data(
        filters: Vec<ProfileFilter>,
        filter_type: FilterType,
        chunk: usize,
        max_bytes_per_chunk: usize,
    ) -> (Vec<u8>, (usize, usize)) {
        let groups = DATA.with(|data| Data::get_entries(data));
        let mapped_groups: Vec<ProfileResponse> = groups
            .iter()
            .map(|(_identifier, _group_data)| {
                Self::_map_profile_to_profile_response(_identifier.clone(), _group_data.clone())
            })
            .collect();

        let filtered_groups = Self::_get_filtered_profiles(mapped_groups, filters);
        if let Ok(bytes) = serialize(&filtered_groups) {
            if bytes.len() >= max_bytes_per_chunk {
                let start = chunk * max_bytes_per_chunk;
                let end = (chunk + 1) * (max_bytes_per_chunk);

                let response = if end >= bytes.len() {
                    bytes[start..].to_vec()
                } else {
                    bytes[start..end].to_vec()
                };

                let mut max_chunks: f64 = 0.00;
                if max_bytes_per_chunk < bytes.len() {
                    max_chunks = (bytes.len() / max_bytes_per_chunk) as f64;
                }
                return (response, (chunk, max_chunks.ceil() as usize));
            }
            return (bytes, (0, 0));
        } else {
            return (vec![], (0, 0));
        }
    }
}
