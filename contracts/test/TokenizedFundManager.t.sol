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

        // Deploy mock verifier
        mockVerifier = new MockNovaDecider();

        // Deploy fund manager with mock verifier
        fundManager = new TokenizedFundManager(agent, whitelistRoot, address(mockVerifier));
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

        // Create mock proofs (non-empty)
        bytes memory positionProof = abi.encode("mock_position_proof");

        // Liquidity proof must be exactly 28 * 32 = 896 bytes (28 uint256 values)
        uint256[28] memory mockNovaProof;
        for (uint256 i = 0; i < 28; i++) {
            mockNovaProof[i] = i + 1; // Fill with dummy values
        }
        bytes memory liquidityProof = abi.encode(mockNovaProof);

        bytes memory whitelistProof = abi.encode("mock_whitelist_proof");

        bytes memory proofBundle = abi.encode(
            positionProof,
            liquidityProof,
            whitelistProof
        );

        bytes memory metadata = abi.encode("encrypted_transaction_data");

        bool success = fundManager.executeRebalance(proofBundle, metadata);
        assertTrue(success);

        assertEq(fundManager.getAuditTrailLength(), 1);
    }

    function testUnauthorizedAgentCannotRebalance() public {
        address unauthorizedAgent = address(0x2);
        vm.prank(unauthorizedAgent);

        bytes memory proofBundle = abi.encode("", "", "");
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

        bytes memory proofBundle = abi.encode("", "", ""); // Empty proofs
        bytes memory metadata = abi.encode("data");

        vm.expectRevert(TokenizedFundManager.ProofVerificationFailed.selector);
        fundManager.executeRebalance(proofBundle, metadata);
    }

    function testComplianceReport() public {
        vm.startPrank(agent);

        // Execute multiple rebalances with valid Nova proof format
        for (uint256 i = 0; i < 3; i++) {
            uint256[28] memory mockNovaProof;
            for (uint256 j = 0; j < 28; j++) {
                mockNovaProof[j] = i * 28 + j; // Unique values per iteration
            }

            bytes memory proofBundle = abi.encode(
                abi.encode("proof1"),
                abi.encode(mockNovaProof),
                abi.encode("proof3")
            );
            fundManager.executeRebalance(proofBundle, abi.encode("metadata", i));
        }

        vm.stopPrank();

        // Get compliance report
        TokenizedFundManager.Transaction[] memory report = fundManager.getComplianceReport(0, 2);
        assertEq(report.length, 3);
    }

    function testDailyRebalanceLimit() public {
        vm.startPrank(agent);

        // Create valid proof bundle
        uint256[28] memory mockNovaProof;
        for (uint256 i = 0; i < 28; i++) {
            mockNovaProof[i] = i + 1;
        }

        bytes memory proofBundle = abi.encode(
            abi.encode("proof1"),
            abi.encode(mockNovaProof),
            abi.encode("proof3")
        );

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
        // Create proof bundle with valid Nova proof format
        uint256[28] memory validNovaProof;
        for (uint256 i = 0; i < 28; i++) {
            validNovaProof[i] = i + 100; // Dummy values
        }

        bytes memory proofBundle = abi.encode(
            abi.encode("position_proof"),
            abi.encode(validNovaProof),
            abi.encode("whitelist_proof")
        );

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

    function testInvalidNovaProofLength() public {
        vm.prank(agent);

        // Create proof bundle with INVALID Nova proof (wrong size)
        bytes memory invalidLiquidityProof = abi.encode("wrong_size_proof"); // Too small

        bytes memory proofBundle = abi.encode(
            abi.encode("position_proof"),
            invalidLiquidityProof,
            abi.encode("whitelist_proof")
        );

        // Should revert with InvalidProof because size is wrong
        vm.expectRevert(TokenizedFundManager.InvalidProof.selector);
        fundManager.executeRebalance(proofBundle, abi.encode("metadata"));
    }
}
