import { expect } from "chai";
import { ethers } from "hardhat";
import {
  PNKTheosisOracle,
  CathedralKlerosBridgeWithVoting
} from "../typechain-types";
import { HardhatEthersSigner } from "@nomicfoundation/hardhat-ethers/signers";

describe("Kleros Cathedral Bridge & Theosis Voting", function () {
  let pnkTheosisOracle: PNKTheosisOracle;
  let bridgeWithVoting: CathedralKlerosBridgeWithVoting;
  let owner: HardhatEthersSigner;
  let juror1: HardhatEthersSigner;
  let juror2: HardhatEthersSigner;
  let mockVeaRelay: HardhatEthersSigner;

  beforeEach(async function () {
    [owner, juror1, juror2, mockVeaRelay] = await ethers.getSigners();

    const OracleFactory = await ethers.getContractFactory("PNKTheosisOracle");
    pnkTheosisOracle = (await OracleFactory.deploy()) as PNKTheosisOracle;
    await pnkTheosisOracle.waitForDeployment();

    const BridgeFactory = await ethers.getContractFactory("CathedralKlerosBridgeWithVoting");
    bridgeWithVoting = (await BridgeFactory.deploy(
      mockVeaRelay.address,
      await pnkTheosisOracle.getAddress()
    )) as CathedralKlerosBridgeWithVoting;
    await bridgeWithVoting.waitForDeployment();
  });

  describe("PNKTheosisOracle", function () {
    it("Should allow owner to update juror metrics", async function () {
      await pnkTheosisOracle.updateJurorMetrics(juror1.address, 1000, 5000); // 1000 PNK, 5000 Theosis (0.5)

      const metrics = await pnkTheosisOracle.getJurorMetrics(juror1.address);
      expect(metrics[0]).to.equal(1000);
      expect(metrics[1]).to.equal(5000);
    });

    it("Should revert if non-owner tries to update", async function () {
      await expect(
        pnkTheosisOracle.connect(juror1).updateJurorMetrics(juror2.address, 1000, 5000)
      ).to.be.revertedWithCustomError(pnkTheosisOracle, "OwnableUnauthorizedAccount");
    });

    it("Should revert if Theosis level > 10000", async function () {
      await expect(
        pnkTheosisOracle.updateJurorMetrics(juror1.address, 1000, 10001)
      ).to.be.revertedWith("Theosis level must be <= 10000");
    });
  });

  describe("CathedralKlerosBridgeWithVoting", function () {
    beforeEach(async function () {
      // Setup some initial metrics
      await pnkTheosisOracle.updateJurorMetrics(juror1.address, 1000, 0); // Base: 1000 PNK, 0 Theosis (1.0x)
      await pnkTheosisOracle.updateJurorMetrics(juror2.address, 1000, 5000); // 1000 PNK, 5000 Theosis (1.5x)
    });

    it("Should calculate voting weight correctly based on Theosis", async function () {
      const weight1 = await bridgeWithVoting.calculateVotingWeight(juror1.address);
      const weight2 = await bridgeWithVoting.calculateVotingWeight(juror2.address);

      expect(weight1).to.equal(1000); // 1000 * (1 + 0) = 1000
      expect(weight2).to.equal(1500); // 1000 * (1 + 0.5) = 1500
    });

    it("Should emit VoteCast event when casting vote", async function () {
      const voteData = ethers.hexlify(ethers.toUtf8Bytes("Approve"));
      const tx = await bridgeWithVoting.connect(juror2).castVote(1, voteData);

      await expect(tx)
        .to.emit(bridgeWithVoting, "VoteCast")
        .withArgs(juror2.address, 1, 1500);

      // MessageBridged now correctly emits with msg.sender = juror2
      await expect(tx)
        .to.emit(bridgeWithVoting, "MessageBridged")
        .withArgs(
            (val: any) => true, // block.timestamp is passed, match any
            juror2.address, // Sender should be the original caller now
            voteData
        );
    });
  });
});
