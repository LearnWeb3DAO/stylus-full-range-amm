import { stylusSwap } from "./constants";
import {
  addLiquidity,
  createPool,
  getBalance,
  getPositionLiquidity,
  removeLiquidity,
  swap,
} from "./utils";
import { deployMockErc20 } from "./utils/deployMockERC20";

import { expect, test } from "bun:test";
import { zeroAddress } from "viem";

test("Two ERC-20 Tokens, 10% fee", async () => {
  const tokenOne = await deployMockErc20("Test One", "ONE");
  const tokenTwo = await deployMockErc20("Test Two", "TWO");

  const [poolId, token0, token1] = await stylusSwap.read.getPoolId([
    tokenOne,
    tokenTwo,
    1000,
  ]);

  await createPool(tokenOne, tokenTwo, 1000);
  const [originalToken0Balance, originalToken1Balance] = await Promise.all([
    getBalance(token0),
    getBalance(token1),
  ]);

  await addLiquidity(poolId, 100_000n, 100_000n, 0n, 0n);

  const [afterAddLiquidityToken0Balance, afterAddLiquidityToken1Balance] =
    await Promise.all([getBalance(token0), getBalance(token1)]);

  const token0AddedAsLiquidity =
    originalToken0Balance - afterAddLiquidityToken0Balance;
  const token1AddedAsLiquidity =
    originalToken1Balance - afterAddLiquidityToken1Balance;

  expect(token0AddedAsLiquidity).toEqual(100_000n);
  expect(token1AddedAsLiquidity).toEqual(100_000n);

  const userLiquidity = await getPositionLiquidity(poolId);
  expect(userLiquidity).toEqual(100_000n - 1000n);

  await swap(poolId, 10n, 0n, true);

  const [afterSwapToken0Balance, afterSwapToken1Balance] = await Promise.all([
    getBalance(token0),
    getBalance(token1),
  ]);

  const token0Spent = afterAddLiquidityToken0Balance - afterSwapToken0Balance;
  const token1Gained = afterSwapToken1Balance - afterAddLiquidityToken1Balance;

  expect(token0Spent).toEqual(10n);
  expect(token1Gained).toEqual(9n);

  await removeLiquidity(poolId, userLiquidity);

  const [afterRemoveLiquidityToken0Balance, afterRemoveLiquidityToken1Balance] =
    await Promise.all([getBalance(token0), getBalance(token1)]);

  const token0Removed =
    afterRemoveLiquidityToken0Balance - afterSwapToken0Balance;
  const token1Removed =
    afterRemoveLiquidityToken1Balance - afterSwapToken1Balance;

  expect(token0Removed).toEqual(99_009n);
  expect(token1Removed).toEqual(98_991n);
});

test("ETH and ERC-20 Token, 10% fee", async () => {
  const token = await deployMockErc20("Test", "TST");

  const [poolId, token0, token1] = await stylusSwap.read.getPoolId([
    zeroAddress,
    token,
    1000,
  ]);
  await createPool(zeroAddress, token, 1000);
  const [originalToken0Balance, originalToken1Balance] = await Promise.all([
    getBalance(zeroAddress),
    getBalance(token),
  ]);

  const addLiquidityReceipt = await addLiquidity(
    poolId,
    100_000n,
    100_000n,
    0n,
    0n,
    true
  );

  const ethSpentOnGas =
    addLiquidityReceipt.cumulativeGasUsed *
    addLiquidityReceipt.effectiveGasPrice;

  const [afterAddLiquidityToken0Balance, afterAddLiquidityToken1Balance] =
    await Promise.all([getBalance(zeroAddress), getBalance(token)]);

  const token0AddedAsLiquidity =
    originalToken0Balance - ethSpentOnGas - afterAddLiquidityToken0Balance;
  const token1AddedAsLiquidity =
    originalToken1Balance - afterAddLiquidityToken1Balance;

  expect(token0AddedAsLiquidity).toEqual(100_000n);
  expect(token1AddedAsLiquidity).toEqual(100_000n);

  const swapReceipt = await swap(poolId, 10n, 0n, true, true);

  const ethSpentOnGasSwap =
    swapReceipt.cumulativeGasUsed * swapReceipt.effectiveGasPrice;

  const [afterSwapToken0Balance, afterSwapToken1Balance] = await Promise.all([
    getBalance(zeroAddress),
    getBalance(token),
  ]);

  const token0Spent =
    afterAddLiquidityToken0Balance - ethSpentOnGasSwap - afterSwapToken0Balance;
  const token1Gained = afterSwapToken1Balance - afterAddLiquidityToken1Balance;

  expect(token0Spent).toEqual(10n);
  expect(token1Gained).toEqual(9n);
});

test("Cannot create pool with same token pair and fee value twice", async () => {
  const tokenOne = await deployMockErc20("Test One", "ONE");
  const tokenTwo = await deployMockErc20("Test Two", "TWO");

  await createPool(tokenOne, tokenTwo, 1000);

  expect(createPool(tokenOne, tokenTwo, 1000)).rejects.toThrow(
    "PoolAlreadyExists"
  );
});

test("Cannot add liquidity or swap in a pool that does not exist", async () => {
  const randomPoolId =
    "0x0000000000000000000000000000000000000000000000000000000000000000";

  expect(
    addLiquidity(randomPoolId, 100_000n, 100_000n, 0n, 0n)
  ).rejects.toThrow("PoolDoesNotExist");

  expect(swap(randomPoolId, 10n, 0n, true)).rejects.toThrow("PoolDoesNotExist");
});

test("Cannot remove more liquidity than you have", async () => {
  const tokenOne = await deployMockErc20("Test One", "ONE");
  const tokenTwo = await deployMockErc20("Test Two", "TWO");

  const [poolId, _token0, _token1] = await stylusSwap.read.getPoolId([
    tokenOne,
    tokenTwo,
    1000,
  ]);

  await createPool(tokenOne, tokenTwo, 1000);
  await addLiquidity(poolId, 100_000n, 100_000n, 0n, 0n);

  expect(removeLiquidity(poolId, 500_000n)).rejects.toThrow(
    "InsufficientLiquidityOwned"
  );
});
