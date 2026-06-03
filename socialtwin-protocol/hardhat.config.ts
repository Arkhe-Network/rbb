import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-toolbox";
import * as dotenv from "dotenv";
import * as path from "path";

dotenv.config({ path: path.join(__dirname, ".env") });

const PRIVATE_KEY = process.env.PRIVATE_KEY || "";
const RBB_RPC_URL = process.env.RBB_RPC_URL || "https://mainnet.rbb.org";
const RBBSCAN_API_KEY = process.env.RBBSCAN_API_KEY || "";

const config: HardhatUserConfig = {
  solidity: {
    version: "0.8.24",
    settings: {
      viaIR: true,
      evmVersion: "cancun",
      optimizer: { enabled: true, runs: 200 },
    },
  },
  networks: {
    rbb: {
      url: RBB_RPC_URL,
      accounts: PRIVATE_KEY ? [PRIVATE_KEY] : [],
      chainId: 12120014,
    },
    rbbSepolia: {
      url: "https://sepolia.rbb.org",
      accounts: PRIVATE_KEY ? [PRIVATE_KEY] : [],
      chainId: 121200142,
    },
    hardhat: {
      chainId: 121200142,
    },
  },
  etherscan: {
    apiKey: RBBSCAN_API_KEY,
    customChains: [
      {
        network: "rbb",
        chainId: 12120014,
        urls: { apiURL: "https://api.rbbscan.org/api", browserURL: "https://rbbscan.org" },
      },
      {
        network: "rbbSepolia",
        chainId: 121200142,
        urls: { apiURL: "https://api-sepolia.rbbscan.org/api", browserURL: "https://sepolia.rbbscan.org" },
      },
    ],
  },
  paths: {
    sources: "./contracts",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts",
  },
};

export default config;
