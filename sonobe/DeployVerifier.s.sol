// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Script.sol";

interface NovaDecider {
    function verifyNovaProof(uint256[28] calldata proof) external view returns (bool);
}

contract DeployVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");

        vm.startBroadcast(deployerPrivateKey);

        // Read the compiled verifier bytecode
        string memory verifierCode = vm.readFile("CompositeFundVerifier.sol");

        // Deploy using create
        address verifier = deployCode("CompositeFundVerifier.sol:NovaDecider");

        vm.stopBroadcast();

        console.log("Verifier deployed at:", verifier);
    }
}
