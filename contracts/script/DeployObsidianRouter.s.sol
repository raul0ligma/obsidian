// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/ObsidianRouter.sol";
import {console} from "forge-std/console.sol";

contract DeployObsidianRouter is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);

        address verifier = address(0x397A5f7f3dBd538f23DE225B51f532c34448dA9B);

        bytes32 obsidianProgramVKey = bytes32(
            0x0087e1ea649d141c1b62fc8155f7df892a3ef29b9744fe46ded393fab786638b
        );

        address owner = vm.addr(deployerPrivateKey);

        ObsidianRouter obsidianRouter = new ObsidianRouter(
            verifier,
            obsidianProgramVKey,
            owner
        );

        console.log("ObsidianRouter deployed at:", address(obsidianRouter));

        obsidianRouter.setVerifyBlock(false);
        console.log("VerifyBlock state set to: false");

        vm.stopBroadcast();
    }
}
