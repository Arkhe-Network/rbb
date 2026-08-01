import { ethers } from "hardhat";

async function main() {
    // 1. Deploy AlloPool
    const AlloPool = await ethers.getContractFactory("AlloPool");
    const pool = await AlloPool.deploy(
        ethers.ZeroAddress, // strategy placeholder
        "0x0000000000000000000000000000000000000000" // manager address
    );
    await pool.waitForDeployment();
    console.log("AlloPool deployed to:", await pool.getAddress());

    // 2. Deploy QuadraticFundingStrategy
    const QFStrategy = await ethers.getContractFactory("QuadraticFundingStrategy");
    const qf = await QFStrategy.deploy(
        await pool.getAddress(),
        ethers.parseEther("100"), // matching pool
        7 * 24 * 60 * 60 // 7 days
    );
    await qf.waitForDeployment();
    console.log("QF Strategy deployed to:", await qf.getAddress());

    // 3. Update pool with strategy
    await pool.setStrategy(await qf.getAddress());

    // 4. Register recipients
    await qf.registerRecipient("0x0000000000000000000000000000000000000000");
}

main().catch(console.error);