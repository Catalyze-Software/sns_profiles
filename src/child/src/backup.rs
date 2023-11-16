use crate::store::DATA;
use candid::{CandidType, Decode, Encode};
use ic_scalable_canister::store::Data;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use shared::profile_models::Profile;

pub type ChunkId = u64;
pub type Hash = Vec<u8>;
pub const CHUNK_SIZE: u64 = 1 * 1024 * 1024;

#[derive(Default)]
pub struct Backup {
    pub hash: Hash,
    pub chunks: Vec<Vec<u8>>,
}

#[derive(CandidType, Serialize, Deserialize, Default)]
pub struct Chunk {
    chunk_id: ChunkId,
    data: Vec<u8>,
}

impl Backup {
    pub fn backup_data(&mut self) {
        DATA.with(|data| {
            let data = data.borrow();
            let serialized = Encode!(&*data).expect("Failed to encode data");
            let hash = Sha256::digest(&serialized);

            // clear first
            self.chunks.clear();
            self.hash.clear();

            self.hash = hash.to_vec();
            for data in serialized.chunks(CHUNK_SIZE as usize) {
                self.chunks.extend_from_slice(&[data.to_vec()])
            }
        })
    }

    pub fn check_backup_data_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn download_chunk(&self, n: u64) -> Vec<u8> {
        self.chunks[n as usize].clone()
    }

    pub fn restore_data(&self) {
        let mut concatenated: Vec<u8> = Vec::new();

        for data in self.chunks.iter() {
            concatenated.extend_from_slice(data);
        }

        let restored_data = Decode!(&concatenated, Data<Profile>).expect("Failed to decode data");

        DATA.with(|data| {
            *data.borrow_mut() = restored_data;
        });
    }

    pub fn remove_backup(&mut self) {
        self.hash.clear();
        self.chunks.clear();
    }

    pub fn upload_chunk(&mut self, chunk: (u64, Vec<u8>)) {
        self.chunks.insert(chunk.0 as usize, chunk.1)
    }
}
