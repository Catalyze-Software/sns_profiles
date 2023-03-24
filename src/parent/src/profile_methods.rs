use candid::candid_method;
use ic_cdk::query;
use ic_scalable_misc::{
    enums::filter_type::FilterType, models::paged_response_models::PagedResponse,
};

use shared::profile_models::{ProfileFilter, ProfileResponse, ProfileSort};

use super::store::ScalableData;

#[query(composite = true)]
#[candid_method(query)]
async fn get_profiles(
    limit: usize,
    page: usize,
    filters: Vec<ProfileFilter>,
    filter_type: FilterType,
    sort: ProfileSort,
) -> PagedResponse<ProfileResponse> {
    ScalableData::get_child_canister_data(limit, page, filters, filter_type, sort).await
}
