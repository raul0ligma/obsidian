use dotenv::dotenv;
use server::order_processor::{NewOrderRequest, NewOrderResponse};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

use alloy::{hex, primitives::Keccak256};
use obsidian_lib::ObsidianInput;
use sp1_sdk::{include_elf, EnvProver, HashableKey, ProverClient, SP1ProvingKey, SP1Stdin};

const OBSIDIAN_ELF: &[u8] = include_elf!("obsidian-program");

fn main() {
    dotenv().ok();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let socket_path = "/tmp/zkvm_prover.sock";
    log::info!("initializing env prover");
    let client = ProverClient::from_env();
    log::info!("loading elf prover");
    let (pk, vk) = client.setup(OBSIDIAN_ELF);
    log::info!("computed verification key {}", vk.bytes32());

    // remove socket if it already exists
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path).unwrap();
    }

    let listener = UnixListener::bind(socket_path).unwrap();
    log::info!("prover service listening on {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                log::info!("new connection established");
                log::debug!("using verification key {}", vk.bytes32());
                handle_client(&client, &pk, stream);
            }
            Err(err) => {
                log::error!("connection error: {}", err);
                break;
            }
        }
    }
}

fn handle_client(client: &EnvProver, pk: &SP1ProvingKey, mut stream: UnixStream) {
    // read request
    let mut buffer = Vec::new();
    if let Err(e) = stream.read_to_end(&mut buffer) {
        log::error!("failed to read from stream: {}", e);
        return;
    }

    let request: ObsidianInput = match bincode::deserialize(&buffer) {
        Ok(req) => req,
        Err(e) => {
            log::error!("failed to parse request: {}", e);
            let response = NewOrderResponse {
                block: 0,
                proof: String::new(),
                public_values: String::new(),
                error: Some(format!("failed to parse request: {}", e)),
            };
            send_response(&mut stream, &response);
            return;
        }
    };

    let start = std::time::Instant::now();
    log::info!("starting proof generation");

    let response = generate_proof(client, pk, request);

    log::info!("proof generation completed in {:?}", start.elapsed());
    send_response(&mut stream, &response);
}

fn send_response(stream: &mut UnixStream, response: &NewOrderResponse) {
    let response_bytes = match bincode::serialize(response) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("failed to serialize response: {}", e);
            return;
        }
    };

    if let Err(e) = stream.write_all(&response_bytes) {
        log::error!("failed to write response: {}", e);
    }
}

fn generate_proof(
    client: &EnvProver,
    pk: &SP1ProvingKey,
    vm_input: ObsidianInput,
) -> NewOrderResponse {
    let mut stdin = SP1Stdin::new();
    stdin.write(&vm_input);

    match client.prove(pk, &stdin).groth16().run() {
        Ok(proof) => {
            log::info!("successfully generated proof");

            let solidity_proof = proof.bytes();
            let hex_proof = hex::encode(solidity_proof.clone());
            let hex_public_values = hex::encode(proof.public_values);

            log::debug!("public values: 0x{}", hex_public_values);
            log::debug!("proof: 0x{}", hex_proof);

            NewOrderResponse {
                block: 0,
                proof: format!("0x{}", hex_proof),
                public_values: format!("0x{}", hex_public_values),
                error: None,
            }
        }
        Err(e) => {
            log::error!("failed to generate proof: {}", e);
            NewOrderResponse {
                block: 0,
                proof: String::new(),
                public_values: String::new(),
                error: Some(format!("proof generation failed: {}", e)),
            }
        }
    }
}
