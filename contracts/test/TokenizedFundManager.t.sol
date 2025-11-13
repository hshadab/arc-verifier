// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/TokenizedFundManager.sol";

contract TokenizedFundManagerTest is Test {
    TokenizedFundManager public fundManager;
    address public admin;
    address public agent;
    bytes32 public whitelistRoot;

    function setUp() public {
        admin = address(this);
        agent = address(0x1);
        whitelistRoot = keccak256("mock_whitelist_root");

        fundManager = new TokenizedFundManager(agent, whitelistRoot);
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
        bytes memory liquidityProof = abi.encode("mock_liquidity_proof");
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

        // Execute multiple rebalances
        for (uint256 i = 0; i < 3; i++) {
            bytes memory proofBundle = abi.encode(
                abi.encode("proof1"),
                abi.encode("proof2"),
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

        bytes memory proofBundle = abi.encode(
            abi.encode("proof1"),
            abi.encode("proof2"),
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
}
