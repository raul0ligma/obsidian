//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use std::str::FromStr;

use alloy::{
    hex::FromHex,
    primitives::{address, b256, fixed_bytes, keccak256, Address, Keccak256},
    providers::{Provider, ProviderBuilder},
    rpc,
    serde::quantity::vec,
    sol_types,
    transports::http::reqwest,
};
use alloy_primitives::U256;
use alloy_rlp::Encodable;
use alloy_sol_types::SolType;
use clap::Parser;
use obsidian_lib::{
    decoder::NodeDecoder,
    header::LeanHeader,
    states::uni_v2,
    swapper::uni_v2_swapper::{self, SwapInput},
    verifier::{Node, Proofs, VerifierInputs},
    ObsidianInput,
};
use sp1_sdk::{include_elf, HashableKey, ProverClient, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const OBSIDIAN_ELF: &[u8] = include_elf!("obsidian-program");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,

    #[clap(long)]
    local: bool,
}

#[tokio::main]
async fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    let args = Args::parse();

    let rpc_url = "https://eth.llamarpc.com";
    let uniswap_storage_slot = vec![b256!(
        "0000000000000000000000000000000000000000000000000000000000000008"
    )];

    let pool_address = address!("0xAE461cA67B15dc8dc81CE7615e0320dA1A9aB8D5");
    let provider = ProviderBuilder::new().on_http(reqwest::Url::from_str(rpc_url).unwrap());
    let latest = provider
        .get_block_by_number(rpc::types::BlockNumberOrTag::Latest)
        .await
        .unwrap()
        .unwrap();

    println!("latest block {}", latest.header.number);
    let mut buffer = Vec::<u8>::new();

    latest.header.inner.encode(&mut buffer);
    let header = keccak256(buffer);
    println!("{:0x} {:0x}", header, latest.header.hash);

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

    let inputs = VerifierInputs {
        header: LeanHeader::from(latest.header.inner),
        address: pool_address.to_vec(),
        storage_slot: uniswap_storage_slot.first().unwrap().to_vec(),
        proofs: Proofs {
            account_proof: account_collector,
            storage_proof: storage_collector,
        },
    };

    if !args.execute && !args.prove && !args.local {
        eprintln!("Error: You must specify either --execute or --prove or --local");
        std::process::exit(1);
    }

    let swap_payload = SwapInput {
        sell_amount: U256::from(5000000).to_be_bytes_vec(),
        sell_token0: false,
        sell_token: address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").to_vec(),
        buy_token: address!("0x6B175474E89094C44Da98b954EedeAC495271d0F").to_vec(),
        seller: address!("0xd4f23AfEAcfc05399E58e122B9a23cD04FA02C3B").to_vec(),
    };
    if args.local {
        let out = obsidian_lib::verifier::MPTVerifier::verify_slot(inputs).unwrap();
        println!("{:0x?}", out);
        let reserves_state: uni_v2::UniV2ReservesState =
            uni_v2::UniV2ReservesState::try_from(out.slot_data.clone()).unwrap();

        let swap_out = uni_v2_swapper::swap(reserves_state, swap_payload);
        println!("swap out {:0x?}", swap_out);
        // let order = pack_order(swap_out, block_number, out.block_hash);
    } else {
        let start = std::time::Instant::now();
        println!("Starting proof generation");
        let vm_input = ObsidianInput {
            swap_payload,
            block_verifier_inputs: inputs,
        };
        // setup vms
        let client = ProverClient::from_env();
        let mut stdin = SP1Stdin::new();
        stdin.write(&vm_input);

        if args.execute {
            // Execute the program
            let (output, report) = client.execute(OBSIDIAN_ELF, &stdin).run().unwrap();
            println!("Program executed successfully.");

            // Read the output.

            println!("{:0x?}", hex::encode(output.as_slice()));
            // Record the number of cycles executed.
            println!("Number of cycles: {}", report.total_instruction_count());
        } else {
            // Setup the program for proving.
            let (pk, vk) = client.setup(OBSIDIAN_ELF);
            println!("vk: {:?}", vk.bytes32());

            // Generate the proof
            let proof = client
                .prove(&pk, &stdin)
                .groth16()
                .run()
                .expect("failed to generate proof");

            println!("Successfully generated proof!");

            println!(
                "public values: 0x{}",
                hex::encode(proof.clone().public_values)
            );
            let solidity_proof = proof.bytes();
            println!("proof: 0x{}", hex::encode(solidity_proof));

            // Verify the proof.
            client.verify(&proof, &vk).expect("failed to verify proof");
            println!("Successfully verified proof!");

            println!("Proof generation completed in {:?}", start.elapsed());
        }
    }
}
