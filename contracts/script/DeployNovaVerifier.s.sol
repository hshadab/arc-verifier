// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/FundLiquidityVerifier.sol";

contract DeployNovaVerifier is Script {
    function run() external {
        vm.startBroadcast();

        NovaDecider verifier = new NovaDecider();

        console.log("====================================");
        console.log("NovaDecider Verifier Deployed!");
        console.log("====================================");
        console.log("Contract address:", address(verifier));
        console.log("Network: Arc Testnet");
        console.log("Chain ID:", block.chainid);
        console.log("Deployer:", msg.sender);
        console.log("====================================");

        vm.stopBroadcast();
    }
}
