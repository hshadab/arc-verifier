// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/TokenizedFundManager.sol";

/// @notice Mock NovaDecider for testing
contract MockNovaDecider {
    bool public shouldPass = true;

    function setShouldPass(bool _shouldPass) external {
        shouldPass = _shouldPass;
    }

    function verifyOpaqueNovaProof(uint256[28] calldata) external view returns (bool) {
        return shouldPass;
    }
}

contract TokenizedFundManagerTest is Test {
    TokenizedFundManager public fundManager;
    MockNovaDecider public mockVerifier;
    address public admin;
    address public agent;
    bytes32 public whitelistRoot;

    function setUp() public {
        admin = address(this);
        agent = address(0x1);
        whitelistRoot = keccak256("mock_whitelist_root");

        // Deploy mock verifier and fund manager (single folded proof)
        mockVerifier = new MockNovaDecider();
        fundManager = new TokenizedFundManager(
            agent,
            whitelistRoot,
            address(mockVerifier)
        );
    }

    function testInitialSetup() public {
        assertEq(fundManager.admin(), admin);
        assertTrue(fundManager.authorizedAgents(agent));
        assertEq(fundManager.assetWhitelistRoot(), whitelistRoot);
    }

    function testGetPolicyParameters() public {
        (uint256 maxPosition, uint256 minLiquidity) = fundManager.getPolicyParameters();
        assertEq(maxPosition, 40);
        assertEq(minLiquidity, 10);
    }

    function testExecuteRebalanceWithMockProofs() public {
        vm.prank(agent);

        // Create folded proof (28 * 32 bytes, encoded as uint256[28])
        uint256[28] memory folded;
        for (uint256 i = 0; i < 28; i++) {
            folded[i] = i + 1;
        }
        bytes memory proofBundle = abi.encode(folded);

        bytes memory metadata = abi.encode("encrypted_transaction_data");

        bool success = fundManager.executeRebalance(proofBundle, metadata);
        assertTrue(success);

        assertEq(fundManager.getAuditTrailLength(), 1);
    }

    function testUnauthorizedAgentCannotRebalance() public {
        address unauthorizedAgent = address(0x2);
        vm.prank(unauthorizedAgent);

        bytes memory proofBundle = new bytes(0);
        bytes memory metadata = abi.encode("data");

        vm.expectRevert(TokenizedFundManager.Unauthorized.selector);
        fundManager.executeRebalance(proofBundle, metadata);
    }

    function testAdminCanAuthorizeAgent() public {
        address newAgent = address(0x3);

        fundManager.setAgentAuthorization(newAgent, true);
        assertTrue(fundManager.authorizedAgents(newAgent));

        fundManager.setAgentAuthorization(newAgent, false);
        assertFalse(fundManager.authorizedAgents(newAgent));
    }

    function testEmptyProofFails() public {
        vm.prank(agent);

        bytes memory proofBundle = new bytes(0); // Empty folded proof
        bytes memory metadata = abi.encode("data");

        vm.expectRevert(TokenizedFundManager.ProofVerificationFailed.selector);
        fundManager.executeRebalance(proofBundle, metadata);
    }

    function testComplianceReport() public {
        vm.startPrank(agent);

        // Execute multiple rebalances with valid folded proof
        for (uint256 i = 0; i < 3; i++) {
            uint256[28] memory folded2;
            for (uint256 j = 0; j < 28; j++) {
                folded2[j] = 100 + i * 28 + j;
            }
            bytes memory proofBundle2 = abi.encode(folded2);
            fundManager.executeRebalance(proofBundle2, abi.encode("metadata", i));
        }

        vm.stopPrank();

        // Get compliance report
        TokenizedFundManager.Transaction[] memory report = fundManager.getComplianceReport(0, 2);
        assertEq(report.length, 3);
    }

    function testDailyRebalanceLimit() public {
        vm.startPrank(agent);

        // Create valid folded proof
        uint256[28] memory folded3;
        for (uint256 i = 0; i < 28; i++) {
            folded3[i] = i + 1;
        }
        bytes memory proofBundle = abi.encode(folded3);

        // Execute up to limit
        for (uint256 i = 0; i < 10; i++) {
            fundManager.executeRebalance(proofBundle, abi.encode("metadata", i));
        }

        // 11th should fail
        vm.expectRevert(
            abi.encodeWithSelector(
                TokenizedFundManager.PolicyViolation.selector,
                "Daily rebalance limit exceeded"
            )
        );
        fundManager.executeRebalance(proofBundle, abi.encode("metadata", 11));

        vm.stopPrank();
    }

    function testNovaVerifierIntegration() public {
        // Create folded proof with valid Nova proof format (28 uint256 values)
        uint256[28] memory validNovaProof;
        for (uint256 i = 0; i < 28; i++) {
            validNovaProof[i] = i + 100; // Dummy values
        }

        bytes memory proofBundle = abi.encode(validNovaProof);

        // Should succeed with mock verifier returning true
        vm.prank(agent);
        bool success = fundManager.executeRebalance(proofBundle, abi.encode("metadata"));
        assertTrue(success);

        // Now make verifier fail
        mockVerifier.setShouldPass(false);

        // Should revert with ProofVerificationFailed
        vm.prank(agent);
        vm.expectRevert(TokenizedFundManager.ProofVerificationFailed.selector);
        fundManager.executeRebalance(proofBundle, abi.encode("metadata2"));
    }

    // Position/Whitelist verifier integration now folded into a single proof path

    function testInvalidNovaProofLength() public {
        vm.prank(agent);

        // Create folded proof with INVALID size
        bytes memory proofBundle = abi.encode("wrong_size_proof"); // Too small

        // Should revert with InvalidProof because size is wrong
        vm.expectRevert(TokenizedFundManager.InvalidProof.selector);
        fundManager.executeRebalance(proofBundle, abi.encode("metadata"));
    }

    // Invalid length for folded proof already tested above
}
