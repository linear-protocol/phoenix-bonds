module.exports = {
  network: {
    networkId: "mainnet",
    nodeUrl:
      process.env.NEAR_CLI_MAINNET_RPC_SERVER_URL ||
      "https://rpc.mainnet.near.org",
    walletUrl: "https://wallet.mainnet.near.org",
    helperUrl: "https://helper.mainnet.near.org",
    explorerUrl: "https://explorer.mainnet.near.org",
  },
  contractId: "phoenix-bonds.near",
  ownerId: "linear.sputnik-dao.near",
  linearAddress: "linear-protocol.near",
  tau: 0.03,
  bootstrapLength: 15 * 24, // hours
  accrual: {
    alpha: 4.4, // days
    minAlpha: 0.1, // days
    targetMeanLength: 15, // days
    adjustInterval: 24, // hours
    adjustRate: 0.01,
  },
};
