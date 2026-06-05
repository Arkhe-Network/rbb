// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/access/Ownable.sol";

contract CathedralKlerosBridge is Ownable {
    address public veaRelay;

    event MessageBridged(uint256 indexed id, address indexed sender, bytes data);

    constructor(address _veaRelay) Ownable(msg.sender) {
        veaRelay = _veaRelay;
    }

    function bridgeMessage(bytes calldata _data) external {
        // Mock Vea relay bridging logic
        emit MessageBridged(block.timestamp, msg.sender, _data);
    }

    function setVeaRelay(address _veaRelay) external onlyOwner {
        veaRelay = _veaRelay;
    }
}
