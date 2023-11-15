use crate::store::DATA;
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::call;
use ic_cdk_macros::update;
use ic_scalable_canister::store::Data;
use sha2::{Digest, Sha256};
use shared::profile_models::Profile;
use std::cell::RefCell;

const CHUNK_SIZE: u64 = 1 * 1024 * 1024;
const BACKUP_CANISTER: &str = "iqqqg-mqaaa-aaaap-abrnq-cai";

#[derive(Default, Clone)]
pub struct Backup {
    pub hash: Vec<u8>,
    pub chunks: Vec<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Chunk {
    chunk_id: u64,
    data: Vec<u8>,
}

thread_local! {
    static BACKUP: RefCell<Backup> = RefCell::new(Backup::default());
}

#[update]
fn backup_data() {
    DATA.with(|data| {
        let data = data.borrow();
        let serialized = Encode!(&*data).expect("Failed to encode data");
        let hash = Sha256::digest(&serialized);

        BACKUP.with(|backup| {
            let mut backup = backup.borrow_mut();

            // clear first
            *backup = Backup::default();

            backup.hash = hash.to_vec();
            for data in serialized.chunks(CHUNK_SIZE as usize) {
                backup.chunks.extend_from_slice(&[data.to_vec()])
            }
        });
    })
}

#[update]
fn restore_data() {
    BACKUP.with(|backup| {
        let backup = backup.borrow();
        let mut concatenated: Vec<u8> = Vec::new();

        for data in backup.chunks.iter() {
            concatenated.extend_from_slice(data);
        }

        let restored_data: Data<Profile> =
            Decode!(&concatenated, Data<Profile>).expect("Failed to decode data");

        DATA.with(|data| {
            *data.borrow_mut() = restored_data;
        });
    });
}

#[update]
async fn push() {
    let canister_id = Principal::from_text(BACKUP_CANISTER).expect("Failed to parse principal");

    let backup: Backup = BACKUP.with(|backup| backup.borrow().clone());

    call::<(), ()>(canister_id, "clear", ())
        .await
        .expect("Failed to call clear");

    for chunk in backup.chunks.into_iter() {
        call::<(Vec<u8>,), ()>(canister_id, "addData", (chunk,))
            .await
            .expect("Failed to call addData");
    }
}

#[update]
async fn pull() {
    let canister_id = Principal::from_text(BACKUP_CANISTER).expect("Failed to parse principal");

    let n = call::<(), (u64,)>(canister_id, "size", ())
        .await
        .expect("Failed to call size")
        .0;

    let mut chunks: Vec<Vec<u8>> = Vec::new();

    for i in 0..n {
        let chunk: (Vec<u8>,) = call::<(u64,), (Vec<u8>,)>(canister_id, "getChunk", (i,))
            .await
            .expect("Failed to call getChunk");
        chunks.push(chunk.0);
    }
}
