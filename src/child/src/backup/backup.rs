use std::cell::RefCell;

use crate::store::DATA;
use candid::{CandidType, Decode, Deserialize, Encode};
use ic_scalable_canister::store::Data;
use sha2::{Digest, Sha256};
use shared::profile_models::Profile;

thread_local! {
    pub static BACKUP: RefCell<Backup> = RefCell::new(Backup::default());
}

pub type ChunkId = u64;
pub type Hash = Vec<u8>;
pub const CHUNK_SIZE: u64 = 1 * 1024 * 1024;

#[derive(Default, CandidType, Clone)]
pub struct Backup {
    hash: Hash,
    chunks: Vec<Chunk>,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Chunk {
    chunk_id: ChunkId,
    data: Vec<u8>,
}

impl Backup {
    pub fn backup_data(&mut self) -> String {
        DATA.with(|data| {
            let data = data.borrow();

            let serialized = Encode!(&*data).expect("Failed to encode data");
            let _ = Decode!(&serialized, Data<Profile>).expect("Failed to decode data");

            let hash = Sha256::digest(&serialized);

            // clear first
            self.chunks.clear();
            self.hash.clear();

            for (i, data) in serialized.chunks(CHUNK_SIZE as usize).enumerate() {
                self.chunks.push(Chunk {
                    chunk_id: i as u64,
                    data: data.to_vec(),
                });
            }
            self.hash = hash.to_vec();
        });

        DATA.with(|data| {
            let data = data.borrow();

            let serialized = Encode!(&*data).expect("Failed to encode data");
            let _ = Decode!(&serialized, Data<Profile>).expect("Failed to decode data");

            let data_hash = Sha256::digest(&serialized);
            assert_eq!(data_hash.to_vec(), self.hash);

            data_hash.iter().map(|b| format!("{:02x}", b)).collect()
        })
    }

    pub fn restore_data(&self) -> String {
        let mut concatenated: Vec<u8> = Vec::new();
        for (i, chunk) in self.chunks.iter().enumerate() {
            if i as u64 != chunk.chunk_id {
                ic_cdk::trap(&format!(
                    "Chunk id mismatch: expected {}, got {}",
                    i, chunk.chunk_id
                ));
            }

            concatenated.extend_from_slice(chunk.data.as_slice());
        }

        let backup_hash = Sha256::digest(&concatenated);
        assert_eq!(self.hash, backup_hash.to_vec());

        let restored_data = Decode!(&concatenated, Data<Profile>).expect("Failed to decode data");

        DATA.with(|data| {
            let mut data = data.borrow_mut();
            *data = restored_data;
        });

        hash_string(&self.hash)
    }

    pub fn download_chunk(&self, n: u64) -> Chunk {
        Chunk {
            chunk_id: n,
            data: self.chunks[n as usize].data.clone(),
        }
    }

    pub fn upload_chunk(&mut self, chunk: Chunk) {
        let chunk = Chunk {
            chunk_id: chunk.chunk_id,
            data: chunk.data,
        };
        self.chunks.insert(chunk.chunk_id as usize, chunk);
    }

    pub fn finalize_upload(&mut self) -> String {
        let mut concatenated: Vec<u8> = Vec::new();
        for chunk in self.chunks.iter() {
            concatenated.extend_from_slice(&chunk.data.as_slice());
        }

        let hash = Sha256::digest(&concatenated);
        self.hash = hash.to_vec();

        hash_string(&hash.to_vec())
    }

    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn clear_backup(&mut self) {
        *self = Backup::default();
    }
}

fn hash_string(hash: &Hash) -> String {
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}
