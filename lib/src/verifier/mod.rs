use crate::header::LeanHeader;
use alloy_primitives::U256;
use alloy_sol_types::{sol, SolStruct};
use serde::{Deserialize, Serialize};
use tiny_keccak::Hasher;

use crate::decoder::NodeDecoder;

#[derive(Debug)]
pub enum NodeType {
    Branch(Vec<Vec<u8>>),
    Extension(bool, Vec<u8>, Vec<u8>),
    Leaf(bool, Vec<u8>, Vec<u8>),
}

pub struct Node {
    pub original: Vec<u8>,
    pub node: NodeType,
}

#[derive(Debug)]
pub struct AccountState {
    pub storage_hash: Vec<u8>,
    pub balance: U256,
    pub code_hash: Vec<u8>,
    pub nonce: U256,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Proofs {
    pub account_proof: Vec<Vec<u8>>,
    pub storage_proof: Vec<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifierInputs {
    pub header: LeanHeader,
    pub address: Vec<u8>,
    pub storage_slot: Vec<u8>,
    pub proofs: Proofs,
}

#[derive(Debug)]
pub struct VerifierOutput {
    pub block_hash: Vec<u8>,
    pub slot_data: Vec<u8>,
}

pub type VerifyResultWithData<T> = Result<T, String>;

pub struct MPTVerifier;

impl MPTVerifier {
    fn key_to_nibbles(key: &[u8]) -> Vec<u8> {
        let mut nibbles: Vec<u8> = Vec::with_capacity(key.len() * 2);
        for &byte in key {
            nibbles.push(byte >> 4); // upper half
            nibbles.push(byte & 0x0F); // lower half
        }
        nibbles
    }

    fn keccak(input: &[u8]) -> Vec<u8> {
        let mut hasher = tiny_keccak::Keccak::v256();
        hasher.update(input);
        let out = &mut [0_u8; 32];
        hasher.finalize(out);
        out.to_vec()
    }

    fn verify_and_get_data(
        root_hash: Vec<u8>,
        key: &[u8],
        proof: Vec<Node>,
    ) -> VerifyResultWithData<Vec<u8>> {
        let mut current_hash = root_hash;
        let nibbles = Self::key_to_nibbles(key);
        let mut nibble_index = 0;
        for node_proof in proof {
            if current_hash.len() < 32 {
                // println!("{:0x?} {:0x?}", node_proof.original, current_hash);

                if current_hash != node_proof.original {
                    panic!(
                        "expected {:0x?} got {:0x?}",
                        node_proof.original, current_hash
                    )
                }
            } else {
                let out_hash = Self::keccak(&node_proof.original);
                // println!("{:0x?} {:0x?}", out_hash, current_hash);

                if out_hash != current_hash {
                    panic!("expected {:0x?} got {:0x?}", out_hash, current_hash)
                }
            }

            match node_proof.node {
                NodeType::Branch(val) => {
                    if nibble_index > nibbles.len() {
                        //println!("{:0x?} found", val[16].clone());
                        return Ok(val[16].clone());
                    }

                    let nibble = nibbles[nibble_index] as usize;
                    //println!("{:?} {:?} {:0x?}", nibble, nibble_index, val[nibble]);
                    nibble_index += 1;
                    if val[nibble].is_empty() {
                        panic!("path not found in branch");
                    }

                    if val[nibble].len() < 32 {
                        // copy everything
                        current_hash.copy_from_slice(&val[nibble]);
                    } else {
                        // only copy 32 bytes, which should always be the  full length
                        current_hash.copy_from_slice(&val[nibble][0..32]);
                    }
                }
                NodeType::Leaf(is_odd, slug, value) => {
                    let mut slug_nibbles = Self::key_to_nibbles(&slug);
                    // always pop first nibble because it has type
                    slug_nibbles.remove(0);
                    if !is_odd {
                        // remove if not odd
                        slug_nibbles.remove(0);
                    }

                    for &val in slug_nibbles.iter() {
                        if nibbles[nibble_index] != val {
                            panic!("did not match")
                        }
                        nibble_index += 1;
                    }

                    // we should have matched everything by now
                    if nibble_index != nibbles.len() {
                        panic!("incomplete leaf node")
                    }

                    return Ok(value);
                }
                NodeType::Extension(is_odd, slug, next) => {
                    let mut slug_nibbles = Self::key_to_nibbles(&slug);
                    // always pop first nibble because it has type
                    slug_nibbles.remove(0);
                    if !is_odd {
                        // remove if not odd
                        slug_nibbles.remove(0);
                    }

                    for &val in slug_nibbles.iter() {
                        if nibbles[nibble_index] != val {
                            panic!("did not match")
                        }
                        nibble_index += 1;
                    }

                    if next.len() < 32 {
                        // copy everything
                        current_hash.copy_from_slice(&next);
                    } else {
                        // only copy 32 bytes, which should always be the  full length
                        current_hash.copy_from_slice(&next[0..32]);
                    }
                }
            }
        }
        Ok(Vec::new())
    }

    pub fn verify_and_get_account_state(
        state_root: &[u8],
        address: Vec<u8>,
        proof: Vec<Node>,
    ) -> VerifyResultWithData<AccountState> {
        let address_hash = Self::keccak(&address);
        let out = Self::verify_and_get_data(state_root.to_vec(), &address_hash, proof).unwrap();
        let decode = NodeDecoder::decode_rlp(&out);
        if decode.len() < 4 {
            return Err(format!(
                "inconsistent account state with length {}",
                decode.len()
            ));
        }

        Ok(AccountState {
            storage_hash: decode[2].clone(),
            balance: U256::from_be_slice(decode[1].as_slice()),
            code_hash: decode[3].clone(),
            nonce: U256::from_be_slice(decode[0].as_slice()),
        })
    }

    pub fn verify_and_get_slot(
        state_root: &[u8],
        slot: Vec<u8>,
        proof: Vec<Node>,
    ) -> VerifyResultWithData<Vec<u8>> {
        let slot_hash = Self::keccak(&slot);
        let out = Self::verify_and_get_data(state_root.to_vec(), &slot_hash, proof).unwrap();
        let decode = NodeDecoder::decode_rlp(&out);
        if decode.is_empty() {
            return Err(format!("no storage found {}", decode.len()));
        }

        Ok(decode[0].clone())
    }

    pub fn verify_slot(input: VerifierInputs) -> VerifyResultWithData<VerifierOutput> {
        // compute the block hash here
        let block_hash = input.header.hash();

        let mut account_proofs: Vec<Node> = Vec::new();
        for node in input.proofs.account_proof {
            account_proofs.push(NodeDecoder::decode_mpt_node(&node));
        }

        // start from state root
        let account_state = Self::verify_and_get_account_state(
            input.header.state_root.as_slice(),
            input.address,
            account_proofs,
        )?;

        let mut storage_proofs: Vec<Node> = Vec::new();
        for node in input.proofs.storage_proof {
            storage_proofs.push(NodeDecoder::decode_mpt_node(&node));
        }

        // verify with computed storage hash
        let storage_value = Self::verify_and_get_slot(
            &account_state.storage_hash,
            input.storage_slot,
            storage_proofs,
        )?;

        Ok(VerifierOutput {
            block_hash: block_hash.to_vec(),
            slot_data: storage_value,
        })
    }
}
