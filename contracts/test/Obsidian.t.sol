// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "src/ObsidianRouter.sol";

contract MockSP1Verifier is ISP1Verifier {
    mapping(bytes32 => mapping(bytes => mapping(bytes => bool)))
        private _verifyResults;

    function setVerifyProofResult(
        bytes32 vkey,
        bytes memory publicValues,
        bytes memory proofBytes,
        bool result
    ) external {
        _verifyResults[vkey][publicValues][proofBytes] = result;
    }

    function verifyProof(
        bytes32 vkey,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external view override {
        require(
            _verifyResults[vkey][publicValues][proofBytes],
            "MockSP1Verifier: Verification failed"
        );
    }
}

contract ObsidianRouterTest is Test {
    address public constant SELLER = 0xd4f23AfEAcfc05399E58e122B9a23cD04FA02C3B;
    address public constant USDC = 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48;
    address public constant DAI = 0x6B175474E89094C44Da98b954EedeAC495271d0F;

    ObsidianRouter public router;
    MockSP1Verifier public verifier;
    address public buyer = address(0xBEEF);
    bytes32 public constant PROGRAM_VKEY = bytes32(uint256(0x123));

    function testMainnetFork() public {
        uint256 mainnetFork = vm.createFork(vm.envString("MAINNET_RPC_URL"));
        vm.selectFork(mainnetFork);

        bytes
            memory publicValues = hex"000000000000000000000000d4f23afeacfc05399e58e122b9a23cd04fa02c3b4e6d8a6fb809ec5b8d785acadd3a04ad542b55363b24d1c3fea5dc87a298f3a90000000000000000000000000000000000000000000000000000000001513ce80000000000000000000000000000000000000000000000004553022b078fae8c00000000000000000000000000000000000000000000000000000000004c4b400000000000000000000000006b175474e89094c44da98b954eedeac495271d0f000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
        bytes
            memory proofBytes = hex"11b6a09d14d5ba6b75d0b7c68da3d6a9757a4aee495e6a61ff914cff527f6794c35c84d00ef6b65ccd43ed859f3312806f2b8fa4b8056e83467ceb600af0e0351761d2632ac586f78260a69a5a25dd3f73ded784751100019a3e86f9072dbaa5e5c1edc308296f61ad25c26aefa141ee6bec8db9a98cada26e74a6c4e782cc5f35d185bc27a30de9fa223c175686055a8317c506d4f90c5e0744642dda56653206aa37dd04eb0c553b4c3a8e27f8e0bb28853cdd0b4d8f257e80b2526f823bbc3239e1f501e99ff1840d04ac537a3e3b0f70a4400f91e198bdbb029c4b5acf460e607c531d196f3da9b8a9328e7ef9a77ad3613de0094a60196da3d33350ac6a946818aa";

        ObsidianOrder memory order = abi.decode(publicValues, (ObsidianOrder));

        bytes32 vkey = hex"0087e1ea649d141c1b62fc8155f7df892a3ef29b9744fe46ded393fab786638b";

        uint256 sellerPrivateKey = vm.envUint("SELLER_PRIVATE_KEY");
        address sellerAddress = order.seller;

        router = new ObsidianRouter(
            address(0x397A5f7f3dBd538f23DE225B51f532c34448dA9B),
            vkey,
            sellerAddress
        );

        address derivedAddress = vm.addr(sellerPrivateKey);
        require(
            derivedAddress == sellerAddress,
            "private key does not match seller address"
        );

        bytes32 digest = router.getOrderHash(
            order.block_number,
            order.sold_amount,
            order.buy_token,
            order.sell_token
        );
        console.log(order.block_number);

        (uint8 v, bytes32 r, bytes32 s) = vm.sign(sellerPrivateKey, digest);
        bytes memory signature = abi.encodePacked(r, s, v);

        deal(DAI, buyer, 1000000e18);
        deal(USDC, order.seller, 500000000e6);

        vm.startPrank(buyer);
        IERC20(DAI).approve(address(router), type(uint256).max);
        vm.stopPrank();

        vm.startPrank(order.seller);
        IERC20(USDC).approve(address(router), type(uint256).max);
        vm.stopPrank();

        console.log("========== Initial Balances ==========");
        console.log("Buyer DAI:", IERC20(DAI).balanceOf(buyer));
        console.log("Buyer USDC:", IERC20(USDC).balanceOf(buyer));
        console.log("Seller DAI:", IERC20(DAI).balanceOf(order.seller));
        console.log("Seller USDC:", IERC20(USDC).balanceOf(order.seller));

        // vm.prank(sellerAddress);
        // router.setVerifyBlock(true);
        // vm.stopPrank();

        vm.prank(buyer);
        router.solve(publicValues, proofBytes, signature);

        console.log("========== Final Balances ==========");
        console.log("Buyer DAI:", IERC20(DAI).balanceOf(buyer));
        console.log("Buyer USDC:", IERC20(USDC).balanceOf(buyer));
        console.log("Seller DAI:", IERC20(DAI).balanceOf(order.seller));
        console.log("Seller USDC:", IERC20(USDC).balanceOf(order.seller));
    }
}
