use super::backup::{data_hash, Chunk, ChunkId};
use super::principals::is_owner;
use crate::backup::backup::BACKUP;
use candid::candid_method;
use ic_cdk_macros::{query, update};

#[update(guard = "is_owner")]
#[candid_method(update)]
fn backup_data() -> String {
    BACKUP.with(|b| b.borrow_mut().backup_data())
}

#[update(guard = "is_owner")]
#[candid_method(update)]
fn restore_data() -> String {
    BACKUP.with(|b| b.borrow_mut().restore_data())
}

#[query(guard = "is_owner")]
#[candid_method(query)]
fn download_chunk(n: ChunkId) -> Chunk {
    BACKUP.with(|b| b.borrow().download_chunk(n))
}

#[update(guard = "is_owner")]
#[candid_method(update)]
fn upload_chunk(chunk: Chunk) {
    BACKUP.with(|b| b.borrow_mut().upload_chunk(chunk))
}

#[update(guard = "is_owner")]
#[candid_method(update)]
fn finalize_upload() -> String {
    BACKUP.with(|b| b.borrow_mut().finalize_upload())
}

#[query(guard = "is_owner")]
#[candid_method(query)]
fn total_chunks() -> u64 {
    BACKUP.with(|b| b.borrow().total_chunks() as u64)
}

#[update(guard = "is_owner")]
#[candid_method(update)]
fn clear_backup() {
    BACKUP.with(|b| b.borrow_mut().clear_backup());
}

#[query(guard = "is_owner")]
#[candid_method(query)]
fn hash() -> String {
    data_hash().iter().map(|b| format!("{:02x}", b)).collect()
}
