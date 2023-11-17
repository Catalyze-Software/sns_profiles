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
    hash: Option<Hash>,
    chunks: Vec<Vec<u8>>,
}

#[derive(CandidType, Deserialize)]
pub struct Chunk {
    chunk_id: ChunkId,
    data: Vec<u8>,
}

impl Backup {
    pub fn backup_data(&mut self) -> String {
        DATA.with(|data| {
            let data = data.borrow();

            let serialized = Encode!(&*data).expect("Failed to encode data");
            let hash = Sha256::digest(&serialized);

            // clear first
            self.chunks.clear();
            self.hash = None;

            for data in serialized.chunks(CHUNK_SIZE as usize) {
                self.chunks.extend_from_slice(&[data.to_vec()]);
            }
            self.hash = Some(hash.to_vec());

            hash_string(hash.to_vec())
        })
    }

    pub fn restore_data(&self) -> String {
        let mut concatenated: Vec<u8> = Vec::new();
        for data in self.chunks.iter() {
            concatenated.extend_from_slice(data);
        }

        let hash = Sha256::digest(&concatenated);

        match self.hash.clone() {
            None => panic!("No backup found"),
            Some(backup_hash) => assert_eq!(backup_hash.to_vec(), hash.to_vec()),
        }

        let restored_data = Decode!(&concatenated, Data<Profile>).expect("Failed to decode data");

        DATA.with(|data| {
            let mut data = data.borrow_mut();
            let serialized = Encode!(&*data).expect("Failed to encode data");
            let data_hash = Sha256::digest(&serialized);

            assert_eq!(data_hash.to_vec(), hash.to_vec());

            *data = restored_data;
        });

        hash_string(hash.to_vec())
    }

    pub fn download_chunk(&self, n: u64) -> Chunk {
        Chunk {
            chunk_id: n,
            data: self.chunks[n as usize].clone(),
        }
    }

    pub fn upload_chunk(&mut self, chunk: Chunk) {
        match self.hash {
            Some(_) => panic!("Backup not empty"),
            None => {
                self.chunks.insert(chunk.chunk_id as usize, chunk.data);
            }
        }
    }

    pub fn finalize_upload(&mut self) -> String {
        let mut concatenated: Vec<u8> = Vec::new();
        for chunk in self.chunks.iter() {
            concatenated.extend_from_slice(chunk);
        }

        let hash = Sha256::digest(&concatenated);
        self.hash = Some(hash.to_vec());

        hash_string(hash.to_vec())
    }

    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }

    pub fn clear_backup(&mut self) {
        *self = Backup::default();
    }

    pub fn hash(&self) -> String {
        match self.hash.clone() {
            None => panic!("No backup found"),
            Some(hash) => hash_string(hash),
        }
    }
}

fn hash_string(hash: Hash) -> String {
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}
