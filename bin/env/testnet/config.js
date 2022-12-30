module.exports = {
  network: {
    networkId: "testnet",
    nodeUrl: process.env.NEAR_CLI_TESTNET_RPC_SERVER_URL || "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org",
  },
  contractId: 'phoenix-bonds.testnet',
  ownerId: 'linear.sputnik-dao.near',
  linearAddress: 'linear-protocol.testnet',
  tau: 0.03,
  bootstrapLength: 5 * 24, // hours
  accrual: {
    alpha: 1.4666, // days
    minAlpha: 0.1, // days
    targetMeanLength: 5, // days
    adjustInterval: 24, // hours
    adjustRate: 0.03,
  }
}
