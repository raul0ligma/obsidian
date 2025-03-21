// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

struct ObsidianOrder {
    address seller;
    bytes32 block_hash;
    uint256 block_number;
    uint256 bought_amount;
    uint256 sold_amount;
    address buy_token;
    address sell_token;
}

contract Fibonacci {
    address public verifier;

    bytes32 public obsidianProgramVKey;

    constructor(address _verifier, bytes32 _obsidianProgramVKey) {
        verifier = _verifier;
        obsidianProgramVKey = _obsidianProgramVKey;
    }

    function solve(bytes calldata _publicValues, bytes calldata _proofBytes) {
        ISP1Verifier(verifier).verifyProof(
            fibonacciProgramVKey,
            _publicValues,
            _proofBytes
        );
        ObsidianOrder memory order = abi.decode(_publicValues, (ObsidianOrder));
    }
}
