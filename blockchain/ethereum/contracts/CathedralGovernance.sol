// SPDX-License-Identifier: AGPL-3.0
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract CathedralGovernance is AccessControl {
    using EnumerableSet for EnumerableSet.AddressSet;
    using ECDSA for bytes32;

    bytes32 public constant VOTER_ROLE = keccak256("VOTER_ROLE");
    bytes32 public constant VETO_ROLE = keccak256("VETO_ROLE");

    IERC20 public stakingToken;
    uint256 public minStake;
    uint256 public votingPeriod; // em blocos
    uint256 public delegationPeriod;

    struct Proposal {
        uint256 id;
        string title;
        string description;
        bytes codeChanges;
        address proposer;
        uint256 startBlock;
        uint256 endBlock;
        uint256 snapshotBlock;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 abstainVotes;
        bool executed;
        bool vetoed;
        mapping(address => bool) hasVoted;
        // Delegação: quem delegou para quem
        mapping(address => address) delegations;
    }

    struct Delegation {
        address delegator;
        address delegate;
        uint256 untilBlock;
    }

    uint256 public proposalCount;
    mapping(uint256 => Proposal) public proposals;
    EnumerableSet.AddressSet private voters;
    mapping(address => Delegation) public delegations;

    event ProposalCreated(uint256 indexed id, address proposer, string title);
    event VoteCast(uint256 indexed id, address voter, bool support, uint256 weight);
    event ProposalExecuted(uint256 indexed id);
    event ProposalVetoed(uint256 indexed id, address vetoer);
    event DelegationCreated(address indexed delegator, address indexed delegate, uint256 untilBlock);

    constructor(address _stakingToken, uint256 _minStake, uint256 _votingPeriod, uint256 _delegationPeriod) {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(VETO_ROLE, msg.sender);
        stakingToken = IERC20(_stakingToken);
        minStake = _minStake;
        votingPeriod = _votingPeriod;
        delegationPeriod = _delegationPeriod;
    }

    function addVoter(address voter) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(stakingToken.balanceOf(voter) >= minStake, "Insufficient stake");
        voters.add(voter);
        grantRole(VOTER_ROLE, voter);
    }

    function removeVoter(address voter) external onlyRole(DEFAULT_ADMIN_ROLE) {
        voters.remove(voter);
        revokeRole(VOTER_ROLE, voter);
    }

    function delegate(address delegatee) external {
        require(hasRole(VOTER_ROLE, msg.sender), "Not a voter");
        require(delegatee != msg.sender, "Cannot delegate to self");
        require(delegations[msg.sender].delegator == address(0), "Already delegated");
        require(block.number <= delegations[msg.sender].untilBlock, "Delegation active");

        // Verifica se o delegatário é um voter
        require(hasRole(VOTER_ROLE, delegatee), "Delegatee not a voter");

        delegations[msg.sender] = Delegation({
            delegator: msg.sender,
            delegate: delegatee,
            untilBlock: block.number + delegationPeriod
        });

        emit DelegationCreated(msg.sender, delegatee, block.number + delegationPeriod);
    }

    function revokeDelegation() external {
        require(delegations[msg.sender].delegator != address(0), "No active delegation");
        delete delegations[msg.sender];
    }

    function propose(
        string memory title,
        string memory description,
        bytes memory codeChanges
    ) external returns (uint256) {
        require(hasRole(VOTER_ROLE, msg.sender), "Not a voter");
        proposalCount++;
        uint256 id = proposalCount;
        Proposal storage p = proposals[id];
        p.id = id;
        p.title = title;
        p.description = description;
        p.codeChanges = codeChanges;
        p.proposer = msg.sender;
        p.startBlock = block.number;
        p.endBlock = block.number + votingPeriod;
        p.snapshotBlock = block.number - 1; // Snapshot no bloco anterior
        p.executed = false;
        p.vetoed = false;
        emit ProposalCreated(id, msg.sender, title);
        return id;
    }

    function vote(uint256 proposalId, bool support) external {
        Proposal storage p = proposals[proposalId];
        require(hasRole(VOTER_ROLE, msg.sender), "Not a voter");
        require(block.number >= p.startBlock && block.number <= p.endBlock, "Voting not active");
        require(!p.hasVoted[msg.sender], "Already voted");

        // Verifica se o voter delegou seu voto
        address voter = msg.sender;
        uint256 weight = 1; // Peso base (pode ser ponderado por stake)

        // Se há delegação ativa para este voter, o voto é contado como peso do delegado?
        // Nesta implementação, o delegado vota em nome do delegador, mas o peso é do delegador.
        // Para simplificar, usamos 1 voto por endereço.

        p.hasVoted[voter] = true;
        if (support) {
            p.forVotes += weight;
        } else {
            p.againstVotes += weight;
        }
        emit VoteCast(proposalId, voter, support, weight);
    }

    function voteWithDelegation(uint256 proposalId, bool support) external {
        // Se o msg.sender é um delegatário, pode votar em nome de seus delegadores
        // Implementação simplificada: apenas o delegatário vota com seu próprio voto + os de seus delegados
        // Para cada delegador, verifica se não votou ainda e vota com o peso do delegador.
        // Esta função pode ser complexa; para este exemplo, assumimos que o delegatário vota apenas por si.
        vote(proposalId, support);
    }

    function execute(uint256 proposalId) external {
        Proposal storage p = proposals[proposalId];
        require(block.number > p.endBlock, "Voting not ended");
        require(!p.executed && !p.vetoed, "Already executed or vetoed");
        require(p.forVotes > p.againstVotes, "Proposal rejected");
        p.executed = true;
        emit ProposalExecuted(proposalId);
        // Aqui a ação onchain (ex: aplicar código)
    }

    function veto(uint256 proposalId) external onlyRole(VETO_ROLE) {
        Proposal storage p = proposals[proposalId];
        require(!p.executed && !p.vetoed, "Already executed or vetoed");
        p.vetoed = true;
        emit ProposalVetoed(proposalId, msg.sender);
    }

    function getProposal(uint256 proposalId) external view returns (
        uint256 id, string memory title, string memory description,
        address proposer, uint256 startBlock, uint256 endBlock,
        uint256 forVotes, uint256 againstVotes, uint256 abstainVotes,
        bool executed, bool vetoed
    ) {
        Proposal storage p = proposals[proposalId];
        return (p.id, p.title, p.description, p.proposer, p.startBlock, p.endBlock,
                p.forVotes, p.againstVotes, p.abstainVotes, p.executed, p.vetoed);
    }

    function getVoters() external view returns (address[] memory) {
        return voters.values();
    }
}
