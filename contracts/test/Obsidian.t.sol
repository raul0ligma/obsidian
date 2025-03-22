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
    address public constant ROUTER_ADDRESS =
        0xEe395f9489bdC1b74A1BFc3B3164a5FEFb7146AE;

    ObsidianRouter public router;
    MockSP1Verifier public verifier;
    address public buyer = address(0xBEEF);
    bytes32 public constant PROGRAM_VKEY = bytes32(uint256(0x123));

    function testMainnetFork() public {
        uint256 mainnetFork = vm.createFork(vm.envString("MAINNET_RPC_URL"));
        vm.selectFork(mainnetFork);

        bytes
            memory publicValues = hex"000000000000000000000000befe5e6df5f3e4cba02b11ba233f6584f295b96d0e653461bc96391383c7ed60c6aca044ec9533eb3332e7a1dfbfcdd887d2b9500000000000000000000000000000000000000000000000000000000001aa35fc0000000000000000000000000000000000000000000000000001c8df78cc77c200000000000000000000000000000000000000000000000000000000000f42400000000000000000000000004200000000000000000000000000000000000006000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda02913";
        bytes
            memory proofBytes = hex"11b6a09d107bce4a042087be1bd27441006bc5ce3fb670654357fa7060e5ed2f6324c75d1047db20bee01862b1bd21eec259085af25313d2d0aff2d32ef727f4ebe3bede1314b4197c902662ff1d399c447538caf56515a505f797ec4c6d876d430af93116bda1aff1006a3cb7cd5dbe4a8b740cb51ca4e60c71190c0caefaebe8d6fb43011869007658d4795a3cad5a1bad7984ca7efa4788a30022bdc159ff265b690d19c203e5346780ca8ee020ccf9cd2836868cee110991ddeb1eb5e4b03e1b8cc606a9f3d9af567b68252da2b339c347c1cfe15b697ece45b9ebbc26043b2fb7b20a5f30ef551089cc0cc3a36800d2557297da9cb00a20a2ffa29ef436be292fd0";

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

        // deal(DAI, buyer, 1000000e18);
        // deal(USDC, order.seller, 500000000e6);

        // vm.startPrank(buyer);
        // IERC20(DAI).approve(address(router), type(uint256).max);
        // vm.stopPrank();

        // vm.startPrank(order.seller);
        // IERC20(USDC).approve(address(router), type(uint256).max);
        // vm.stopPrank();

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
    function testPreDeployedRouterWithSender() public {
        uint256 mainnetFork = vm.createFork(vm.envString("MAINNET_RPC_URL"));
        vm.selectFork(mainnetFork);

        // Connect to pre-deployed router
        router = ObsidianRouter(ROUTER_ADDRESS);

        // Public values and proof from the original test
        bytes
            memory publicValues = hex"000000000000000000000000befe5e6df5f3e4cba02b11ba233f6584f295b96d0e653461bc96391383c7ed60c6aca044ec9533eb3332e7a1dfbfcdd887d2b9500000000000000000000000000000000000000000000000000000000001aa35fc0000000000000000000000000000000000000000000000000001c8df78cc77c200000000000000000000000000000000000000000000000000000000000f42400000000000000000000000004200000000000000000000000000000000000006000000000000000000000000833589fcd6edb6e08f4c7c32d4f71b54bda02913";
        bytes
            memory proofBytes = hex"11b6a09d107bce4a042087be1bd27441006bc5ce3fb670654357fa7060e5ed2f6324c75d1047db20bee01862b1bd21eec259085af25313d2d0aff2d32ef727f4ebe3bede1314b4197c902662ff1d399c447538caf56515a505f797ec4c6d876d430af93116bda1aff1006a3cb7cd5dbe4a8b740cb51ca4e60c71190c0caefaebe8d6fb43011869007658d4795a3cad5a1bad7984ca7efa4788a30022bdc159ff265b690d19c203e5346780ca8ee020ccf9cd2836868cee110991ddeb1eb5e4b03e1b8cc606a9f3d9af567b68252da2b339c347c1cfe15b697ece45b9ebbc26043b2fb7b20a5f30ef551089cc0cc3a36800d2557297da9cb00a20a2ffa29ef436be292fd0";

        // Signature already generated
        bytes
            memory signature = hex"edcdd53460eb1778d6a3fd0c6acd991f32a6283f7432aa4be8f8c5e527d6a36b28067b98078a96aa95c29339596bdecd79b53553c46f4d832101b04295f82d5b1b"; // Replace with actual signature

        address sender = 0xd4f23AfEAcfc05399E58e122B9a23cD04FA02C3B;

        ObsidianOrder memory order = abi.decode(publicValues, (ObsidianOrder));

        // Log initial balances
        // console.log("========== Initial Balances ==========");
        // console.log("Sender DAI:", IERC20(DAI).balanceOf(sender));
        // console.log("Sender USDC:", IERC20(USDC).balanceOf(sender));
        // console.log("Seller DAI:", IERC20(DAI).balanceOf(order.seller));
        // console.log("Seller USDC:", IERC20(USDC).balanceOf(order.seller));

        // Optional: Fund and approve for sender if needed
        // deal(DAI, sender, 1000000e18);
        // vm.prank(sender);
        // IERC20(DAI).approve(address(router), type(uint256).max);

        bytes memory generatedCalldata = abi.encodeWithSelector(
            router.solve.selector,
            publicValues,
            proofBytes,
            signature
        );

        console.log("Generated calldata:");
        console.logBytes(generatedCalldata);
        console.log("Generated calldata length:", generatedCalldata.length);

        // Execute the transaction as the specified sender
        vm.prank(sender);
        router.solve(publicValues, proofBytes, signature);

        // Log final balances
        // console.log("========== Final Balances ==========");
        // console.log("Sender DAI:", IERC20(DAI).balanceOf(sender));
        // console.log("Sender USDC:", IERC20(USDC).balanceOf(sender));
        // console.log("Seller DAI:", IERC20(DAI).balanceOf(order.seller));
        // console.log("Seller USDC:", IERC20(USDC).balanceOf(order.seller));
    }
}
