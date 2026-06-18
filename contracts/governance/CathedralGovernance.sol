// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title CathedralGovernance
 * @dev ASI-GOV.BR Governance Contract
 * Handles citizen voting, delegation, regional weighting, and expert human veto.
 */
contract CathedralGovernance {
    struct Voter {
        bool hasVoted;
        address delegate;
        uint256 voteWeight; // Based on regional weight / reputation
    }

    struct Proposal {
        uint256 id;
        address proposer;
        string descriptionHash; // IPFS or WormGraph hash
        uint256 forVotes;
        uint256 againstVotes;
        uint256 startTime;
        uint256 endTime;
        bool executed;
        bool vetoed;
    }

    address public vetoCouncil;
    address public admin;
    uint256 public proposalCount;

    mapping(uint256 => Proposal) public proposals;
    mapping(address => Voter) public voters;
    mapping(address => uint256) public initialWeights; // Track original weight for re-delegation
    mapping(uint256 => mapping(address => bool)) public proposalVotes;

    event ProposalCreated(uint256 indexed id, address proposer, string descriptionHash);
    event Voted(uint256 indexed proposalId, address voter, bool support, uint256 weight);
    event ProposalExecuted(uint256 indexed id);
    event ProposalVetoed(uint256 indexed id, address vetoer);
    event DelegateChanged(address indexed delegator, address indexed fromDelegate, address indexed toDelegate);

    modifier onlyVetoCouncil() {
        require(msg.sender == vetoCouncil, "Only veto council can perform this action");
        _;
    }

    modifier onlyAdmin() {
        require(msg.sender == admin, "Only admin can perform this action");
        _;
    }

    constructor(address _vetoCouncil) {
        vetoCouncil = _vetoCouncil;
        admin = msg.sender;
    }

    /**
     * @dev Register a voter and assign weight (simplified for demo).
     */
    function registerVoter(address _voter, uint256 _weight) external onlyAdmin {
        require(initialWeights[_voter] == 0, "Voter already registered");
        voters[_voter].voteWeight = _weight;
        initialWeights[_voter] = _weight;
    }

    /**
     * @dev Delegate votes to another citizen or expert.
     */
    function delegateVote(address _to) external {
        address currentDelegate = voters[msg.sender].delegate;
        require(_to != msg.sender, "Cannot delegate to self");

        voters[msg.sender].delegate = _to;

        uint256 delegatorWeight = initialWeights[msg.sender];
        if (delegatorWeight > 0) {
            if (currentDelegate != address(0)) {
                voters[currentDelegate].voteWeight -= delegatorWeight;
            } else {
                // If delegating for the first time, subtract only the user's initial weight from their total
                voters[msg.sender].voteWeight -= delegatorWeight;
            }

            if (_to != address(0)) {
                voters[_to].voteWeight += delegatorWeight;
            } else {
                // If undelegating (to address(0)), return the weight to the user
                voters[msg.sender].voteWeight += delegatorWeight;
            }
        }

        emit DelegateChanged(msg.sender, currentDelegate, _to);
    }

    /**
     * @dev Submit a new public policy proposal.
     */
    function submitProposal(string memory _descriptionHash, uint256 _votingPeriod) external returns (uint256) {
        proposalCount++;
        uint256 proposalId = proposalCount;

        Proposal storage newProposal = proposals[proposalId];
        newProposal.id = proposalId;
        newProposal.proposer = msg.sender;
        newProposal.descriptionHash = _descriptionHash;
        newProposal.startTime = block.timestamp;
        newProposal.endTime = block.timestamp + _votingPeriod;

        emit ProposalCreated(proposalId, msg.sender, _descriptionHash);
        return proposalId;
    }

    /**
     * @dev Vote on a proposal.
     */
    function vote(uint256 _proposalId, bool _support) external {
        Proposal storage p = proposals[_proposalId];
        require(block.timestamp >= p.startTime, "Voting has not started");
        require(block.timestamp <= p.endTime, "Voting has ended");
        require(!proposalVotes[_proposalId][msg.sender], "Already voted on this proposal");

        uint256 weight = voters[msg.sender].voteWeight;
        require(weight > 0, "No voting weight");

        proposalVotes[_proposalId][msg.sender] = true;

        if (_support) {
            p.forVotes += weight;
        } else {
            p.againstVotes += weight;
        }

        emit Voted(_proposalId, msg.sender, _support, weight);
    }

    /**
     * @dev Execute a proposal if approved and not vetoed.
     */
    function executeProposal(uint256 _proposalId) external {
        Proposal storage p = proposals[_proposalId];
        require(block.timestamp > p.endTime, "Voting is still active");
        require(!p.executed, "Proposal already executed");
        require(!p.vetoed, "Proposal has been vetoed");

        // Quorum and majority check
        require(p.forVotes > p.againstVotes, "Proposal did not pass");

        p.executed = true;
        emit ProposalExecuted(_proposalId);
    }

    /**
     * @dev Human veto mechanism by expert council.
     */
    function vetoProposal(uint256 _proposalId) external onlyVetoCouncil {
        Proposal storage p = proposals[_proposalId];
        require(!p.executed, "Cannot veto executed proposal");

        p.vetoed = true;
        emit ProposalVetoed(_proposalId, msg.sender);
    }
}
