use server::order_processor::{NewOrderRequest, NewOrderResponse};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
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
use sp1_build::{build_program_with_args, BuildArgs};
use sp1_sdk::{
    include_elf, EnvProver, HashableKey, ProverClient, SP1ProvingKey, SP1Stdin, SP1VerifyingKey,
};

const OBSIDIAN_ELF: &[u8] = include_elf!("obsidian-program");

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let socket_path = "/tmp/zkvm_prover.sock";
    println!("initializing env prover");
    let client = ProverClient::from_env();
    println!("loading elf prover");
    let (pk, vk) = client.setup(OBSIDIAN_ELF);
    println!("computed vk {}", vk.bytes32());
    // Remove socket if it already exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path).unwrap();
    }

    let listener = UnixListener::bind(socket_path).unwrap();
    println!("Prover service listening on {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection");
                println!("verifying key {}", vk.bytes32());
                handle_client(&client, &pk, stream);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }
}

fn handle_client(client: &EnvProver, pk: &SP1ProvingKey, mut stream: UnixStream) {
    // Read request
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();

    let request: ObsidianInput = match bincode::deserialize(&buffer) {
        Ok(req) => req,
        Err(e) => {
            let response = NewOrderResponse {
                block: 0,
                proof: String::new(),
                public_values: String::new(),
                error: Some(format!("Failed to parse request: {}", e)),
            };
            let response_bytes = bincode::serialize(&response).unwrap();
            stream.write_all(&response_bytes).unwrap();
            return;
        }
    };

    let start = std::time::Instant::now();
    println!("Starting proof generation");

    let response = generate_proof(client, pk, request);

    println!("Proof generation completed in {:?}", start.elapsed());

    let response_bytes = bincode::serialize(&response).unwrap();
    stream.write_all(&response_bytes).unwrap();
}

fn generate_proof(
    client: &EnvProver,
    pk: &SP1ProvingKey,
    vm_input: ObsidianInput,
) -> NewOrderResponse {
    let mut stdin = SP1Stdin::new();
    stdin.write(&vm_input);
    let proof = client
        .prove(pk, &stdin)
        .groth16()
        .run()
        .expect("failed to generate proof");

    println!("Successfully generated proof!");

    println!("{:0x?}", proof.public_values);
    let solidity_proof = proof.bytes();
    println!("proof: 0x{}", hex::encode(solidity_proof.clone()));

    NewOrderResponse {
        block: 0,
        proof: format!("0x{}", hex::encode(solidity_proof)),
        public_values: format!("0x{}", hex::encode(proof.public_values)),
        error: None,
    }
}
