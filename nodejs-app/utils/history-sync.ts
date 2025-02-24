import { Address, createPublicClient, http } from "viem";
import { MultichainWatcher } from "./multichain-watcher.js";

/// Sync history of missed events between certain blocks
export async function historySync(
  watcher: MultichainWatcher,
  chainId: number,
  fromBlock: bigint,
  toBlock: bigint,
  addresses?: Address[]
): Promise<void> {
  const rpc = getSyncRPC(chainId);
  const publicClient = createPublicClient({
    transport: http(rpc),
  });
  if ((await publicClient.getChainId()) !== chainId) {
    throw new Error(
      `Chain id of rpc ${rpc} does not match expected chain id ${chainId}`
    );
  }
  const maxRange = BigInt(1_000_000);
  const ranges = Array.from(
    { length: Math.max(Number((toBlock - fromBlock) / maxRange), 1) },
    (_, i) => fromBlock + maxRange * BigInt(i)
  );
  if (ranges.at(-1) !== toBlock) {
    ranges.push(toBlock);
  }
  for (let i = 1; i < ranges.length; i++) {
    console.log(`Getting logs from blocks ${ranges[i - 1]} to ${ranges[i]}...`);
    const logs = await publicClient.getLogs({
      address: addresses,
      fromBlock: ranges[i - 1],
      toBlock: ranges[i],
    });
    console.log(`Processing ${logs.length} logs...`);
    for (let j = 0; j < logs.length; j++) {
      // Process logs 1 by one in order to prevent race conditions
      await watcher.processLogs(chainId, [logs[j]]);
      await new Promise((resolve) => setTimeout(resolve, 100)); // Stay under RPC limits
    }
  }
  console.log("Sync finished!");
}

function getSyncRPC(chainId: number): string {
  switch (chainId) {
    case 1:
      return `https://mainnet.infura.io/v3/${process.env.INFURA_API_KEY}`; //"https://rpc.ankr.com/eth";
    case 11155111:
      return `https://sepolia.infura.io/v3/${process.env.INFURA_API_KEY}`; // "https://rpc.ankr.com/eth_sepolia";
    case 137:
      return `https://polygon-mainnet.infura.io/v3/${process.env.INFURA_API_KEY}`; // "https://rpc.ankr.com/polygon";
    case 421614:
      return `https://arbitrum-sepolia.infura.io/v3/${process.env.INFURA_API_KEY}`; // "https://rpc.ankr.com/arbitrum_sepolia";
    default:
      throw new Error(`Chain with id ${chainId} not found`);
  }
}
