use std::{collections::HashMap, iter::FromIterator};

use candid::{candid_method, Principal};
use ic_cdk_macros::{query, update};

use ic_cdk::caller;
use ic_scalable_misc::enums::api_error_type::ApiError;

use shared::profile_models::{
    PostProfile, PostWallet, Profile, ProfileFilter, ProfileResponse, RelationType, UpdateProfile,
};

use super::store::{Store, DATA};

// temporary method to add profiles to the canister
#[update]
#[candid_method(update)]
pub fn migration_add_profiles(profiles: Vec<(Principal, Profile)>) -> () {
    if caller()
        == Principal::from_text("ledm3-52ncq-rffuv-6ed44-hg5uo-iicyu-pwkzj-syfva-heo4k-p7itq-aqe")
            .unwrap()
    {
        DATA.with(|data| {
            data.borrow_mut().current_entry_id = profiles.clone().len() as u64;
            data.borrow_mut().entries = HashMap::from_iter(profiles);
        })
    }
}

// This method is used to add a profile to the canister,
// The method is async because it optionally creates a new canister is created
#[update]
#[candid_method(update)]
pub async fn add_profile(
    post_profile: PostProfile,
    member_canister: Principal,
) -> Result<ProfileResponse, ApiError> {
    Store::add_profile(caller(), post_profile, member_canister).await
}

// This method is used to get a single profile by an user principal
#[query]
#[candid_method(query)]
pub fn get_profile_by_user_principal(principal: Principal) -> Result<ProfileResponse, ApiError> {
    Store::get_profile_by_user_principal(principal)
}

// This method is used to get a single profile by an identifier
#[query]
#[candid_method(query)]
pub fn get_profile_by_identifier(id: Principal) -> Result<ProfileResponse, ApiError> {
    Store::get_profile_by_identifier(id)
}

// This method is used to get multiple profiles by principals
#[query]
#[candid_method(query)]
pub fn get_profiles_by_user_principal(principals: Vec<Principal>) -> Vec<ProfileResponse> {
    Store::get_profiles_by_user_principal(principals)
}

// This method is used to get multiple profiles by identifiers
#[query]
#[candid_method(query)]
pub fn get_profiles_by_identifier(identifiers: Vec<Principal>) -> Vec<ProfileResponse> {
    Store::get_profiles_by_identifier(identifiers)
}

// This method is used to edit a profile
#[update]
#[candid_method(update)]
pub fn edit_profile(update_profile: UpdateProfile) -> Result<ProfileResponse, ApiError> {
    Store::update_profile(caller(), update_profile)
}

// This method is used to add a wallet reference to the profile
#[update]
#[candid_method(update)]
pub fn add_wallet(wallet: PostWallet) -> Result<ProfileResponse, ApiError> {
    Store::add_wallet(caller(), wallet)
}

// This method is used to set a wallet as primary
#[update]
#[candid_method(update)]
pub fn set_wallet_as_primary(wallet_principal: Principal) -> Result<(), ()> {
    Store::set_wallet_as_primary(caller(), wallet_principal)
}

// This method is used to remove a wallet reference from the profile
#[update]
#[candid_method(update)]
pub fn remove_wallet(wallet: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_wallet(caller(), wallet)
}

// This method is used to add a starred reference to the profile, for example a starred event, group or task
#[update]
#[candid_method(update)]
pub fn add_starred(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::add_starred(caller(), identifier)
}

// This method is used to remove a starred reference from the profile
#[update]
#[candid_method(update)]
pub fn remove_starred(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_starred(caller(), identifier)
}

// This method is used to get all starred events
#[query]
#[candid_method(query)]
pub fn get_starred_events() -> Vec<Principal> {
    Store::get_starred(caller(), "evt".to_string())
}

// This method is used to get all starred tasks
#[query]
#[candid_method(query)]
pub fn get_starred_tasks() -> Vec<Principal> {
    Store::get_starred(caller(), "tsk".to_string())
}

// This method is used to get all starred groups
#[query]
#[candid_method(query)]
pub fn get_starred_groups() -> Vec<Principal> {
    Store::get_starred(caller(), "grp".to_string())
}

// This method adds a relation to the profile (Friend or Blocked)
#[update]
#[candid_method(update)]
pub fn add_relation(
    identifier: Principal,
    relation_type: RelationType,
) -> Result<ProfileResponse, ApiError> {
    Store::add_relation(caller(), relation_type, identifier)
}

// This method is used to get all relations of a specific type (Friend or Blocked)
#[query]
#[candid_method(query)]
pub fn get_relations(relation_type: RelationType) -> Vec<Principal> {
    Store::get_relations(caller(), relation_type)
}

// This method is used to remove a relation from the profile
#[update]
#[candid_method(update)]
pub fn remove_relation(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_relation(caller(), identifier)
}

// This method is used to approve the code of conduct for the specific caller
#[update]
#[candid_method(update)]
pub fn approve_code_of_conduct(version: u64) -> Result<bool, ApiError> {
    Store::approve_code_of_conduct(caller(), version)
}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get filtered profiles the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
#[query]
#[candid_method(query)]
fn get_chunked_data(
    filters: Vec<ProfileFilter>,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {
    if caller() != DATA.with(|data| data.borrow().parent) {
        return (vec![], (0, 0));
    }

    Store::get_chunked_data(filters, chunk, max_bytes_per_chunk)
}
