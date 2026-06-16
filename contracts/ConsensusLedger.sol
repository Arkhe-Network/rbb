// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title Cathedral Consensus Ledger
 * @dev On-chain immutable registry for multi-agent decisions in Cathedral ARKHE v28.3.
 */
contract ConsensusLedger {
    struct Decision {
        string workflowId;
        uint256 timestamp;
        string finalDecision;
        string temporalChainHash;
        string outcomeJson;
    }

    // Mapping from recordId to Decision
    mapping(string => Decision) public decisions;

    event DecisionRecorded(string indexed recordId, string indexed workflowId, uint256 timestamp);

    /**
     * @dev Records a new multi-agent consensus decision.
     * @param recordId Unique identifier for this record
     * @param workflowId Identifier of the workflow
     * @param finalDecision The text/summary of the decision
     * @param temporalChainHash Hash anchor to TemporalChain for full data integrity
     * @param outcomeJson JSON representation of the outcome details
     */
    function recordDecision(
        string memory recordId,
        string memory workflowId,
        string memory finalDecision,
        string memory temporalChainHash,
        string memory outcomeJson
    ) public {
        require(bytes(decisions[recordId].workflowId).length == 0, "Record ID already exists");

        decisions[recordId] = Decision({
            workflowId: workflowId,
            timestamp: block.timestamp,
            finalDecision: finalDecision,
            temporalChainHash: temporalChainHash,
            outcomeJson: outcomeJson
        });

        emit DecisionRecorded(recordId, workflowId, block.timestamp);
    }

    /**
     * @dev Retrieves a decision by its record ID.
     */
    function getDecision(string memory recordId) public view returns (
        string memory workflowId,
        uint256 timestamp,
        string memory finalDecision,
        string memory temporalChainHash,
        string memory outcomeJson
    ) {
        Decision memory d = decisions[recordId];
        require(bytes(d.workflowId).length > 0, "Decision not found");

        return (d.workflowId, d.timestamp, d.finalDecision, d.temporalChainHash, d.outcomeJson);
    }
}