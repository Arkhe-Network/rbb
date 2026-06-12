import { expect } from "chai";
import { ethers } from "hardhat";
import { CathedralSPHINCSVerifier } from "../typechain-types";
import { generateMockSignature } from "../scripts/generate_signature";

describe("CathedralSPHINCSVerifier", function () {
    let verifier: CathedralSPHINCSVerifier;

    beforeEach(async function () {
        const factory = await ethers.getContractFactory("CathedralSPHINCSVerifier");
        verifier = await factory.deploy();
        await verifier.waitForDeployment();
    });

    it("should successfully verify a valid signature and measure gas", async function () {
        const msgStr = "TestMessage";
        const { message, signature, publicKeyRoot } = generateMockSignature(msgStr);

        // Call purely to check boolean return
        const isValid = await verifier.verifySPHINCS(message, signature, publicKeyRoot);
        expect(isValid).to.be.true;

        // Call as a transaction to measure gas usage
        // Note: verifySPHINCS is an external pure function, but we can wrap it in an estimateGas call
        const gasEstimate = await verifier.verifySPHINCS.estimateGas(message, signature, publicKeyRoot);
        console.log(`\tGas Used for verification: ${gasEstimate.toString()}`);
    });

    it("should reject an invalid signature", async function () {
        const msgStr = "TestMessage";
        const { message, signature, publicKeyRoot } = generateMockSignature(msgStr);

        // Mutate the signature slightly
        const badSignature = signature.slice(0, -4) + "0000";

        const isValid = await verifier.verifySPHINCS(message, badSignature, publicKeyRoot);
        expect(isValid).to.be.false;
    });

    it("should reject an invalid public key root", async function () {
        const msgStr = "TestMessage";
        const { message, signature, publicKeyRoot } = generateMockSignature(msgStr);

        // Mutate the public key root
        const badPublicKeyRoot = ethers.hexlify(ethers.randomBytes(32));

        const isValid = await verifier.verifySPHINCS(message, signature, badPublicKeyRoot);
        expect(isValid).to.be.false;
    });

    it("should reject a wrong message", async function () {
        const msgStr = "TestMessage";
        const { signature, publicKeyRoot } = generateMockSignature(msgStr);

        const badMessage = ethers.getBytes(ethers.id("WrongMessage"));

        const isValid = await verifier.verifySPHINCS(badMessage, signature, publicKeyRoot);
        expect(isValid).to.be.false;
    });
});
