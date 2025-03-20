use alloy_primitives::{Address, Bloom, Bytes, B256, B64, U256};
use alloy_rlp::{BufMut, Encodable};
use serde::{Deserialize, Serialize};
use tiny_keccak::{self, Hasher};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeanHeader {
    pub parent_hash: [u8; 32],
    pub ommers_hash: [u8; 32],
    pub beneficiary: [u8; 20],
    pub state_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub logs_bloom: Vec<u8>,
    pub difficulty: [u8; 32],
    pub number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub extra_data: Vec<u8>,
    pub mix_hash: [u8; 32],
    pub nonce: [u8; 8],
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<[u8; 32]>,
    pub blob_gas_used: Option<u64>,
    pub excess_blob_gas: Option<u64>,
    pub parent_beacon_block_root: Option<[u8; 32]>,
    pub requests_hash: Option<[u8; 32]>,
}

impl LeanHeader {
    fn keccak(input: &[u8]) -> Vec<u8> {
        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(input);
        let out = &mut [0_u8; 32];
        hasher.finalize(out);
        out.to_vec()
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut buf = Vec::new();
        self.encode(&mut buf);

        let mut out = [0u8; 32];
        let hash_vec = Self::keccak(&buf);
        out.copy_from_slice(&hash_vec);
        out
    }

    pub fn encode(&self, out: &mut dyn BufMut) {
        let list_header = alloy_rlp::Header {
            list: true,
            payload_length: self.header_payload_length(),
        };
        list_header.encode(out);

        B256::from_slice(&self.parent_hash).encode(out);
        B256::from_slice(&self.ommers_hash).encode(out);
        Address::from_slice(&self.beneficiary).encode(out);
        B256::from_slice(&self.state_root).encode(out);
        B256::from_slice(&self.transactions_root).encode(out);
        B256::from_slice(&self.receipts_root).encode(out);
        Bloom::from_slice(&self.logs_bloom).encode(out);
        U256::from_be_slice(&self.difficulty).encode(out);
        U256::from(self.number).encode(out);
        U256::from(self.gas_limit).encode(out);
        U256::from(self.gas_used).encode(out);
        U256::from(self.timestamp).encode(out);
        Bytes::copy_from_slice(&self.extra_data).encode(out);
        B256::from_slice(&self.mix_hash).encode(out);
        B64::from_slice(&self.nonce).encode(out);

        if let Some(base_fee) = self.base_fee_per_gas {
            U256::from(base_fee).encode(out);
        }

        if let Some(ref root) = self.withdrawals_root {
            B256::from_slice(root).encode(out);
        }

        if let Some(blob_gas_used) = self.blob_gas_used {
            U256::from(blob_gas_used).encode(out);
        }

        if let Some(excess_blob_gas) = self.excess_blob_gas {
            U256::from(excess_blob_gas).encode(out);
        }

        if let Some(ref parent_root) = self.parent_beacon_block_root {
            B256::from_slice(parent_root).encode(out);
        }

        if let Some(ref requests) = self.requests_hash {
            B256::from_slice(requests).encode(out);
        }
    }

    fn header_payload_length(&self) -> usize {
        let mut length = 0;

        length += B256::from_slice(&self.parent_hash).length();
        length += B256::from_slice(&self.ommers_hash).length();
        length += Address::from_slice(&self.beneficiary).length();
        length += B256::from_slice(&self.state_root).length();
        length += B256::from_slice(&self.transactions_root).length();
        length += B256::from_slice(&self.receipts_root).length();
        length += Bloom::from_slice(&self.logs_bloom).length();
        length += U256::from_be_slice(&self.difficulty).length();
        length += U256::from(self.number).length();
        length += U256::from(self.gas_limit).length();
        length += U256::from(self.gas_used).length();
        length += U256::from(self.timestamp).length();
        length += Bytes::copy_from_slice(&self.extra_data).length();
        length += B256::from_slice(&self.mix_hash).length();
        length += B64::from_slice(&self.nonce).length();

        if let Some(base_fee) = self.base_fee_per_gas {
            length += U256::from(base_fee).length();
        }

        if let Some(ref root) = self.withdrawals_root {
            length += B256::from_slice(root).length();
        }

        if let Some(blob_gas_used) = self.blob_gas_used {
            length += U256::from(blob_gas_used).length();
        }

        if let Some(excess_blob_gas) = self.excess_blob_gas {
            length += U256::from(excess_blob_gas).length();
        }

        if let Some(ref parent_root) = self.parent_beacon_block_root {
            length += B256::from_slice(parent_root).length();
        }

        if let Some(ref requests) = self.requests_hash {
            length += B256::from_slice(requests).length();
        }

        length
    }
}

impl From<alloy_consensus::Header> for LeanHeader {
    fn from(header: alloy_consensus::Header) -> Self {
        let mut parent_hash = [0u8; 32];
        let mut ommers_hash = [0u8; 32];
        let mut beneficiary = [0u8; 20];
        let mut state_root = [0u8; 32];
        let mut transactions_root = [0u8; 32];
        let mut receipts_root = [0u8; 32];
        let mut logs_bloom = [0u8; 256];
        let mut difficulty = [0u8; 32];
        let mut mix_hash = [0u8; 32];
        let mut nonce = [0u8; 8];

        parent_hash.copy_from_slice(header.parent_hash.as_slice());
        ommers_hash.copy_from_slice(header.ommers_hash.as_slice());
        beneficiary.copy_from_slice(header.beneficiary.as_slice());
        state_root.copy_from_slice(header.state_root.as_slice());
        transactions_root.copy_from_slice(header.transactions_root.as_slice());
        receipts_root.copy_from_slice(header.receipts_root.as_slice());
        let logs_bloom = header.logs_bloom.as_slice().to_vec();
        difficulty.copy_from_slice(&header.difficulty.to_be_bytes::<32>());
        mix_hash.copy_from_slice(header.mix_hash.as_slice());
        nonce.copy_from_slice(header.nonce.as_slice());

        let withdrawals_root = header.withdrawals_root.map(|wr| {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(wr.as_slice());
            bytes
        });

        let parent_beacon_block_root = header.parent_beacon_block_root.map(|pbr| {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(pbr.as_slice());
            bytes
        });

        let requests_hash = header.requests_hash.map(|rh| {
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(rh.as_slice());
            bytes
        });

        LeanHeader {
            parent_hash,
            ommers_hash,
            beneficiary,
            state_root,
            transactions_root,
            receipts_root,
            logs_bloom,
            difficulty,
            number: header.number,
            gas_limit: header.gas_limit,
            gas_used: header.gas_used,
            timestamp: header.timestamp,
            extra_data: header.extra_data.to_vec(),
            mix_hash,
            nonce,
            base_fee_per_gas: header.base_fee_per_gas,
            withdrawals_root,
            blob_gas_used: header.blob_gas_used,
            excess_blob_gas: header.excess_blob_gas,
            parent_beacon_block_root,
            requests_hash,
        }
    }
}
