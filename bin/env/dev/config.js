module.exports = {
  network: {
    networkId: "testnet",
    nodeUrl:
      process.env.NEAR_CLI_TESTNET_RPC_SERVER_URL ||
      "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org",
  },
  contractId: "phoenix-dev2.testnet",
  ownerId: "phoenix-dev2.testnet",
  linearAddress: "linear-protocol.testnet",
  tau: 0.03,
  bootstrapLength: 5, // hours
  accrual: {
    alpha: 4.4, // days
    minAlpha: 0.1, // days
    targetMeanLength: 2, // days
    adjustInterval: 24, // hours
    adjustRate: 0.01,
  },
};
