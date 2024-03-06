use crate::store::{ENTRIES, FRIEND_REQUEST};
use shared::profile_models::{FriendRequest, Profile};

#[ic_cdk::query(guard = "auth")]
fn read_profile_entries() -> Vec<(String, Profile)> {
    ENTRIES.with(|entries| entries.borrow().iter().collect::<Vec<(String, Profile)>>())
}

#[ic_cdk::query(guard = "auth")]
fn read_profile_friend_requests() -> Vec<(u64, FriendRequest)> {
    FRIEND_REQUEST.with(|requests| {
        requests
            .borrow()
            .iter()
            .collect::<Vec<(u64, FriendRequest)>>()
    })
}

// GUARDS
const ALLOWED: [&str; 2] = [
    // sam candid ui
    "nvifv-62idm-izjcy-rvy63-7tqjz-mg2d7-jiw6m-soqvp-hdayh-mnqf5-yqe",
    // catalyze development
    "syzio-xu6ca-burmx-4afo2-ojpcw-e75j3-m67o5-s5bes-5vvsv-du3t4-wae",
];

fn auth() -> Result<(), String> {
    if ALLOWED.contains(&ic_cdk::caller().to_string().as_str()) {
        Ok(())
    } else {
        Err("Unauthorized".to_string())
    }
}
