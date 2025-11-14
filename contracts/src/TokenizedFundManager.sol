// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title NovaDecider Interface
/// @notice Interface for the deployed NovaDecider verifier contract
interface INovaDecider {
    function verifyOpaqueNovaProof(uint256[28] calldata proof) external view returns (bool);
}

/// @title Tokenized Fund Manager with Zero-Knowledge Proofs
/// @notice Manages tokenized RWA fund with privacy-preserving compliance proofs
/// @dev Uses Nova/Sonobe ZK proofs for fund policy enforcement
contract TokenizedFundManager {
    /*//////////////////////////////////////////////////////////////
                                 ERRORS
    //////////////////////////////////////////////////////////////*/

    error Unauthorized();
    error ProofVerificationFailed();
    error InvalidProof();
    error InsufficientBalance();
    error PolicyViolation(string reason);

    /*//////////////////////////////////////////////////////////////
                                 EVENTS
    //////////////////////////////////////////////////////////////*/

    event RebalanceExecuted(
        bytes32 indexed txHash,
        uint256 timestamp,
        bytes32 proofCommitment
    );

    event PolicyUpdated(string parameter, uint256 newValue);

    event AgentAuthorized(address indexed agent, bool status);

    /*//////////////////////////////////////////////////////////////
                                STORAGE
    //////////////////////////////////////////////////////////////*/

    /// Fund policy parameters (public, transparent)
    uint256 public constant MAX_SINGLE_POSITION = 40; // 40%
    uint256 public constant MIN_LIQUIDITY = 10;       // 10%

    /// State commitments (private)
    bytes32 public assetWhitelistRoot;

    /// Authorized agents
    mapping(address => bool) public authorizedAgents;

    /// Fund admin
    address public immutable admin;

    /// NovaDecider verifier contract (single folded proof)
    INovaDecider public immutable novaVerifier;

    /// Audit trail
    struct Transaction {
        bytes32 txHash;
        uint256 timestamp;
        bytes32 proofCommitment;
    }

    Transaction[] public auditTrail;

    /// Daily tracking (for rate limits)
    uint256 public currentDay;
    uint256 public dailyRebalanceCount;
    uint256 public constant MAX_DAILY_REBALANCES = 10;

    /*//////////////////////////////////////////////////////////////
                              CONSTRUCTOR
    //////////////////////////////////////////////////////////////*/

    constructor(
        address _agent,
        bytes32 _whitelistRoot,
        address _novaVerifier
    ) {
        admin = msg.sender;
        authorizedAgents[_agent] = true;
        assetWhitelistRoot = _whitelistRoot;
        novaVerifier = INovaDecider(_novaVerifier);

        emit AgentAuthorized(_agent, true);
    }

    /*//////////////////////////////////////////////////////////////
                               MODIFIERS
    //////////////////////////////////////////////////////////////*/

    modifier onlyAdmin() {
        if (msg.sender != admin) revert Unauthorized();
        _;
    }

    modifier onlyAgent() {
        if (!authorizedAgents[msg.sender]) revert Unauthorized();
        _;
    }

    /*//////////////////////////////////////////////////////////////
                            CORE FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /// @notice Execute a fund rebalancing with ZK proof verification
    /// @param foldedProof Single folded Nova proof attesting to all compliance checks
    /// @param metadata Encrypted transaction metadata
    /// @return success Whether the rebalancing was executed
    function executeRebalance(
        bytes calldata foldedProof,
        bytes calldata metadata
    ) external onlyAgent returns (bool success) {
        // Update daily tracking
        _updateDailyTracking();

        // Check rate limit
        if (dailyRebalanceCount >= MAX_DAILY_REBALANCES) {
            revert PolicyViolation("Daily rebalance limit exceeded");
        }

        // Verify folded proof once (all constraints folded together)
        _verifyFoldedProof(foldedProof);

        // If all proofs valid, record the transaction
        bytes32 txHash = keccak256(
            abi.encodePacked(metadata, block.timestamp, msg.sender)
        );

        auditTrail.push(
            Transaction({
                txHash: txHash,
                timestamp: block.timestamp,
                proofCommitment: keccak256(foldedProof)
            })
        );

        dailyRebalanceCount++;

        emit RebalanceExecuted(txHash, block.timestamp, keccak256(foldedProof));

        return true;
    }

    /// @notice Generate a compliance report for auditors
    /// @param fromIndex Starting index in audit trail
    /// @param toIndex Ending index in audit trail
    /// @return transactions Array of transactions in the range
    function getComplianceReport(uint256 fromIndex, uint256 toIndex)
        external
        view
        returns (Transaction[] memory transactions)
    {
        require(fromIndex <= toIndex, "Invalid range");
        require(toIndex < auditTrail.length, "Index out of bounds");

        uint256 length = toIndex - fromIndex + 1;
        transactions = new Transaction[](length);

        for (uint256 i = 0; i < length; i++) {
            transactions[i] = auditTrail[fromIndex + i];
        }

        return transactions;
    }

    /*//////////////////////////////////////////////////////////////
                          PROOF VERIFICATION
    //////////////////////////////////////////////////////////////*/

    /// @notice Verify folded Nova proof (all constraints)
    /// @dev Calls the deployed NovaDecider verifier contract once
    function _verifyFoldedProof(bytes memory proof) internal view {
        if (proof.length == 0) {
            revert ProofVerificationFailed();
        }

        // For Nova proofs, we expect exactly 28 uint256 values (900 bytes when ABI-encoded)
        // The proof should be pre-formatted as uint256[28]
        if (proof.length != 28 * 32) {
            revert InvalidProof();
        }

        // Decode and verify
        uint256[28] memory novaProof = abi.decode(proof, (uint256[28]));
        bool verified = novaVerifier.verifyOpaqueNovaProof(novaProof);

        if (!verified) {
            revert ProofVerificationFailed();
        }
    }

    /*//////////////////////////////////////////////////////////////
                           ADMIN FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /// @notice Authorize or deauthorize an agent
    function setAgentAuthorization(address agent, bool status)
        external
        onlyAdmin
    {
        authorizedAgents[agent] = status;
        emit AgentAuthorized(agent, status);
    }

    /// @notice Update asset whitelist root
    function updateWhitelistRoot(bytes32 newRoot) external onlyAdmin {
        assetWhitelistRoot = newRoot;
        emit PolicyUpdated("whitelistRoot", uint256(newRoot));
    }

    /*//////////////////////////////////////////////////////////////
                          INTERNAL HELPERS
    //////////////////////////////////////////////////////////////*/

    function _updateDailyTracking() private {
        uint256 today = block.timestamp / 1 days;
        if (today > currentDay) {
            currentDay = today;
            dailyRebalanceCount = 0;
        }
    }

    /*//////////////////////////////////////////////////////////////
                              VIEW FUNCTIONS
    //////////////////////////////////////////////////////////////*/

    /// @notice Get total number of transactions in audit trail
    function getAuditTrailLength() external view returns (uint256) {
        return auditTrail.length;
    }

    /// @notice Get fund policy parameters
    function getPolicyParameters()
        external
        pure
        returns (uint256 maxPosition, uint256 minLiquidity)
    {
        return (MAX_SINGLE_POSITION, MIN_LIQUIDITY);
    }
}
