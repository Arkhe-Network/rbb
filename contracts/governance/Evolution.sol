// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Evolution
 * @dev The Constitutional Evolution contract for Cathedral AGI.
 * Allows proposing and voting on constitutional changes.
 */
contract Evolution {
    address public constitutionContract;

    struct Proposal {
        uint256 id;
        string description;
        uint256 targetBatchSize;
        uint256 votes;
        bool executed;
    }

    uint256 public proposalCount;
    mapping(uint256 => Proposal) public proposals;
    mapping(uint256 => mapping(address => bool)) public hasVoted;

    event ProposalCreated(uint256 id, string description, uint256 targetBatchSize);
    event Voted(uint256 proposalId, address voter);
    event ProposalExecuted(uint256 id);

    constructor(address _constitutionContract) {
        constitutionContract = _constitutionContract;
    }

    function proposeChange(string memory description, uint256 targetBatchSize) external {
        proposalCount++;
        proposals[proposalCount] = Proposal({
            id: proposalCount,
            description: description,
            targetBatchSize: targetBatchSize,
            votes: 0,
            executed: false
        });

        emit ProposalCreated(proposalCount, description, targetBatchSize);
    }

    function vote(uint256 proposalId) external {
        require(proposalId > 0 && proposalId <= proposalCount, "Invalid proposal");
        require(!hasVoted[proposalId][msg.sender], "Already voted");
        require(!proposals[proposalId].executed, "Already executed");

        hasVoted[proposalId][msg.sender] = true;
        proposals[proposalId].votes++;

        emit Voted(proposalId, msg.sender);
    }

    // In a real system, execution requires checking BFT signatures,
    // 2/3 quorum, and ZK proofs. This is a simplified stub.
    function executeProposal(uint256 proposalId) external {
        Proposal storage p = proposals[proposalId];
        require(!p.executed, "Already executed");
        // Simulated quorum logic: require at least 1 vote for the stub
        require(p.votes > 0, "Not enough votes");

        p.executed = true;

        // Call the Constitution contract to update parameter (e.g. via an interface)
        // Note: For a complete implementation, an interface for Constitution is needed.
        (bool success, ) = constitutionContract.call(
            abi.encodeWithSignature("updateBatchSize(uint256)", p.targetBatchSize)
        );
        require(success, "Update failed");

        emit ProposalExecuted(proposalId);
    }
}
