// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./CathedralKlerosBridge.sol";
import "./PNKTheosisOracle.sol";

contract CathedralKlerosBridgeWithVoting is CathedralKlerosBridge {
    PNKTheosisOracle public pnkTheosisOracle;

    event VoteCast(address indexed juror, uint256 courtId, uint256 weight);

    constructor(address _veaRelay, address _pnkTheosisOracle) CathedralKlerosBridge(_veaRelay) {
        pnkTheosisOracle = PNKTheosisOracle(_pnkTheosisOracle);
    }

    function calculateVotingWeight(address _juror) public view returns (uint256) {
        (uint256 pnkBalance, uint256 theosisLevel) = pnkTheosisOracle.getJurorMetrics(_juror);

        // Base weight is PNK balance. Theosis level (0-10000) acts as a multiplier.
        // Example: Theosis of 5000 means 1.5x weight. Theosis of 10000 means 2.0x weight.
        // Multiplier = 1 + (theosisLevel / 10000)

        uint256 theosisMultiplier = 10000 + theosisLevel; // e.g., 10000 + 5000 = 15000

        uint256 weight = (pnkBalance * theosisMultiplier) / 10000;

        return weight;
    }

    function castVote(uint256 _courtId, bytes calldata _data) external {
        uint256 weight = calculateVotingWeight(msg.sender);

        // Bridge the vote logic via Vea Relay to Kleros on Arbitrum
        emit VoteCast(msg.sender, _courtId, weight);

        // Use an internal call or call super so that msg.sender in bridgeMessage (if it had logic relying on it)
        // doesn't become `address(this)`. Or rather, CathedralKlerosBridge only emits right now, so we can emit directly
        // or just rely on the super behavior instead of an external call `this.bridgeMessage`
        emit MessageBridged(block.timestamp, msg.sender, _data);
    }
}
