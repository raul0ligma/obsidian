use alloy_primitives::{Address, FixedBytes, U256};
use alloy_sol_types::{sol, SolValue};
use serde::{Deserialize, Serialize};
use verifier::VerifierInputs;
pub mod decoder;
pub mod header;
pub mod states;
pub mod swapper;
pub mod verifier;

sol! {
    struct Order {
        address seller;
        bytes32 block_hash;
        uint256 block_number;
        uint256 bought_amount;
        uint256 sold_amount;
        address buy_token;
        address sell_token;
    }
}

pub fn pack_order(
    swapped: swapper::uni_v2_swapper::SwapOutput,
    block_number: u64,
    block_hash: Vec<u8>,
) -> Vec<u8> {
    Order {
        seller: Address::from_slice(&swapped.seller),
        block_hash: FixedBytes::from_slice(&block_hash),
        block_number: U256::from(block_number),
        bought_amount: U256::from_be_slice(&swapped.bought_amount),
        sold_amount: U256::from_be_slice(&swapped.sold_amount),
        buy_token: Address::from_slice(&swapped.buy_token),
        sell_token: Address::from_slice(&swapped.sell_token),
    }
    .abi_encode()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObsidianInput {
    pub block_verifier_inputs: VerifierInputs,
    pub swap_payload: swapper::uni_v2_swapper::SwapInput,
}
