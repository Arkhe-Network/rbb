import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();

  console.log("Deploying Kleros Bridge stack with account:", deployer.address);

  // 1. Deploy PNKTheosisOracle
  const OracleFactory = await ethers.getContractFactory("PNKTheosisOracle");
  const pnkTheosisOracle = await OracleFactory.deploy();
  await pnkTheosisOracle.waitForDeployment();
  const oracleAddress = await pnkTheosisOracle.getAddress();
  console.log("PNKTheosisOracle deployed to:", oracleAddress);

  // 2. Deploy base CathedralKlerosBridge
  // For the actual bridge we need a Vea Relay address. Using a mock zero address for deployment
  const veaRelayMock = "0x0000000000000000000000000000000000000000";
  const BaseBridgeFactory = await ethers.getContractFactory("CathedralKlerosBridge");
  const baseBridge = await BaseBridgeFactory.deploy(veaRelayMock);
  await baseBridge.waitForDeployment();
  const baseBridgeAddress = await baseBridge.getAddress();
  console.log("CathedralKlerosBridge (Base) deployed to:", baseBridgeAddress);

  // 3. Deploy CathedralKlerosBridgeWithVoting
  const BridgeWithVotingFactory = await ethers.getContractFactory("CathedralKlerosBridgeWithVoting");
  const bridgeWithVoting = await BridgeWithVotingFactory.deploy(veaRelayMock, oracleAddress);
  await bridgeWithVoting.waitForDeployment();
  const bridgeWithVotingAddress = await bridgeWithVoting.getAddress();
  console.log("CathedralKlerosBridgeWithVoting deployed to:", bridgeWithVotingAddress);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
