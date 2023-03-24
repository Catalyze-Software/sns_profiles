use candid::{candid_method, Principal};
use ic_cdk_macros::{query, update};

use ic_cdk::caller;
use ic_scalable_misc::enums::{
    api_error_type::ApiError, application_role_type::ApplicationRole, filter_type::FilterType,
};

use shared::profile_models::{
    PostProfile, PostWallet, ProfileFilter, ProfileResponse, RelationType, UpdateProfile,
};

use super::store::{Store, DATA};

#[update]
#[candid_method(update)]
pub async fn add_profile(
    post_profile: PostProfile,
    member_canister: Principal,
) -> Result<ProfileResponse, ApiError> {
    Store::add_profile(caller(), post_profile, member_canister).await
}

#[update]
#[candid_method(update)]
pub fn edit_profile(update_profile: UpdateProfile) -> Result<ProfileResponse, ApiError> {
    Store::update_profile(caller(), update_profile)
}

#[query]
#[candid_method(update)]
pub fn get_profile_by_user_principal(principal: Principal) -> Result<ProfileResponse, ApiError> {
    Store::get_profile_by_user_principal(principal)
}

#[query]
#[candid_method(update)]
pub fn get_profile_by_identifier(id: Principal) -> Result<ProfileResponse, ApiError> {
    Store::get_profile_by_identifier(id)
}

#[query]
#[candid_method(update)]
pub fn get_profiles_by_user_principal(principals: Vec<Principal>) -> Vec<ProfileResponse> {
    Store::get_profiles_by_user_principal(principals)
}

#[query]
#[candid_method(update)]
pub fn get_profiles_by_identifier(identifiers: Vec<Principal>) -> Vec<ProfileResponse> {
    Store::get_profiles_by_identifier(identifiers)
}

#[query]
#[candid_method(query)]
pub fn get_application_role() -> Result<ApplicationRole, ApiError> {
    Store::get_application_role(caller())
}

#[update]
#[candid_method(update)]
pub fn add_wallet(wallet: PostWallet) -> Result<ProfileResponse, ApiError> {
    Store::add_wallet(caller(), wallet)
}

#[update]
#[candid_method(update)]
pub fn set_wallet_as_primary(wallet_principal: Principal) -> Result<(), ()> {
    Store::set_wallet_as_primary(caller(), wallet_principal)
}

#[update]
#[candid_method(update)]
pub fn remove_wallet(wallet: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_wallet(caller(), wallet)
}

#[update]
#[candid_method(update)]
pub fn add_starred(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::add_starred(caller(), identifier)
}

#[update]
#[candid_method(update)]
pub fn remove_starred(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_starred(caller(), identifier)
}

#[query]
#[candid_method(query)]
pub fn get_starred_events() -> Vec<Principal> {
    Store::get_starred(caller(), "evt".to_string())
}

#[query]
#[candid_method(query)]
pub fn get_starred_tasks() -> Vec<Principal> {
    Store::get_starred(caller(), "tsk".to_string())
}

#[query]
#[candid_method(query)]
pub fn get_starred_groups() -> Vec<Principal> {
    Store::get_starred(caller(), "grp".to_string())
}

#[update]
#[candid_method(update)]
pub fn add_relation(
    identifier: Principal,
    relation_type: RelationType,
) -> Result<ProfileResponse, ApiError> {
    Store::add_relation(caller(), relation_type, identifier)
}

#[query]
#[candid_method(query)]
pub fn get_relations(relation_type: RelationType) -> Vec<Principal> {
    Store::get_relations(caller(), relation_type)
}

#[update]
#[candid_method(update)]
pub fn remove_relation(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_relation(caller(), identifier)
}

#[update]
#[candid_method(update)]
pub fn approve_code_of_conduct(version: u64) -> Result<bool, ApiError> {
    Store::approve_code_of_conduct(caller(), version)
}

#[query]
#[candid_method(query)]
fn get_chunked_data(
    filters: Vec<ProfileFilter>,
    filter_type: FilterType,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {
    if caller() != DATA.with(|data| data.borrow().parent) {
        return (vec![], (0, 0));
    }

    Store::get_chunked_data(filters, filter_type, chunk, max_bytes_per_chunk)
}
