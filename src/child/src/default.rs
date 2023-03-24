use candid::{candid_method, Principal};
use ic_cdk::caller;

#[allow(unused_imports)]
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use ic_scalable_canister::{
    ic_methods,
    store::{Data, Metadata},
};
use ic_scalable_misc::{
    enums::api_error_type::ApiError,
    models::http_models::{HttpRequest, HttpResponse},
};

use crate::store::DATA;

#[query]
#[candid_method(query)]
pub fn sanity_check() -> String {
    DATA.with(|data| Data::get_name(data))
}

#[query]
#[candid_method(query)]
pub fn get_metadata() -> Result<Metadata, ApiError> {
    DATA.with(|data| Data::get_metadata(data))
}

#[pre_upgrade]
pub fn pre_upgrade() {
    DATA.with(|data| ic_methods::pre_upgrade(data))
}

#[post_upgrade]
pub fn post_upgrade() {
    DATA.with(|data| ic_methods::post_upgrade(data))
}

#[update]
#[candid_method(update)]
async fn add_entry_by_parent(principal: Option<Principal>, entry: Vec<u8>) -> Result<(), ApiError> {
    DATA.with(|v| Data::add_entry_by_parent(v, caller(), entry, Some("pfe".to_string())))
}

#[update]
#[candid_method(update)]
fn accept_cycles() -> u64 {
    ic_methods::accept_cycles()
}

#[query]
#[candid_method(query)]
fn http_request(req: HttpRequest) -> HttpResponse {
    DATA.with(|data| Data::http_request_with_metrics(data, req, vec![]))
}

#[init]
#[candid_method(init)]
pub fn init(owner: Principal, parent: Principal, name: String, identifier: usize) {
    DATA.with(|data| {
        ic_methods::init(&data, parent, name, identifier);
    });
}

#[test]
pub fn candid() {
    use candid::export_service;
    use candid::Principal;
    use ic_cdk::api::management_canister::http_request::HttpResponse;
    use ic_scalable_canister::store::Metadata;
    use ic_scalable_misc::enums::api_error_type::ApiError;
    use ic_scalable_misc::enums::application_role_type::ApplicationRole;
    use ic_scalable_misc::enums::filter_type::FilterType;
    use ic_scalable_misc::helpers::candid_helper::save_candid;
    use ic_scalable_misc::models::http_models::HttpRequest;
    use shared::profile_models::*;
    export_service!();

    save_candid(__export_service(), String::from("child"));
}
