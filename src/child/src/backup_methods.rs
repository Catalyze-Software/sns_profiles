use candid::candid_method;
use ic_cdk_macros::{query, update};

use crate::{backup::Chunk, store::BACKUP};

#[update]
#[candid_method(update)]
pub fn backup_data() {
    BACKUP.with(|b| b.borrow_mut().backup_data())
}

#[query]
#[candid_method(query)]
pub fn check_backup_data_chunks() -> usize {
    BACKUP.with(|b| b.borrow_mut().check_backup_data_chunks())
}

#[query]
#[candid_method(query)]
pub fn download_chunk(n: u64) -> Vec<u8> {
    BACKUP.with(|b| b.borrow_mut().download_chunk(n))
}

#[update]
#[candid_method(update)]
pub fn restore_data() {
    BACKUP.with(|b| b.borrow_mut().restore_data())
}

#[update]
#[candid_method(update)]
pub fn upload_chunk(chunk: Chunk) {
    BACKUP.with(|b| b.borrow_mut().upload_chunk(chunk))
}
