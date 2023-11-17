use super::backup::{Chunk, ChunkId};
use super::principals::is_owner;
use crate::backup::backup::BACKUP;
use candid::candid_method;
use ic_cdk_macros::{query, update};

#[update(guard = "is_owner")]
#[candid_method(update)]
pub fn backup_data() -> String {
    BACKUP.with(|b| b.borrow_mut().backup_data())
}

#[update(guard = "is_owner")]
#[candid_method(update)]
pub fn restore_data() -> String {
    BACKUP.with(|b| b.borrow_mut().restore_data())
}

#[query(guard = "is_owner")]
#[candid_method(query)]
pub fn download_chunk(n: ChunkId) -> Chunk {
    BACKUP.with(|b| b.borrow().download_chunk(n))
}

#[update(guard = "is_owner")]
#[candid_method(update)]
pub fn upload_chunk(chunk: Chunk) {
    BACKUP.with(|b| b.borrow_mut().upload_chunk(chunk))
}

#[update(guard = "is_owner")]
#[candid_method(update)]
pub fn finalize_upload() -> String {
    BACKUP.with(|b| b.borrow_mut().finalize_upload())
}

#[query(guard = "is_owner")]
#[candid_method(query)]
pub fn total_chunks() -> u64 {
    BACKUP.with(|b| b.borrow().total_chunks() as u64)
}
