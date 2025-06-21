export const StylusSwapABI = [
  {
    inputs: [
      { internalType: "address", name: "", type: "address" },
      { internalType: "address", name: "", type: "address" },
      { internalType: "address", name: "", type: "address" },
      { internalType: "uint256", name: "", type: "uint256" },
    ],
    name: "FailedOrInsufficientTokenTransfer",
    type: "error",
  },
  {
    inputs: [
      { internalType: "address", name: "", type: "address" },
      { internalType: "uint256", name: "", type: "uint256" },
    ],
    name: "FailedToReturnExtraEth",
    type: "error",
  },
  { inputs: [], name: "InsufficientAmount", type: "error" },
  { inputs: [], name: "InsufficientLiquidityMinted", type: "error" },
  { inputs: [], name: "InsufficientLiquidityOwned", type: "error" },
  {
    inputs: [{ internalType: "bytes32", name: "", type: "bytes32" }],
    name: "PoolAlreadyExists",
    type: "error",
  },
  {
    inputs: [{ internalType: "bytes32", name: "", type: "bytes32" }],
    name: "PoolDoesNotExist",
    type: "error",
  },
  { inputs: [], name: "TooMuchSlippage", type: "error" },
  {
    inputs: [
      { internalType: "bytes32", name: "pool_id", type: "bytes32" },
      { internalType: "uint256", name: "amount_0_desired", type: "uint256" },
      { internalType: "uint256", name: "amount_1_desired", type: "uint256" },
      { internalType: "uint256", name: "amount_0_min", type: "uint256" },
      { internalType: "uint256", name: "amount_1_min", type: "uint256" },
    ],
    name: "addLiquidity",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "token_a", type: "address" },
      { internalType: "address", name: "token_b", type: "address" },
      { internalType: "uint24", name: "fee", type: "uint24" },
    ],
    name: "createPool",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "uint256", name: "amount_0_desired", type: "uint256" },
      { internalType: "uint256", name: "amount_1_desired", type: "uint256" },
      { internalType: "uint256", name: "amount_0_min", type: "uint256" },
      { internalType: "uint256", name: "amount_1_min", type: "uint256" },
      { internalType: "uint256", name: "balance_0", type: "uint256" },
      { internalType: "uint256", name: "balance_1", type: "uint256" },
    ],
    name: "getLiquidityAmounts",
    outputs: [
      { internalType: "uint256", name: "", type: "uint256" },
      { internalType: "uint256", name: "", type: "uint256" },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "address", name: "token_a", type: "address" },
      { internalType: "address", name: "token_b", type: "address" },
      { internalType: "uint24", name: "fee", type: "uint24" },
    ],
    name: "getPoolId",
    outputs: [
      { internalType: "bytes32", name: "", type: "bytes32" },
      { internalType: "address", name: "", type: "address" },
      { internalType: "address", name: "", type: "address" },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "bytes32", name: "pool_id", type: "bytes32" },
      { internalType: "address", name: "owner", type: "address" },
    ],
    name: "getPositionId",
    outputs: [{ internalType: "bytes32", name: "", type: "bytes32" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "bytes32", name: "pool_id", type: "bytes32" },
      { internalType: "address", name: "owner", type: "address" },
    ],
    name: "getPositionLiquidity",
    outputs: [{ internalType: "uint256", name: "", type: "uint256" }],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [
      { internalType: "bytes32", name: "pool_id", type: "bytes32" },
      { internalType: "uint256", name: "liquidity_to_remove", type: "uint256" },
    ],
    name: "removeLiquidity",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
  {
    inputs: [
      { internalType: "bytes32", name: "pool_id", type: "bytes32" },
      { internalType: "uint256", name: "input_amount", type: "uint256" },
      { internalType: "uint256", name: "min_output_amount", type: "uint256" },
      { internalType: "bool", name: "zero_for_one", type: "bool" },
    ],
    name: "swap",
    outputs: [],
    stateMutability: "payable",
    type: "function",
  },
] as const;
