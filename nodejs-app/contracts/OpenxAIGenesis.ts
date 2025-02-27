export const OpenxAIGenesisContract = {
  address: "0xBb2b75AF6D25A9474BCBef8AF08FF80A80316cD1",
  abi: [
    {
      type: "constructor",
      inputs: [
        {
          name: "_ethOracle",
          type: "address",
          internalType: "contract AggregatorV3Interface",
        },
        {
          name: "_wrappedEth",
          type: "address[]",
          internalType: "contract IERC20[]",
        },
        {
          name: "_stableCoins",
          type: "address[]",
          internalType: "contract IERC20[]",
        },
        {
          name: "_tiers",
          type: "tuple[]",
          internalType: "struct OpenxAIGenesis.Tier[]",
          components: [
            { name: "amount", type: "uint96", internalType: "uint96" },
            {
              name: "escrow",
              type: "address",
              internalType: "address payable",
            },
          ],
        },
      ],
      stateMutability: "nonpayable",
    },
    {
      type: "function",
      name: "rescue",
      inputs: [
        { name: "_token", type: "address", internalType: "contract IERC20" },
        { name: "_receiver", type: "address", internalType: "address payable" },
        { name: "_amount", type: "uint256", internalType: "uint256" },
      ],
      outputs: [],
      stateMutability: "nonpayable",
    },
    {
      type: "function",
      name: "tiers",
      inputs: [{ name: "", type: "uint256", internalType: "uint256" }],
      outputs: [
        { name: "amount", type: "uint96", internalType: "uint96" },
        { name: "escrow", type: "address", internalType: "address payable" },
      ],
      stateMutability: "view",
    },
    {
      type: "function",
      name: "transfer_erc20",
      inputs: [
        { name: "_token", type: "address", internalType: "contract IERC20" },
        { name: "_amount", type: "uint256", internalType: "uint256" },
      ],
      outputs: [],
      stateMutability: "nonpayable",
    },
    {
      type: "function",
      name: "transfer_native",
      inputs: [],
      outputs: [],
      stateMutability: "payable",
    },
    {
      type: "event",
      name: "Participated",
      inputs: [
        {
          name: "tier",
          type: "uint256",
          indexed: true,
          internalType: "uint256",
        },
        {
          name: "account",
          type: "address",
          indexed: true,
          internalType: "address",
        },
        {
          name: "amount",
          type: "uint256",
          indexed: false,
          internalType: "uint256",
        },
      ],
      anonymous: false,
    },
    { type: "error", name: "FailedCall", inputs: [] },
    {
      type: "error",
      name: "InsufficientBalance",
      inputs: [
        { name: "balance", type: "uint256", internalType: "uint256" },
        { name: "needed", type: "uint256", internalType: "uint256" },
      ],
    },
    {
      type: "error",
      name: "InvalidPrice",
      inputs: [{ name: "price", type: "int256", internalType: "int256" }],
    },
    {
      type: "error",
      name: "NotRescue",
      inputs: [
        { name: "caller", type: "address", internalType: "address" },
        { name: "rescue", type: "address", internalType: "address" },
      ],
    },
    {
      type: "error",
      name: "SafeERC20FailedOperation",
      inputs: [{ name: "token", type: "address", internalType: "address" }],
    },
    {
      type: "error",
      name: "UnsupportedTransferToken",
      inputs: [
        { name: "token", type: "address", internalType: "contract IERC20" },
      ],
    },
  ],
} as const;
