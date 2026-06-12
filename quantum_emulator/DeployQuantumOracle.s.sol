// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "./QuantumTimestampOracle.sol";
import "./CathedralSPHINCSVerifier.sol";

contract DeployQuantumOracle is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address emulatorAddress = vm.addr(deployerPrivateKey); // or use a specific address

        vm.startBroadcast(deployerPrivateKey);

        CathedralSPHINCSVerifier verifier = new CathedralSPHINCSVerifier();
        QuantumTimestampOracle oracle = new QuantumTimestampOracle(emulatorAddress, address(verifier));

        vm.stopBroadcast();

        console.log("CathedralSPHINCSVerifier deployed at:", address(verifier));
        console.log("QuantumTimestampOracle deployed at:", address(oracle));
    }
}
