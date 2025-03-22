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

use crate::order_processor::{NewOrderRequest, NewOrderResponse};
pub const rpc_url: &str = "https://eth.llamarpc.com";
// pub struct Prover {
//     client: EnvProver,
//     rpc_url: String,
//     pk: SP1ProvingKey,
//     vk: SP1VerifyingKey,
// }

// impl Prover {
//     pub fn new(cfg: crate::Config) -> Self {
//         let client = ProverClient::from_env();
//         let (pk, vk) = client.setup(OBSIDIAN_ELF);
//         Self {
//             client,
//             rpc_url: cfg.rpc_url,
//             pk,
//             vk,
//         }
//     }

//     pub async fn prove(
//         &self,
//         params: crate::order_processor::NewOrderRequest,
//     ) -> crate::order_processor::NewOrderResponse {

//         let mut stdin = SP1Stdin::new();
//         stdin.write(&vm_input);
//         let proof = self
//             .client
//             .prove(&self.pk, &stdin)
//             .groth16()
//             .run()
//             .expect("failed to generate proof");

//         println!("Successfully generated proof!");

//         println!("{:0x?}", proof.public_values);
//         let solidity_proof = proof.bytes();
//         println!("proof: 0x{}", hex::encode(solidity_proof.clone()));

//         NewOrderResponse {
//             block: block_number,
//             proof: format!("0x{}", hex::encode(solidity_proof)),
//             public_values: format!("0x{}", hex::encode(proof.public_values)),
//             error: None,
//         }
//     }
// }

pub struct Prover {
    socket_path: String,
}

impl Prover {
    pub fn new() -> Self {
        Self {
            socket_path: String::from("/tmp/zkvm_prover.sock"),
        }
    }

    pub async fn prove(&self, request: NewOrderRequest) -> Result<NewOrderResponse, String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let uniswap_storage_slot = vec![b256!(
            "0000000000000000000000000000000000000000000000000000000000000008"
        )];

        let pool_address = Address::from_str(&request.pool_address).unwrap();
        let provider = ProviderBuilder::new().on_http(reqwest::Url::from_str(rpc_url).unwrap());
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
            sell_amount: U256::from_str_radix(&request.amount, 10)
                .unwrap()
                .to_be_bytes_vec(),
            sell_token0: false,
            sell_token: Address::from_str(&request.sell_token).unwrap().to_vec(),
            buy_token: Address::from_str(&request.buy_token).unwrap().to_vec(),
            seller: Address::from_str(&request.address).unwrap().to_vec(),
        };
        let vm_input = ObsidianInput {
            swap_payload,
            block_verifier_inputs: inputs,
        };
        let mut stream = match UnixStream::connect(&self.socket_path).await {
            Ok(stream) => stream,
            Err(e) => return Err(format!("Failed to connect to prover: {}", e)),
        };

        // Send request
        let request_bytes = match bincode::serialize(&vm_input) {
            Ok(bytes) => bytes,
            Err(e) => return Err(format!("Failed to serialize request: {}", e)),
        };

        if let Err(e) = stream.write_all(&request_bytes).await {
            return Err(format!("Failed to send request: {}", e));
        }

        // Shutdown write to signal end of request
        if let Err(e) = stream.shutdown().await {
            return Err(format!("Failed to shutdown write: {}", e));
        }

        // Read response
        let mut buffer = Vec::new();
        if let Err(e) = stream.read_to_end(&mut buffer).await {
            return Err(format!("Failed to read response: {}", e));
        }

        // Deserialize response
        let mut prover_response: NewOrderResponse = match bincode::deserialize(&buffer) {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Failed to deserialize response: {}", e)),
        };

        prover_response.block = block_number;

        // Convert to NewOrderResponse
        Ok(prover_response)
    }
}
