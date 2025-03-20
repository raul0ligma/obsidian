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
use alloy_rlp::Encodable;
use alloy_sol_types::SolType;
use clap::Parser;
use obsidian_lib::{
    decoder::NodeDecoder,
    header::LeanHeader,
    states::uni_v2,
    verifier::{Node, Proofs, VerifierInputs},
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

    let pool_address = address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc");
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

    if args.local {
        let out = obsidian_lib::verifier::MPTVerifier::verify_slot(inputs);
        println!("{:0x?}", out)
    } else {
        // setup vms
        let client = ProverClient::from_env();
        let mut stdin = SP1Stdin::new();
        stdin.write(&inputs);

        if args.execute {
            // Execute the program
            let (output, report) = client.execute(OBSIDIAN_ELF, &stdin).run().unwrap();
            println!("Program executed successfully.");

            // Read the output.

            println!("{:0x?}", output);
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

            println!("{:0x?}", proof.public_values);
            let solidity_proof = proof.bytes();
            println!("proof: 0x{}", hex::encode(solidity_proof));
            proof
                .save("obsidian-groth16.bin")
                .expect("saving proof failed");

            // Verify the proof.
            client.verify(&proof, &vk).expect("failed to verify proof");
            println!("Successfully verified proof!");
        }
    }
}
