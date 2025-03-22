// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";
import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";
import {console} from "forge-std/console.sol";

struct ObsidianOrder {
    address seller;
    bytes32 block_hash;
    uint256 block_number;
    uint256 bought_amount;
    uint256 sold_amount;
    address buy_token;
    address sell_token;
}

contract ObsidianRouter {
    address public verifier;
    bytes32 public obsidianProgramVKey;

    // Events
    event OrderExecuted(
        address indexed buyer,
        address indexed seller,
        address buyToken,
        address sellToken,
        uint256 boughtAmount,
        uint256 soldAmount,
        bytes32 blockHash,
        uint256 blockNumber
    );

    constructor(address _verifier, bytes32 _obsidianProgramVKey) {
        verifier = _verifier;
        obsidianProgramVKey = _obsidianProgramVKey;
    }

    function solve(
        bytes calldata _publicValues,
        bytes calldata _proofBytes
    ) external {
        ISP1Verifier(verifier).verifyProof(
            obsidianProgramVKey,
            _publicValues,
            _proofBytes
        );

        ObsidianOrder memory order = abi.decode(_publicValues, (ObsidianOrder));

        bytes32 actualBlockHash = blockhash(order.block_number);
        require(actualBlockHash != bytes32(0), "Block is too old");
        require(
            actualBlockHash == order.block_hash,
            "Block hash does not match"
        );

        require(
            IERC20(order.buy_token).transferFrom(
                msg.sender,
                order.seller,
                order.bought_amount
            ),
            "Buy token transfer failed: Check allowance and balances"
        );

        require(
            IERC20(order.sell_token).transferFrom(
                order.seller,
                msg.sender,
                order.sold_amount
            ),
            "Sell token transfer failed: Check allowance and balances"
        );

        emit OrderExecuted(
            msg.sender,
            order.seller,
            order.buy_token,
            order.sell_token,
            order.bought_amount,
            order.sold_amount,
            order.block_hash,
            order.block_number
        );
    }
}
