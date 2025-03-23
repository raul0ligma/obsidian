# 🟪 obsidian

A zero-knowledge proof-based system for verifying and executing computations based on historical Ethereum state proofs, built with Rust and the SP1 zkVM.

## Overview

Obsidian is a system that verifies Ethereum state proofs inside a zkVM to enable trustless execution of computations against historical blockchain states. The project uses historical block hashes using MPT to prove the authenticity of past states, execution should be done within 256 blocks of proving.

## Key Features

- Historical state verification using MPT proofs
- Trustless execution of computations on verified state
- Zero-knowledge proof generation via SP1 zkVM
- Efficient RLP decoding for minimal proof verification

## Repository Structure

```
obsidian/
├── lib/            # Core shared functionality
├── program/        # zkVM execution program
└── server/         # API and proof generation service
```

## Components

### Library (`lib/`)

The core library provides shared functionality across the project:

- **verifier/**: MPT (Merkle Patricia Tree) verification logic
  - Verifies Ethereum state roots inside the zkVM
  - Implements account state verification (balance, nonce, code hash, storage root)
  - Handles storage slot verification for contract state
  - Supports three node types: branch, extension, and leaf
  - Processes both account proofs and storage proofs
- **swapper/**: Uniswap V2 swap execution logic
- **states/**: State management for Uniswap V2 reserves
- **header/**: Block header processing
- **decoder/**: Lightweight RLP decoder for MPT parsing
  - Minimal implementation focused on MPT node decoding
  - Handles basic RLP string decoding cases
  - Supports branch, extension, and leaf node parsing
  - Uses simple nibble-based prefix handling

### Program (`program/`)

The on-chain program runs in the SP1 zkVM and:

- Verifies block data using MPT proofs
  - Validates state roots against provided proofs
  - Verifies account states and storage slots
- Executes Uniswap V2 swaps
- Generates verifiable order outputs
- Handles state transitions for Uniswap V2 reserves

### Server (`server/`)

Backend service providing:

- REST API endpoints for system interaction
- Proof generation service
- Integration with the SP1 zkVM

## Getting Started

### Prerequisites

- Rust toolchain (specified in `rust-toolchain`)
- [SP1 zkVM SDK](https://github.com/succinctlabs/sp1)
- Node.js (for frontend development)

## Usage

```bash
# Run the prover
RUST_LOG=info cargo run --release --bin prover

# Run the server
RUST_LOG=info cargo run --release --bin server
```

## Documentation

For detailed documentation on the API and components, see the [docs](./docs) directory.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [SP1 zkVM](https://github.com/succinctlabs/sp1)
