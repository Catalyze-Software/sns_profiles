export const idlFactory = ({ IDL }) => {
  const Result = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : IDL.Text });
  const ErrorMessage = IDL.Record({
    'tag' : IDL.Text,
    'message' : IDL.Text,
    'inputs' : IDL.Opt(IDL.Vec(IDL.Text)),
    'location' : IDL.Text,
  });
  const ValidationResponse = IDL.Record({
    'field' : IDL.Text,
    'message' : IDL.Text,
  });
  const UpdateMessage = IDL.Record({
    'canister_principal' : IDL.Principal,
    'message' : IDL.Text,
  });
  const ApiError = IDL.Variant({
    'SerializeError' : ErrorMessage,
    'DeserializeError' : ErrorMessage,
    'NotFound' : ErrorMessage,
    'ValidationError' : IDL.Vec(ValidationResponse),
    'CanisterAtCapacity' : ErrorMessage,
    'UpdateRequired' : UpdateMessage,
    'Unauthorized' : ErrorMessage,
    'Unexpected' : ErrorMessage,
    'BadRequest' : ErrorMessage,
  });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : ApiError });
  const FriendRequestResponse = IDL.Record({
    'id' : IDL.Nat64,
    'to' : IDL.Principal,
    'created_at' : IDL.Nat64,
    'requested_by' : IDL.Principal,
    'message' : IDL.Text,
  });
  const Result_2 = IDL.Variant({
    'Ok' : FriendRequestResponse,
    'Err' : ApiError,
  });
  const ProfilePrivacy = IDL.Variant({
    'Private' : IDL.Null,
    'Public' : IDL.Null,
  });
  const PostProfile = IDL.Record({
    'username' : IDL.Text,
    'display_name' : IDL.Text,
    'extra' : IDL.Text,
    'privacy' : ProfilePrivacy,
    'first_name' : IDL.Text,
    'last_name' : IDL.Text,
  });
  const ChunkData = IDL.Record({
    'chunk_id' : IDL.Nat64,
    'canister' : IDL.Principal,
    'index' : IDL.Nat64,
  });
  const Manifest = IDL.Record({ 'entries' : IDL.Vec(ChunkData) });
  const CanisterStorage = IDL.Variant({
    'None' : IDL.Null,
    'Manifest' : Manifest,
    'Chunk' : ChunkData,
  });
  const Asset = IDL.Variant({
    'Url' : IDL.Text,
    'None' : IDL.Null,
    'CanisterStorage' : CanisterStorage,
  });
  const WalletResponse = IDL.Record({
    'principal' : IDL.Principal,
    'provider' : IDL.Text,
    'is_primary' : IDL.Bool,
  });
  const CodeOfConductDetails = IDL.Record({
    'approved_date' : IDL.Nat64,
    'approved_version' : IDL.Nat64,
  });
  const ApplicationRole = IDL.Variant({
    'Blocked' : IDL.Null,
    'Guest' : IDL.Null,
    'Member' : IDL.Null,
    'Banned' : IDL.Null,
    'Admin' : IDL.Null,
    'Moderator' : IDL.Null,
    'Leader' : IDL.Null,
    'Owner' : IDL.Null,
    'Watcher' : IDL.Null,
  });
  const ProfileResponse = IDL.Record({
    'updated_on' : IDL.Nat64,
    'profile_image' : Asset,
    'principal' : IDL.Principal,
    'banner_image' : Asset,
    'about' : IDL.Text,
    'country' : IDL.Text,
    'username' : IDL.Text,
    'interests' : IDL.Vec(IDL.Nat32),
    'city' : IDL.Text,
    'created_on' : IDL.Nat64,
    'email' : IDL.Text,
    'website' : IDL.Text,
    'display_name' : IDL.Text,
    'extra' : IDL.Text,
    'privacy' : ProfilePrivacy,
    'wallets' : IDL.Vec(WalletResponse),
    'state_or_province' : IDL.Text,
    'first_name' : IDL.Text,
    'last_name' : IDL.Text,
    'member_identifier' : IDL.Principal,
    'causes' : IDL.Vec(IDL.Nat32),
    'code_of_conduct' : CodeOfConductDetails,
    'date_of_birth' : IDL.Nat64,
    'identifier' : IDL.Principal,
    'skills' : IDL.Vec(IDL.Nat32),
    'application_role' : ApplicationRole,
  });
  const Result_3 = IDL.Variant({ 'Ok' : ProfileResponse, 'Err' : ApiError });
  const PostWallet = IDL.Record({
    'principal' : IDL.Principal,
    'provider' : IDL.Text,
  });
  const Result_4 = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : ApiError });
  const CanisterStatusType = IDL.Variant({
    'stopped' : IDL.Null,
    'stopping' : IDL.Null,
    'running' : IDL.Null,
  });
  const DefiniteCanisterSettings = IDL.Record({
    'freezing_threshold' : IDL.Nat,
    'controllers' : IDL.Vec(IDL.Principal),
    'memory_allocation' : IDL.Nat,
    'compute_allocation' : IDL.Nat,
  });
  const CanisterStatusResponse = IDL.Record({
    'status' : CanisterStatusType,
    'memory_size' : IDL.Nat,
    'cycles' : IDL.Nat,
    'settings' : DefiniteCanisterSettings,
    'idle_cycles_burned_per_day' : IDL.Nat,
    'module_hash' : IDL.Opt(IDL.Vec(IDL.Nat8)),
  });
  const RejectionCode = IDL.Variant({
    'NoError' : IDL.Null,
    'CanisterError' : IDL.Null,
    'SysTransient' : IDL.Null,
    'DestinationInvalid' : IDL.Null,
    'Unknown' : IDL.Null,
    'SysFatal' : IDL.Null,
    'CanisterReject' : IDL.Null,
  });
  const Result_5 = IDL.Variant({
    'Ok' : IDL.Tuple(CanisterStatusResponse),
    'Err' : IDL.Tuple(RejectionCode, IDL.Text),
  });
  const UpdateProfile = IDL.Record({
    'profile_image' : Asset,
    'banner_image' : Asset,
    'about' : IDL.Text,
    'country' : IDL.Text,
    'interests' : IDL.Vec(IDL.Nat32),
    'city' : IDL.Text,
    'email' : IDL.Opt(IDL.Text),
    'website' : IDL.Text,
    'display_name' : IDL.Text,
    'extra' : IDL.Text,
    'privacy' : ProfilePrivacy,
    'state_or_province' : IDL.Text,
    'first_name' : IDL.Text,
    'last_name' : IDL.Text,
    'causes' : IDL.Vec(IDL.Nat32),
    'date_of_birth' : IDL.Nat64,
    'skills' : IDL.Vec(IDL.Nat32),
  });
  const DateRange = IDL.Record({
    'end_date' : IDL.Nat64,
    'start_date' : IDL.Nat64,
  });
  const ProfileFilter = IDL.Variant({
    'Interest' : IDL.Nat32,
    'Email' : IDL.Text,
    'Skill' : IDL.Nat32,
    'DisplayName' : IDL.Text,
    'UpdatedOn' : DateRange,
    'City' : IDL.Text,
    'FirstName' : IDL.Text,
    'LastName' : IDL.Text,
    'Cause' : IDL.Nat32,
    'StateOrProvince' : IDL.Text,
    'Country' : IDL.Text,
    'CreatedOn' : DateRange,
    'Username' : IDL.Text,
  });
  const RelationType = IDL.Variant({
    'Blocked' : IDL.Null,
    'Friend' : IDL.Null,
  });
  const HttpRequest = IDL.Record({
    'url' : IDL.Text,
    'method' : IDL.Text,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const HttpHeader = IDL.Record({ 'value' : IDL.Text, 'name' : IDL.Text });
  const HttpResponse = IDL.Record({
    'status' : IDL.Nat,
    'body' : IDL.Vec(IDL.Nat8),
    'headers' : IDL.Vec(HttpHeader),
  });
  const Result_6 = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Null });
  return IDL.Service({
    '__get_candid_interface_tmp_hack' : IDL.Func([], [IDL.Text], ['query']),
    'accept_cycles' : IDL.Func([], [IDL.Nat64], []),
    'accept_friend_request' : IDL.Func([IDL.Nat64], [Result], []),
    'add_entry_by_parent' : IDL.Func([IDL.Vec(IDL.Nat8)], [Result_1], []),
    'add_friend_request' : IDL.Func([IDL.Principal, IDL.Text], [Result_2], []),
    'add_profile' : IDL.Func([PostProfile, IDL.Principal], [Result_3], []),
    'add_starred' : IDL.Func([IDL.Principal], [Result_3], []),
    'add_wallet' : IDL.Func([PostWallet], [Result_3], []),
    'approve_code_of_conduct' : IDL.Func([IDL.Nat64], [Result_4], []),
    'block_user' : IDL.Func([IDL.Principal], [Result_3], []),
    'canister_backup_data' : IDL.Func([], [IDL.Text, IDL.Text], []),
    'canister_status' : IDL.Func([], [Result_5], []),
    'clear_backup' : IDL.Func([], [], []),
    'clear_relations' : IDL.Func([IDL.Text], [IDL.Bool], []),
    'decline_friend_request' : IDL.Func([IDL.Nat64], [Result], []),
    'download_chunk' : IDL.Func(
        [IDL.Nat64],
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'download_entries_chunk' : IDL.Func(
        [IDL.Nat64],
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'download_stable_data_chunk' : IDL.Func(
        [IDL.Nat64],
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        ['query'],
      ),
    'edit_profile' : IDL.Func([UpdateProfile], [Result_3], []),
    'finalize_upload' : IDL.Func([], [IDL.Text], []),
    'get_chunked_data' : IDL.Func(
        [IDL.Vec(ProfileFilter), IDL.Nat64, IDL.Nat64],
        [IDL.Vec(IDL.Nat8), IDL.Tuple(IDL.Nat64, IDL.Nat64)],
        ['query'],
      ),
    'get_friend_requests' : IDL.Func(
        [],
        [IDL.Vec(FriendRequestResponse)],
        ['query'],
      ),
    'get_profile_by_identifier' : IDL.Func(
        [IDL.Principal],
        [Result_3],
        ['query'],
      ),
    'get_profile_by_user_principal' : IDL.Func(
        [IDL.Principal],
        [Result_3],
        ['query'],
      ),
    'get_profiles_by_identifier' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [IDL.Vec(ProfileResponse)],
        ['query'],
      ),
    'get_profiles_by_user_principal' : IDL.Func(
        [IDL.Vec(IDL.Principal)],
        [IDL.Vec(ProfileResponse)],
        ['query'],
      ),
    'get_relations' : IDL.Func(
        [RelationType],
        [IDL.Vec(IDL.Principal)],
        ['query'],
      ),
    'get_relations_count' : IDL.Func(
        [IDL.Principal, RelationType],
        [IDL.Nat64],
        ['query'],
      ),
    'get_starred_events' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'get_starred_groups' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'get_starred_tasks' : IDL.Func([], [IDL.Vec(IDL.Principal)], ['query']),
    'http_request' : IDL.Func([HttpRequest], [HttpResponse], ['query']),
    'remove_friend' : IDL.Func([IDL.Principal], [Result], []),
    'remove_friend_request' : IDL.Func(
        [IDL.Principal, IDL.Nat64],
        [Result],
        [],
      ),
    'remove_starred' : IDL.Func([IDL.Principal], [Result_3], []),
    'remove_wallet' : IDL.Func([IDL.Principal], [Result_3], []),
    'restore_data' : IDL.Func([], [], []),
    'set_wallet_as_primary' : IDL.Func([IDL.Principal], [Result_6], []),
    'total_chunks' : IDL.Func([], [IDL.Nat64], ['query']),
    'total_entries_chunks' : IDL.Func([], [IDL.Nat64], ['query']),
    'total_stable_data_chunks' : IDL.Func([], [IDL.Nat64], ['query']),
    'unblock_user' : IDL.Func([IDL.Principal], [Result_3], []),
    'upload_chunk' : IDL.Func(
        [IDL.Tuple(IDL.Nat64, IDL.Vec(IDL.Nat8))],
        [],
        [],
      ),
  });
};
export const init = ({ IDL }) => {
  return [IDL.Principal, IDL.Text, IDL.Nat64];
};
