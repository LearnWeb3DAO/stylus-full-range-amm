import { maxUint256 } from "viem";
import { MockERC20ABI } from "../abis/MockERC20.abi";
import {
  MOCKERC20_BYTECODE,
  publicClient,
  StylusSwapAddress,
  walletClient,
} from "../constants";

export async function deployMockErc20(name: string, symbol: string) {
  const txHash = await walletClient.deployContract({
    abi: MockERC20ABI,
    bytecode: MOCKERC20_BYTECODE,
    args: [name, symbol, 18],
  });

  const receipt = await publicClient.waitForTransactionReceipt({
    hash: txHash,
  });

  if (!receipt.contractAddress) {
    throw new Error("Failed to deploy contract");
  }

  const mintHash = await walletClient.writeContract({
    abi: MockERC20ABI,
    address: receipt.contractAddress,
    functionName: "mint",
    args: [walletClient.account.address, 10000000000000000000n],
  });

  await publicClient.waitForTransactionReceipt({
    hash: mintHash,
  });

  const approveHash = await walletClient.writeContract({
    abi: MockERC20ABI,
    address: receipt.contractAddress,
    functionName: "approve",
    args: [StylusSwapAddress, maxUint256],
  });

  await publicClient.waitForTransactionReceipt({
    hash: approveHash,
  });

  return receipt.contractAddress;
}
