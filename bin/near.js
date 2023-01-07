const nearAPI = require("near-api-js");
const { Gas, NEAR } = require("near-units");
const { getEnvConfig } = require("./helper");

/**
 * init near object
 * @param {'local' | 'dev' | 'testnet' | 'staging' | 'mainnet'} env
 * @returns
 */
async function init(env) {
  const { keyStores } = nearAPI;
  const homedir = require("os").homedir();
  const CREDENTIALS_DIR = ".near-credentials";
  const credentialsPath = require("path").join(homedir, CREDENTIALS_DIR);
  const keyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

  const configs = getEnvConfig(env);
  const config = configs.network;
  config.keyStore = keyStore;
  return nearAPI.connect(config);
}

async function funcCallProposal(
  signer,
  dao,
  description,
  contract,
  methodName,
  args,
  deposit,
  gas
) {
  deposit = deposit || "0";
  gas = gas || Gas.parse("100 Tgas");

  console.log("args", args);
  args = Buffer.from(JSON.stringify(args)).toString("base64");
  console.log("encoded args", args);

  const proposal = {
    proposal: {
      description,
      kind: {
        FunctionCall: {
          receiver_id: contract,
          actions: [
            {
              method_name: methodName,
              args,
              deposit,
              gas,
            },
          ],
        },
      },
    },
  };

  return signer.functionCall({
    contractId: dao,
    methodName: "add_proposal",
    args: proposal,
    gas: Gas.parse("200 Tgas"),
    attachedDeposit: NEAR.parse("0.1"),
  });
}

async function funcCall(
  signer,
  dao,
  description,
  contract,
  methodName,
  args,
  deposit,
  gas
) {
  if (!dao) {
    return signer.functionCall({
      contractId: contract,
      methodName,
      args,
      gas,
      attachedDeposit: deposit,
    });
  } else {
    return funcCallProposal(
      signer,
      dao,
      description,
      contract,
      methodName,
      args,
      deposit,
      gas
    );
  }
}

module.exports = {
  init,
  funcCallProposal,
  funcCall,
};
