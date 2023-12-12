import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type ApiError = { 'SerializeError' : ErrorMessage } |
  { 'DeserializeError' : ErrorMessage } |
  { 'NotFound' : ErrorMessage } |
  { 'ValidationError' : Array<ValidationResponse> } |
  { 'CanisterAtCapacity' : ErrorMessage } |
  { 'UpdateRequired' : UpdateMessage } |
  { 'Unauthorized' : ErrorMessage } |
  { 'Unexpected' : ErrorMessage } |
  { 'BadRequest' : ErrorMessage };
export type ApplicationRole = { 'Blocked' : null } |
  { 'Guest' : null } |
  { 'Member' : null } |
  { 'Banned' : null } |
  { 'Admin' : null } |
  { 'Moderator' : null } |
  { 'Leader' : null } |
  { 'Owner' : null } |
  { 'Watcher' : null };
export type Asset = { 'Url' : string } |
  { 'None' : null } |
  { 'CanisterStorage' : CanisterStorage };
export interface CanisterStatusResponse {
  'status' : CanisterStatusType,
  'memory_size' : bigint,
  'cycles' : bigint,
  'settings' : DefiniteCanisterSettings,
  'idle_cycles_burned_per_day' : bigint,
  'module_hash' : [] | [Uint8Array | number[]],
}
export type CanisterStatusType = { 'stopped' : null } |
  { 'stopping' : null } |
  { 'running' : null };
export type CanisterStorage = { 'None' : null } |
  { 'Manifest' : Manifest } |
  { 'Chunk' : ChunkData };
export interface ChunkData {
  'chunk_id' : bigint,
  'canister' : Principal,
  'index' : bigint,
}
export interface CodeOfConductDetails {
  'approved_date' : bigint,
  'approved_version' : bigint,
}
export interface DateRange { 'end_date' : bigint, 'start_date' : bigint }
export interface DefiniteCanisterSettings {
  'freezing_threshold' : bigint,
  'controllers' : Array<Principal>,
  'memory_allocation' : bigint,
  'compute_allocation' : bigint,
}
export interface ErrorMessage {
  'tag' : string,
  'message' : string,
  'inputs' : [] | [Array<string>],
  'location' : string,
}
export interface FriendRequestResponse {
  'id' : bigint,
  'to' : Principal,
  'created_at' : bigint,
  'requested_by' : Principal,
  'message' : string,
}
export interface HttpHeader { 'value' : string, 'name' : string }
export interface HttpRequest {
  'url' : string,
  'method' : string,
  'body' : Uint8Array | number[],
  'headers' : Array<[string, string]>,
}
export interface HttpResponse {
  'status' : bigint,
  'body' : Uint8Array | number[],
  'headers' : Array<HttpHeader>,
}
export interface Manifest { 'entries' : Array<ChunkData> }
export interface PostProfile {
  'username' : string,
  'display_name' : string,
  'extra' : string,
  'privacy' : ProfilePrivacy,
  'first_name' : string,
  'last_name' : string,
}
export interface PostWallet { 'principal' : Principal, 'provider' : string }
export type ProfileFilter = { 'Interest' : number } |
  { 'Email' : string } |
  { 'Skill' : number } |
  { 'DisplayName' : string } |
  { 'UpdatedOn' : DateRange } |
  { 'City' : string } |
  { 'FirstName' : string } |
  { 'LastName' : string } |
  { 'Cause' : number } |
  { 'StateOrProvince' : string } |
  { 'Country' : string } |
  { 'CreatedOn' : DateRange } |
  { 'Username' : string };
export type ProfilePrivacy = { 'Private' : null } |
  { 'Public' : null };
export interface ProfileResponse {
  'updated_on' : bigint,
  'profile_image' : Asset,
  'principal' : Principal,
  'banner_image' : Asset,
  'about' : string,
  'country' : string,
  'username' : string,
  'interests' : Uint32Array | number[],
  'city' : string,
  'created_on' : bigint,
  'email' : string,
  'website' : string,
  'display_name' : string,
  'extra' : string,
  'privacy' : ProfilePrivacy,
  'wallets' : Array<WalletResponse>,
  'state_or_province' : string,
  'first_name' : string,
  'last_name' : string,
  'member_identifier' : Principal,
  'causes' : Uint32Array | number[],
  'code_of_conduct' : CodeOfConductDetails,
  'date_of_birth' : bigint,
  'identifier' : Principal,
  'skills' : Uint32Array | number[],
  'application_role' : ApplicationRole,
}
export type RejectionCode = { 'NoError' : null } |
  { 'CanisterError' : null } |
  { 'SysTransient' : null } |
  { 'DestinationInvalid' : null } |
  { 'Unknown' : null } |
  { 'SysFatal' : null } |
  { 'CanisterReject' : null };
export type RelationType = { 'Blocked' : null } |
  { 'Friend' : null };
export type Result = { 'Ok' : boolean } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : null } |
  { 'Err' : ApiError };
export type Result_2 = { 'Ok' : FriendRequestResponse } |
  { 'Err' : ApiError };
export type Result_3 = { 'Ok' : ProfileResponse } |
  { 'Err' : ApiError };
export type Result_4 = { 'Ok' : boolean } |
  { 'Err' : ApiError };
export type Result_5 = { 'Ok' : [CanisterStatusResponse] } |
  { 'Err' : [RejectionCode, string] };
export type Result_6 = { 'Ok' : null } |
  { 'Err' : null };
export interface UpdateMessage {
  'canister_principal' : Principal,
  'message' : string,
}
export interface UpdateProfile {
  'profile_image' : Asset,
  'banner_image' : Asset,
  'about' : string,
  'country' : string,
  'interests' : Uint32Array | number[],
  'city' : string,
  'email' : [] | [string],
  'website' : string,
  'display_name' : string,
  'extra' : string,
  'privacy' : ProfilePrivacy,
  'state_or_province' : string,
  'first_name' : string,
  'last_name' : string,
  'causes' : Uint32Array | number[],
  'date_of_birth' : bigint,
  'skills' : Uint32Array | number[],
}
export interface ValidationResponse { 'field' : string, 'message' : string }
export interface WalletResponse {
  'principal' : Principal,
  'provider' : string,
  'is_primary' : boolean,
}
export interface _SERVICE {
  '__get_candid_interface_tmp_hack' : ActorMethod<[], string>,
  'accept_cycles' : ActorMethod<[], bigint>,
  'accept_friend_request' : ActorMethod<[bigint], Result>,
  'add_entry_by_parent' : ActorMethod<[Uint8Array | number[]], Result_1>,
  'add_friend_request' : ActorMethod<[Principal, string], Result_2>,
  'add_profile' : ActorMethod<[PostProfile, Principal], Result_3>,
  'add_starred' : ActorMethod<[Principal], Result_3>,
  'add_wallet' : ActorMethod<[PostWallet], Result_3>,
  'approve_code_of_conduct' : ActorMethod<[bigint], Result_4>,
  'block_user' : ActorMethod<[Principal], Result_3>,
  'canister_status' : ActorMethod<[], Result_5>,
  'clear_backup' : ActorMethod<[], undefined>,
  'clear_relations' : ActorMethod<[string], boolean>,
  'decline_friend_request' : ActorMethod<[bigint], Result>,
  'download_chunk' : ActorMethod<[bigint], [bigint, Uint8Array | number[]]>,
  'edit_profile' : ActorMethod<[UpdateProfile], Result_3>,
  'finalize_upload' : ActorMethod<[], string>,
  'get_chunked_data' : ActorMethod<
    [Array<ProfileFilter>, bigint, bigint],
    [Uint8Array | number[], [bigint, bigint]]
  >,
  'get_friend_requests' : ActorMethod<[], Array<FriendRequestResponse>>,
  'get_profile_by_identifier' : ActorMethod<[Principal], Result_3>,
  'get_profile_by_user_principal' : ActorMethod<[Principal], Result_3>,
  'get_profiles_by_identifier' : ActorMethod<
    [Array<Principal>],
    Array<ProfileResponse>
  >,
  'get_profiles_by_user_principal' : ActorMethod<
    [Array<Principal>],
    Array<ProfileResponse>
  >,
  'get_relations' : ActorMethod<[RelationType], Array<Principal>>,
  'get_relations_count' : ActorMethod<[Principal, RelationType], bigint>,
  'get_starred_events' : ActorMethod<[], Array<Principal>>,
  'get_starred_groups' : ActorMethod<[], Array<Principal>>,
  'get_starred_tasks' : ActorMethod<[], Array<Principal>>,
  'http_request' : ActorMethod<[HttpRequest], HttpResponse>,
  'remove_friend' : ActorMethod<[Principal], Result>,
  'remove_friend_request' : ActorMethod<[Principal, bigint], Result>,
  'remove_starred' : ActorMethod<[Principal], Result_3>,
  'remove_wallet' : ActorMethod<[Principal], Result_3>,
  'restore_data' : ActorMethod<[], undefined>,
  'set_wallet_as_primary' : ActorMethod<[Principal], Result_6>,
  'total_chunks' : ActorMethod<[], bigint>,
  'unblock_user' : ActorMethod<[Principal], Result_3>,
  'upload_chunk' : ActorMethod<[[bigint, Uint8Array | number[]]], undefined>,
}
