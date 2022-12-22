module.exports = {
  network: {
    networkId: "testnet",
    nodeUrl: process.env.NODE_URL_TESTNET || "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org",
  },
  contractId: 'phoenix-dev1.testnet',
  ownerId: 'phoenix-dev1.testnet',
  linearAddress: 'linear-protocol.testnet',
  tau: 0.03,
  bootstrapLength: 15, // days
  accrual: {
    alpha: 4.4, // days
    minAlpha: 0.1, // days
    targetMeanLength: 15, // days
    adjustInterval: 24, // hours
    adjustRate: 0.01,
  }
}
