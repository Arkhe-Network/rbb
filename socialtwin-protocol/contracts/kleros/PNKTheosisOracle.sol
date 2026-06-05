// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/Ownable.sol";

contract PNKTheosisOracle is Ownable {
    struct JurorMetrics {
        uint256 pnkBalance;
        uint256 theosisLevel; // Scale of 0 to 10000 (0.00 to 1.00)
    }

    mapping(address => JurorMetrics) public metrics;

    event JurorMetricsUpdated(address indexed juror, uint256 pnkBalance, uint256 theosisLevel);

    constructor() Ownable(msg.sender) {}

    function updateJurorMetrics(address _juror, uint256 _pnkBalance, uint256 _theosisLevel) external onlyOwner {
        require(_theosisLevel <= 10000, "Theosis level must be <= 10000");
        metrics[_juror] = JurorMetrics({
            pnkBalance: _pnkBalance,
            theosisLevel: _theosisLevel
        });
        emit JurorMetricsUpdated(_juror, _pnkBalance, _theosisLevel);
    }

    function getJurorMetrics(address _juror) external view returns (uint256, uint256) {
        return (metrics[_juror].pnkBalance, metrics[_juror].theosisLevel);
    }
}
