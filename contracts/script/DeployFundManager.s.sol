// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/TokenizedFundManager.sol";

contract DeployFundManager is Script {
    function run() external {
        // Get deployer private key from environment
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        // Get initial agent address (can be deployer for testing)
        address initialAgent = vm.envOr("INITIAL_AGENT", vm.addr(deployerPrivateKey));

        // Example whitelist root (mock for now)
        bytes32 whitelistRoot = keccak256("arc_approved_assets_v1");

        vm.startBroadcast(deployerPrivateKey);

        // Deploy TokenizedFundManager
        TokenizedFundManager fundManager = new TokenizedFundManager(
            initialAgent,
            whitelistRoot
        );

        console.log("===========================================");
        console.log("TokenizedFundManager deployed!");
        console.log("===========================================");
        console.log("Contract address:", address(fundManager));
        console.log("Admin:", fundManager.admin());
        console.log("Initial agent:", initialAgent);
        console.log("Authorized:", fundManager.authorizedAgents(initialAgent));
        console.log("Max position:", fundManager.MAX_SINGLE_POSITION(), "%");
        console.log("Min liquidity:", fundManager.MIN_LIQUIDITY(), "%");
        console.log("Whitelist root:", vm.toString(whitelistRoot));
        console.log("===========================================");

        vm.stopBroadcast();
    }
}
