// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";
import {IERC20} from "@openzeppelin/token/ERC20/IERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {console} from "forge-std/console.sol";
import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
import {EIP712} from "@openzeppelin/contracts/utils/cryptography/EIP712.sol";

struct ObsidianOrder {
    address seller;
    bytes32 block_hash;
    uint256 block_number;
    uint256 bought_amount;
    uint256 sold_amount;
    address buy_token;
    address sell_token;
}

contract ObsidianRouter is Ownable, EIP712 {
    address public verifier;
    bytes32 public obsidianProgramVKey;
    bool public verifyBlock;

    using ECDSA for bytes32;

    // ObsidianOrder hash
    bytes32 private constant ORDER_TYPEHASH =
        keccak256(
            "ObsidianOrder(uint256 blockNumber,uint256 sellAmount,address buyToken,address sellToken)"
        );

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

    constructor(
        address _verifier,
        bytes32 _obsidianProgramVKey,
        address owner
    ) Ownable(owner) EIP712("ObsidianRouter", "1") {
        verifier = _verifier;
        obsidianProgramVKey = _obsidianProgramVKey;
    }

    function setVerifyBlock(bool state) public onlyOwner {
        verifyBlock = state;
    }

    /**
     * @dev creates a hash of the order signed by seller
     * @param blockNumber agreed block number of the order used for pricing oracle
     * @param sellAmount amount being sold
     * @param sellToken address of the token being sold
     * @param buyToken address of the token being bought
     */
    function getOrderHash(
        uint256 blockNumber,
        uint256 sellAmount,
        address buyToken,
        address sellToken
    ) public view returns (bytes32) {
        bytes32 structHash = keccak256(
            abi.encode(
                ORDER_TYPEHASH,
                blockNumber,
                sellAmount,
                buyToken,
                sellToken
            )
        );
        return _hashTypedDataV4(structHash);
    }

    /**
     * @dev Verifies that the signature is valid for the given order parameters using EIP-712
     */
    function verifySignature(
        ObsidianOrder memory order,
        bytes memory signature
    ) internal view returns (bool) {
        bytes32 orderHash = getOrderHash(
            order.block_number,
            order.sold_amount,
            order.buy_token,
            order.sell_token
        );

        address recoveredSigner = ECDSA.recover(orderHash, signature);

        return recoveredSigner == order.seller;
    }

    function solve(
        bytes calldata _publicValues,
        bytes calldata _proofBytes,
        bytes calldata _orderSignature
    ) external {
        ObsidianOrder memory order = abi.decode(_publicValues, (ObsidianOrder));

        require(
            verifySignature(order, _orderSignature),
            "invalid order signature"
        );

        ISP1Verifier(verifier).verifyProof(
            obsidianProgramVKey,
            _publicValues,
            _proofBytes
        );

        if (verifyBlock) {
            bytes32 actualBlockHash = blockhash(order.block_number);
            require(actualBlockHash != bytes32(0), "block is too old");
            require(
                actualBlockHash == order.block_hash,
                "block hash does not match"
            );
        }

        require(
            IERC20(order.buy_token).transferFrom(
                msg.sender,
                order.seller,
                order.bought_amount
            ),
            "buy token transfer failed: Check allowance and balances"
        );

        require(
            IERC20(order.sell_token).transferFrom(
                order.seller,
                msg.sender,
                order.sold_amount
            ),
            "sell token transfer failed: Check allowance and balances"
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
