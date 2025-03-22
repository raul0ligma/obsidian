use std::str::FromStr;

use alloy::{
    hex,
    primitives::{b256, Address},
    providers::ProviderBuilder,
    rpc,
    transports::http::reqwest,
};
use alloy_primitives::U256;
use alloy_provider::Provider;
use obsidian_lib::{
    header::LeanHeader,
    swapper::uni_v2_swapper::SwapInput,
    verifier::{Proofs, VerifierInputs},
    ObsidianInput,
};

use crate::order_processor::{NewOrderRequest, NewOrderResponse};

pub const RPC_URL: &str = "https://base-rpc.publicnode.com";

pub struct Prover {
    socket_path: String,
}

impl Default for Prover {
    fn default() -> Self {
        Self {
            socket_path: String::from("/tmp/zkvm_prover.sock"),
        }
    }
}

impl Prover {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn prove(&self, request: NewOrderRequest) -> Result<NewOrderResponse, String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        log::info!("preparing proof inputs for request");

        // prepare uniswap storage slot
        let uniswap_storage_slot = [b256!(
            "0000000000000000000000000000000000000000000000000000000000000008"
        )];

        log::debug!("commit block: {}", request.commit_block);

        // parse pool address
        let pool_address = match Address::from_str(&request.pool_address) {
            Ok(addr) => addr,
            Err(e) => return Err(format!("invalid pool address: {}", e)),
        };

        // connect to provider
        let provider = ProviderBuilder::new().on_http(match reqwest::Url::from_str(RPC_URL) {
            Ok(url) => url,
            Err(e) => return Err(format!("invalid RPC URL: {}", e)),
        });

        // get block data
        let latest = match provider
            .get_block_by_number(rpc::types::BlockNumberOrTag::Number(request.commit_block))
            .await
        {
            Ok(Some(block)) => block,
            Ok(None) => return Err(format!("block {} not found", request.commit_block)),
            Err(e) => return Err(format!("failed to fetch block: {}", e)),
        };

        let block_number = latest.header.number;
        log::info!("using block {} for proof generation", block_number);

        // prepare custom eth_getProof parameters
        let params = serde_json::json!([
            format!("{:#x}", pool_address),
            uniswap_storage_slot
                .iter()
                .map(|slot| format!("{:#x}", slot))
                .collect::<Vec<String>>(),
            format!("0x{:x}", request.commit_block)
        ]);

        log::debug!("eth_getProof params: {}", params);

        // get proof data using raw request
        let proof: rpc::types::EIP1186AccountProofResponse =
            match provider.raw_request("eth_getProof".into(), params).await {
                Ok(proof) => proof,
                Err(e) => return Err(format!("failed to get proof: {}", e)),
            };

        // log storage value for debugging
        if let Some(first_proof) = proof.storage_proof.first() {
            log::debug!(
                "retrieved storage value: {:0x?}",
                first_proof.value.to_be_bytes_vec()
            );
        } else {
            return Err("storage proof is empty".to_string());
        }

        // collect account proofs
        let mut account_collector: Vec<Vec<u8>> = Vec::new();
        for acc in proof.account_proof.clone() {
            account_collector.push(acc.to_vec());
        }

        // collect storage proofs
        let slots = proof.storage_proof.clone();
        if let Some(first_slot) = slots.first() {
            let mut storage_collector: Vec<Vec<u8>> = Vec::new();
            for acc in first_slot.clone().proof {
                storage_collector.push(acc.to_vec());
            }

            // parse token addresses
            let sell_token = match Address::from_str(&request.sell_token) {
                Ok(addr) => addr,
                Err(e) => return Err(format!("invalid sell token address: {}", e)),
            };

            let buy_token = match Address::from_str(&request.buy_token) {
                Ok(addr) => addr,
                Err(e) => return Err(format!("invalid buy token address: {}", e)),
            };

            let seller = match Address::from_str(&request.address) {
                Ok(addr) => addr,
                Err(e) => return Err(format!("invalid seller address: {}", e)),
            };

            // parse sell amount
            let sell_amount = match U256::from_str_radix(&request.amount, 10) {
                Ok(amount) => amount,
                Err(e) => return Err(format!("invalid sell amount: {}", e)),
            };

            // prepare verifier inputs
            let inputs = VerifierInputs {
                header: LeanHeader::from(latest.header.inner),
                address: pool_address.to_vec(),
                storage_slot: uniswap_storage_slot.first().unwrap().to_vec(),
                proofs: Proofs {
                    account_proof: account_collector,
                    storage_proof: storage_collector,
                },
            };

            // prepare swap payload
            let swap_payload = SwapInput {
                sell_amount: sell_amount.to_be_bytes_vec(),
                sell_token0: false,
                sell_token: sell_token.to_vec(),
                buy_token: buy_token.to_vec(),
                seller: seller.to_vec(),
            };

            // combine inputs
            let vm_input = ObsidianInput {
                swap_payload,
                block_verifier_inputs: inputs,
            };

            // connect to prover service
            log::info!("connecting to prover service at {}", self.socket_path);
            let mut stream = match UnixStream::connect(&self.socket_path).await {
                Ok(stream) => stream,
                Err(e) => return Err(format!("failed to connect to prover: {}", e)),
            };

            // send request
            let request_bytes = match bincode::serialize(&vm_input) {
                Ok(bytes) => bytes,
                Err(e) => return Err(format!("failed to serialize request: {}", e)),
            };

            if let Err(e) = stream.write_all(&request_bytes).await {
                return Err(format!("failed to send request: {}", e));
            }

            // shutdown write to signal end of request
            if let Err(e) = stream.shutdown().await {
                return Err(format!("failed to shutdown write: {}", e));
            }

            log::info!("request sent, waiting for proof generation");

            // read response
            let mut buffer = Vec::new();
            if let Err(e) = stream.read_to_end(&mut buffer).await {
                return Err(format!("failed to read response: {}", e));
            }

            // deserialize response
            let mut prover_response: NewOrderResponse = match bincode::deserialize(&buffer) {
                Ok(resp) => resp,
                Err(e) => return Err(format!("failed to deserialize response: {}", e)),
            };

            // check for error in response
            if let Some(error) = &prover_response.error {
                log::error!("prover service returned error: {}", error);
                return Err(format!("prover service error: {}", error));
            }

            log::info!("successfully received proof");

            // add block number to response
            prover_response.block = block_number;

            Ok(prover_response)
        } else {
            Err("no storage slots found in proof".to_string())
        }
    }
}
