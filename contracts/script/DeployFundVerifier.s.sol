// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";

contract DeployFundVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);

        // The verifier will be in ../arecibo/FundLiquidityVerifier.sol
        // We'll need to copy it here first, then deploy

        console.log("===========================================");
        console.log("Deploying Fund Liquidity Verifier");
        console.log("===========================================");
        console.log("");
        console.log("NOTE: Copy FundLiquidityVerifier.sol from arecibo/ to contracts/src/ first!");
        console.log("");
        console.log("Then run:");
        console.log("  forge create src/FundLiquidityVerifier.sol:NovaDecider \\");
        console.log("    --rpc-url $ARC_RPC_URL \\");
        console.log("    --private-key $PRIVATE_KEY \\");
        console.log("    --legacy");
        console.log("");
        console.log("===========================================");

        vm.stopBroadcast();
    }
}
