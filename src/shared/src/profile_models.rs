use core::fmt;
use std::{borrow::Cow, collections::HashMap};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_scalable_misc::{
    enums::{application_role_type::ApplicationRole, asset_type::Asset, sort_type::SortDirection},
    models::date_models::DateRange,
    traits::stable_storage_trait::StableStorableTrait,
};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Serialize;

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct Profile {
    pub principal: Principal,
    pub member_identifier: Principal,
    pub username: String,
    pub display_name: String,
    pub application_role: ApplicationRole,
    pub first_name: String,
    pub last_name: String,
    pub privacy: ProfilePrivacy,
    pub about: String,
    pub email: String,
    pub date_of_birth: u64,
    pub city: String,
    pub state_or_province: String,
    pub country: String,
    pub profile_image: Asset,
    pub banner_image: Asset,
    pub skills: Vec<u32>,
    pub interests: Vec<u32>,
    pub causes: Vec<u32>,
    pub website: String,
    pub code_of_conduct: DocumentDetails,
    pub privacy_policy: Option<DocumentDetails>,
    pub terms_of_service: Option<DocumentDetails>,
    pub wallets: HashMap<Principal, Wallet>,
    pub starred: HashMap<Principal, String>,
    pub relations: HashMap<Principal, String>,
    pub extra: String,
    pub updated_on: u64,
    pub created_on: u64,
}

impl StableStorableTrait for Profile {}

impl Storable for Profile {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            principal: Principal::anonymous(),
            member_identifier: Principal::anonymous(),
            username: Default::default(),
            display_name: Default::default(),
            application_role: Default::default(),
            first_name: Default::default(),
            last_name: Default::default(),
            privacy: Default::default(),
            about: Default::default(),
            email: Default::default(),
            date_of_birth: Default::default(),
            city: Default::default(),
            state_or_province: Default::default(),
            country: Default::default(),
            profile_image: Default::default(),
            banner_image: Default::default(),
            skills: Default::default(),
            interests: Default::default(),
            causes: Default::default(),
            website: Default::default(),
            code_of_conduct: Default::default(),
            wallets: Default::default(),
            starred: Default::default(),
            relations: Default::default(),
            extra: Default::default(),
            updated_on: Default::default(),
            created_on: Default::default(),
            privacy_policy: None,
            terms_of_service: None,
        }
    }
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct PostProfile {
    pub username: String,
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub privacy: ProfilePrivacy,
    pub extra: String,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
pub struct UpdateProfile {
    pub display_name: String,
    pub first_name: String,
    pub last_name: String,
    pub privacy: ProfilePrivacy,
    pub about: String,
    pub email: Option<String>,
    pub date_of_birth: u64,
    pub city: String,
    pub state_or_province: String,
    pub country: String,
    pub profile_image: Asset,
    pub banner_image: Asset,
    pub skills: Vec<u32>,
    pub interests: Vec<u32>,
    pub causes: Vec<u32>,
    pub website: String,
    pub extra: String,
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub struct ProfileResponse {
    pub identifier: Principal,
    pub principal: Principal,
    pub member_identifier: Principal,
    pub username: String,
    pub display_name: String,
    pub application_role: ApplicationRole,
    pub first_name: String,
    pub last_name: String,
    pub privacy: ProfilePrivacy,
    pub about: String,
    pub email: String,
    pub date_of_birth: u64,
    pub city: String,
    pub state_or_province: String,
    pub country: String,
    pub profile_image: Asset,
    pub banner_image: Asset,
    pub skills: Vec<u32>,
    pub interests: Vec<u32>,
    pub causes: Vec<u32>,
    pub website: String,
    pub code_of_conduct: DocumentDetails,
    pub privacy_policy: Option<DocumentDetails>,
    pub terms_of_service: Option<DocumentDetails>,
    pub wallets: Vec<WalletResponse>,
    pub extra: String,
    pub updated_on: u64,
    pub created_on: u64,
}

#[derive(Clone, Debug, Default, Serialize, CandidType, Deserialize)]
pub struct DocumentDetails {
    pub approved_version: u64,
    pub approved_date: u64,
}

#[derive(Clone, Debug, Serialize, CandidType, Deserialize)]
pub struct PostWallet {
    pub provider: String,
    pub principal: Principal,
}

#[derive(Clone, Debug, Serialize, CandidType, Deserialize)]
pub struct Wallet {
    pub provider: String,
    pub is_primary: bool,
}

#[derive(Clone, Debug, Serialize, CandidType, Deserialize)]
pub struct WalletResponse {
    pub provider: String,
    pub principal: Principal,
    pub is_primary: bool,
}

impl Default for Wallet {
    fn default() -> Self {
        Self {
            provider: Default::default(),
            is_primary: Default::default(),
        }
    }
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProfilePrivacy {
    Public,
    Private,
}

#[derive(CandidType, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationType {
    Friend,
    Blocked,
}

impl fmt::Display for RelationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RelationType::*;
        match self {
            Friend => write!(f, "friend"),
            Blocked => write!(f, "blocked"),
        }
    }
}

impl Default for ProfilePrivacy {
    fn default() -> Self {
        ProfilePrivacy::Private
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum ProfileSort {
    Username(SortDirection),
    DisplayName(SortDirection),
    FirstName(SortDirection),
    LastName(SortDirection),
    Email(SortDirection),
    City(SortDirection),
    StateOrProvince(SortDirection),
    Country(SortDirection),
    CreatedOn(SortDirection),
    UpdatedOn(SortDirection),
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum ProfileFilter {
    Username(String),
    DisplayName(String),
    FirstName(String),
    LastName(String),
    Email(String),
    City(String),
    StateOrProvince(String),
    Country(String),
    UpdatedOn(DateRange),
    Skill(u32),
    Interest(u32),
    Cause(u32),
    CreatedOn(DateRange),
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FriendRequest {
    pub requested_by: Principal,
    pub message: String,
    pub to: Principal,
    pub created_at: u64,
}

impl Storable for FriendRequest {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct FriendRequestResponse {
    pub id: u64,
    pub requested_by: Principal,
    pub message: String,
    pub to: Principal,
    pub created_at: u64,
}
