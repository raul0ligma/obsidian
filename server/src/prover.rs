use std::str::FromStr;

use alloy::{
    hex::{self, FromHex},
    primitives::{address, b256, fixed_bytes, keccak256, Address, Keccak256},
    providers::{Provider, ProviderBuilder},
    rpc,
    serde::quantity::vec,
    sol_types,
    transports::http::reqwest,
};
use alloy_primitives::U256;
use obsidian_lib::{
    header::LeanHeader,
    swapper::uni_v2_swapper::SwapInput,
    verifier::{Proofs, VerifierInputs},
    ObsidianInput,
};
use sp1_sdk::{
    include_elf, EnvProver, HashableKey, ProverClient, SP1ProvingKey, SP1Stdin, SP1VerifyingKey,
};

use crate::order_processor::NewOrderResponse;
pub const OBSIDIAN_ELF: &[u8] = include_bytes!("../../.artifacts/obsidian-program");

pub struct Prover {
    client: EnvProver,
    rpc_url: String,
    pk: SP1ProvingKey,
    vk: SP1VerifyingKey,
}

impl Prover {
    pub fn new(cfg: crate::Config) -> Self {
        let client = ProverClient::from_env();
        let (pk, vk) = client.setup(OBSIDIAN_ELF);
        Self {
            client,
            rpc_url: cfg.rpc_url,
            pk,
            vk,
        }
    }

    pub async fn prove(
        &self,
        params: crate::order_processor::NewOrderRequest,
    ) -> crate::order_processor::NewOrderResponse {
        let uniswap_storage_slot = vec![b256!(
            "0000000000000000000000000000000000000000000000000000000000000008"
        )];

        let pool_address = Address::from_str(&params.pool_address).unwrap();
        let provider =
            ProviderBuilder::new().on_http(reqwest::Url::from_str(&self.rpc_url).unwrap());
        let latest = provider
            .get_block_by_number(rpc::types::BlockNumberOrTag::Latest)
            .await
            .unwrap()
            .unwrap();
        println!("latest block {}", latest.header.number);

        let proof: rpc::types::EIP1186AccountProofResponse = provider
            .get_proof(pool_address, uniswap_storage_slot.clone())
            .await
            .unwrap();

        println!(
            "expected {:0x?}",
            (proof
                .clone()
                .storage_proof
                .first()
                .unwrap()
                .value
                .to_be_bytes_vec())
        );

        let mut account_collector: Vec<Vec<u8>> = Vec::new();
        for acc in proof.account_proof.clone() {
            account_collector.push(acc.to_vec());
        }

        let slots = proof.storage_proof.clone();
        let mut storage_collector: Vec<Vec<u8>> = Vec::new();
        for acc in slots.first().unwrap().clone().proof {
            storage_collector.push(acc.to_vec());
        }
        let block_number = latest.header.number;

        let inputs = VerifierInputs {
            header: LeanHeader::from(latest.header.inner),
            address: pool_address.to_vec(),
            storage_slot: uniswap_storage_slot.first().unwrap().to_vec(),
            proofs: Proofs {
                account_proof: account_collector,
                storage_proof: storage_collector,
            },
        };
        let swap_payload = SwapInput {
            sell_amount: U256::from_str_radix(&params.amount, 10)
                .unwrap()
                .to_be_bytes_vec(),
            sell_token0: false,
            sell_token: Address::from_str(&params.sell_token).unwrap().to_vec(),
            buy_token: Address::from_str(&params.buy_token).unwrap().to_vec(),
            seller: Address::from_str(&params.address).unwrap().to_vec(),
        };
        let vm_input = ObsidianInput {
            swap_payload,
            block_verifier_inputs: inputs,
        };

        let mut stdin = SP1Stdin::new();
        stdin.write(&vm_input);
        let proof = self
            .client
            .prove(&self.pk, &stdin)
            .groth16()
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        println!("{:0x?}", proof.public_values);
        let solidity_proof = proof.bytes();
        println!("proof: 0x{}", hex::encode(solidity_proof.clone()));

        NewOrderResponse {
            block: block_number,
            proof: format!("0x{}", hex::encode(solidity_proof)),
            public_values: format!("0x{}", hex::encode(proof.public_values)),
            error: None,
        }
    }
}
