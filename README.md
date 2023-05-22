# Profile canister

This repository is responsible for handling profiles of the Catalyze application. Profiles hold the users data like username, skills, intererests, wallets, etc. To get a full view of the data stored in the profile, check out the `shared/src/profile_models.rs` file

## setup

The parent canister is SNS controlled, the child canisters are controlled by their parent. Upgrading the child canister is done through the parent canister as the (gzipped) child wasm is included in the parent canister.

When the parent canister is upgraded it checks if the child wasm has changed (currently it generates a new wasm hash every time you run the script). if changed it upgrades the child canisters automatically.

## Project structure

**|- candid**
Contains the candid files for the `parent` and `child` canister.

**|- frontend**
Contains all declarations that are needed for the frontend

**|- scripts**
Contains a single script that generates the following files for the parent and child canisters;

- candid files
- frontend declarations
- wasms (gzipped and regular)

**|- src/child**
Contains codebase related to the child canisters
**|- src/parent**
Contains codebase related to the child canisters
**|- src/shared**
Contains data used by both codebases

**|- wasm**
Contains

- child wasm
- child wasm (gzipped)
- parent wasm
- parent wasm (gzipped)

## Parent canister

The parent canister manages all underlying child canisters.

#### This canister is responsible for;

- keeping track of all profile child canisters
- spinning up a new child canisters
- composite query call to the children (preperation)

#### methods

Described methods can be found below, for more details you can check out the code which is inline commented

###### DEFAULT

```
// Stores the data in stable storage before upgrading the canister.
pub fn pre_upgrade() {}

// Restores the data from stable- to heap storage after upgrading the canister.
pub fn post_upgrade() {}

// Init methods thats get triggered when the canister is installed
pub fn init() {}
```

##

###### QUERY CALLS

```
// Method to retrieve an available canister to write updates to
fn get_available_canister() -> Result<ScalableCanisterDetails, String> {}

// Method to retrieve all the canisters
fn get_canisters() -> Vec<ScalableCanisterDetails> {}

// Method to retrieve the latest wasm version of the child canister that is currently stored
fn get_latest_wasm_version() -> WasmVersion {}

// HTTP request handler (canister metrics are added to the response)
fn http_request(req: HttpRequest) -> HttpResponse {}

// Method used to get all the profiles from the child canisters filtered, sorted and paged
// requires composite queries to be released to mainnet
async fn get_profiles(
    limit: usize,
    page: usize,
    filters: Vec<ProfileFilter>,
    filter_type: FilterType,
    sort: ProfileSort,
) -> PagedResponse<ProfileResponse> {}

```

##

###### UPDATE CALLS

```
// Method called by child canister once full (inter-canister call)
// can only be called by a child canister
async fn close_child_canister_and_spawn_sibling(
    last_entry_id: u64,
    entry: Vec<u8>
    ) -> Result<Principal, ApiError> {}

// Method to accept cycles when send to this canister
fn accept_cycles() -> u64 {}
```

## Child canister

The child canister is where the data is stored that the app uses.

This canister is responsible for;

- storing data records
- data validation
- messaging the parent to spin up a new sibling

#### methods

Described methods can be found below, for more details you can check out the code which is inline commented

###### DEFAULT

```
// Stores the data in stable storage before upgrading the canister.
pub fn pre_upgrade() {}

// Restores the data from stable- to heap storage after upgrading the canister.
pub fn post_upgrade() {}

// Init methods thats get triggered when the canister is installed
pub fn init(parent: Principal, name: String, identifier: usize) {}
```

##

###### QUERY CALLS

```
// This method is used to add a profile to the canister,
pub async fn add_profile(
    post_profile: PostProfile,
    member_canister: Principal,
) -> Result<ProfileResponse, ApiError> {}

// This method is used to get a single profile by an user principal
pub fn get_profile_by_user_principal(principal: Principal) -> Result<ProfileResponse, ApiError> {}

// This method is used to get a single profile by an identifier
pub fn get_profile_by_identifier(id: Principal) -> Result<ProfileResponse, ApiError> {}

// This method is used to get multiple profiles by principals
pub fn get_profiles_by_user_principal(principals: Vec<Principal>) -> Vec<ProfileResponse> {}

// This method is used to get multiple profiles by identifiers
pub fn get_profiles_by_identifier(identifiers: Vec<Principal>) -> Vec<ProfileResponse> {}

// This method is used to get all starred events
pub fn get_starred_events() -> Vec<Principal> {}

// This method is used to get all starred tasks
pub fn get_starred_tasks() -> Vec<Principal> {}

// This method is used to get all starred groups
pub fn get_starred_groups() -> Vec<Principal> {}

// This method is used to get all relations of a specific type
pub fn get_relations(relation_type: RelationType) -> Vec<Principal> {}

// COMPOSITE_QUERY PREPARATION
// This methods is used by the parent canister to get filtered profiles the (this) child canister
fn get_chunked_data(
    filters: Vec<ProfileFilter>,
    chunk: usize,
    max_bytes_per_chunk: usize,
) -> (Vec<u8>, (usize, usize)) {}

```

###

###### UPDATE CALLS

```
// This method is used to edit a profile
pub fn edit_profile(update_profile: UpdateProfile) -> Result<ProfileResponse, ApiError> {}

// This method is used to add a wallet reference to the profile
pub fn add_wallet(wallet: PostWallet) -> Result<ProfileResponse, ApiError> {}

// This method is used to set a wallet as primary
pub fn set_wallet_as_primary(wallet_principal: Principal) -> Result<(), ()> {}

// This method is used to remove a wallet reference from the profile
pub fn remove_wallet(wallet: Principal) -> Result<ProfileResponse, ApiError> {}

// This method is used to add a starred reference to the profile, for example a starred event, group or task
pub fn add_starred(identifier: Principal) -> Result<ProfileResponse, ApiError> {}

// This method is used to remove a starred reference from the profile
pub fn remove_starred(identifier: Principal) -> Result<ProfileResponse, ApiError> {}

// This method adds a relation to the profile
pub fn add_relation(
    identifier: Principal,
    relation_type: RelationType,
) -> Result<ProfileResponse, ApiError> {}

// This method is used to remove a relation from the profile
pub fn remove_relation(identifier: Principal) -> Result<ProfileResponse, ApiError> {}

// This method is used to approve the code of conduct for the specific caller
pub fn approve_code_of_conduct(version: u64) -> Result<bool, ApiError> {}
```

## SNS controlled

// TBD

## Testing

// TBD
