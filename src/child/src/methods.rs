use std::{collections::HashMap, iter::FromIterator};

use candid::Principal;

use ic_cdk::{caller, query, update};
use ic_scalable_misc::enums::api_error_type::ApiError;

use shared::profile_models::{
    FriendRequestResponse, PostProfile, PostWallet, Profile, ProfileFilter, ProfileResponse,
    RelationType, UpdateProfile,
};

use super::store::{Store, DATA};

// temporary method to add profiles to the canister
#[update]
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
pub async fn add_profile(
    post_profile: PostProfile,
    member_canister: Principal,
) -> Result<ProfileResponse, ApiError> {
    Store::add_profile(caller(), post_profile, member_canister).await
}

// This method is used to get a single profile by an user principal
#[query]
pub fn get_profile_by_user_principal(principal: Principal) -> Result<ProfileResponse, ApiError> {
    Store::get_profile_by_user_principal(principal)
}

// This method is used to get a single profile by an identifier
#[query]
pub fn get_profile_by_identifier(id: Principal) -> Result<ProfileResponse, ApiError> {
    Store::get_profile_by_identifier(id)
}

// This method is used to get multiple profiles by principals
#[query]
pub fn get_profiles_by_user_principal(principals: Vec<Principal>) -> Vec<ProfileResponse> {
    Store::get_profiles_by_user_principal(principals)
}

// This method is used to get multiple profiles by identifiers
#[query]
pub fn get_profiles_by_identifier(identifiers: Vec<Principal>) -> Vec<ProfileResponse> {
    Store::get_profiles_by_identifier(identifiers)
}

// This method is used to edit a profile
#[update]
pub fn edit_profile(update_profile: UpdateProfile) -> Result<ProfileResponse, ApiError> {
    Store::update_profile(caller(), update_profile)
}

// This method is used to add a wallet reference to the profile
#[update]
pub fn add_wallet(wallet: PostWallet) -> Result<ProfileResponse, ApiError> {
    Store::add_wallet(caller(), wallet)
}

// This method is used to set a wallet as primary
#[update]
pub fn set_wallet_as_primary(wallet_principal: Principal) -> Result<(), ()> {
    Store::set_wallet_as_primary(caller(), wallet_principal)
}

// This method is used to remove a wallet reference from the profile
#[update]
pub fn remove_wallet(wallet: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_wallet(caller(), wallet)
}

// This method is used to add a starred reference to the profile, for example a starred event, group or task
#[update]
pub fn add_starred(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::add_starred(caller(), identifier)
}

// This method is used to remove a starred reference from the profile
#[update]
pub fn remove_starred(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_starred(caller(), identifier)
}

// This method is used to get all starred events
#[query]
pub fn get_starred_events() -> Vec<Principal> {
    Store::get_starred(caller(), "evt".to_string())
}

// This method is used to get all starred tasks
#[query]
pub fn get_starred_tasks() -> Vec<Principal> {
    Store::get_starred(caller(), "tsk".to_string())
}

// This method is used to get all starred groups
#[query]
pub fn get_starred_groups() -> Vec<Principal> {
    Store::get_starred(caller(), "grp".to_string())
}

#[update]
pub fn add_friend_request(
    principal: Principal,
    message: String,
) -> Result<FriendRequestResponse, ApiError> {
    Store::add_friend_request(caller(), principal, message)
}

#[update]
pub fn remove_friend(principal: Principal) -> Result<bool, String> {
    Store::remove_friend(caller(), principal)
}

#[update]
pub fn remove_friend_request(principal: Principal, id: u64) -> Result<bool, String> {
    Store::remove_friend_request(principal, id)
}

#[query]
pub fn get_friend_requests(principal: Principal) -> Vec<FriendRequestResponse> {
    Store::get_friend_requests(principal)
}

#[update]
pub fn decline_friend_request(principal: Principal, id: u64) -> Result<bool, String> {
    Store::decline_friend_request(principal, id)
}

#[update]
pub fn unblock_user(principal: Principal) -> Result<ProfileResponse, ApiError> {
    Store::unblock_user(caller(), principal)
}

#[update]
pub fn block_user(principal: Principal) -> Result<ProfileResponse, ApiError> {
    Store::block_user(caller(), principal)
}

// This method is used to get all relations of a specific type (Friend or Blocked)
#[query]
pub fn get_relations(relation_type: RelationType) -> Vec<Principal> {
    Store::get_relations(caller(), relation_type)
}

// This method is used to get relations count of a specific type (Friend or Blocked)
#[query]
pub fn get_relations_count(principal: Principal, relation_type: RelationType) -> u64 {
    Store::get_relations(principal, relation_type).len() as u64
}

// // This method is used to remove a relation from the profile
#[update]
pub fn remove_relation(identifier: Principal) -> Result<ProfileResponse, ApiError> {
    Store::remove_relation(caller(), identifier)
}

// This method is used to approve the code of conduct for the specific caller
#[update]
pub fn approve_code_of_conduct(version: u64) -> Result<bool, ApiError> {
    Store::approve_code_of_conduct(caller(), version)
}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get filtered profiles the (this) child canister
// Data serialized and send as byte array chunks ` (bytes, (start_chunk, end_chunk)) `
// The parent canister can then deserialize the data and pass it to the frontend
#[query]
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
