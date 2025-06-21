import { getContract, zeroAddress, type Address } from "viem";
import { MockERC20ABI } from "./abis/MockERC20.abi";
import { publicClient, stylusSwap, walletClient } from "./constants";
import { deployMockErc20 } from "./utils/deployMockERC20";

export async function createPool(
  tokenOne: Address,
  tokenTwo: Address,
  fee: number
) {
  const createPoolHash = await stylusSwap.write.createPool([
    tokenOne,
    tokenTwo,
    fee,
  ]);

  const createPoolReceipt = await publicClient.waitForTransactionReceipt({
    hash: createPoolHash,
  });

  return createPoolReceipt;
}

export async function addLiquidity(
  poolId: `0x${string}`,
  amount0Desired: bigint,
  amount1Desired: bigint,
  amount0Min: bigint,
  amount1Min: bigint,
  isToken0Native?: boolean
) {
  const addLiquidityHash = await stylusSwap.write.addLiquidity(
    [poolId, amount0Desired, amount1Desired, amount0Min, amount1Min],
    {
      value: isToken0Native ? amount0Desired : 0n,
    }
  );

  const addLiquidityReceipt = await publicClient.waitForTransactionReceipt({
    hash: addLiquidityHash,
  });

  return addLiquidityReceipt;
}

export async function swap(
  poolId: `0x${string}`,
  inputAmount: bigint,
  minOutputAmount: bigint,
  zeroForOne: boolean,
  isToken0Native?: boolean
) {
  const addValue = isToken0Native && zeroForOne;

  const swapHash = await stylusSwap.write.swap(
    [poolId, inputAmount, minOutputAmount, zeroForOne],
    {
      value: addValue ? inputAmount : 0n,
    }
  );

  const swapReceipt = await publicClient.waitForTransactionReceipt({
    hash: swapHash,
  });

  return swapReceipt;
}

export async function removeLiquidity(
  poolId: `0x${string}`,
  liquidityToRemove: bigint
) {
  const removeLiquidityHash = await stylusSwap.write.removeLiquidity([
    poolId,
    liquidityToRemove,
  ]);

  const removeLiquidityReceipt = await publicClient.waitForTransactionReceipt({
    hash: removeLiquidityHash,
  });

  return removeLiquidityReceipt;
}

export async function getPositionLiquidity(poolId: `0x${string}`) {
  const positionLiquidity = await stylusSwap.read.getPositionLiquidity([
    poolId,
    walletClient.account.address,
  ]);
  return positionLiquidity;
}

export async function getBalance(token: Address) {
  if (token === zeroAddress) {
    return publicClient.getBalance({ address: walletClient.account.address });
  }

  const tokenContract = getContract({
    abi: MockERC20ABI,
    address: token,
    client: walletClient,
  });

  const balance = await tokenContract.read.balanceOf([
    walletClient.account.address,
  ]);
  return balance;
}
