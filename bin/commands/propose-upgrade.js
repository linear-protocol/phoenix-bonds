const { readFileSync, appendFileSync } = require("fs");
const nearAPI = require("near-api-js");
const { NEAR } = require("near-units");
const { getEnvConfig } = require("../helper");
const { init } = require("../near");

exports.command = "propose-upgrade";
exports.desc = "Propose an upgrade in DAO";
exports.builder = (yargs) => {
  yargs
    .option("env", {
      describe: "env",
      default: "dev",
    })
    .option("wasm", {
      describe: "New contract wasm file path",
      default: "res/phoenix_bonds.wasm",
    })
    .option("signer", {
      describe: "signer account ID to call new",
    })
    .option("v", {
      describe: "New contract version number",
    })
    .demandOption(["signer", "v"]);
};

exports.handler = async function (argv) {
  const { env, wasm, signer, v } = argv;
  const code = readFileSync(wasm);
  const config = getEnvConfig(env);
  console.log(`Upgrading contract ${config.contractId}`);
  console.log(`DAO is ${config.ownerId}`);

  const near = await init(env);
  const account = await near.account(signer);

  // store blob first
  const outcome = await account.signAndSendTransaction({
    receiverId: config.ownerId,
    actions: [
      nearAPI.transactions.functionCall(
        "store_blob",
        code,
        100000000000000,
        "6185190000000000000000000"
      ),
    ],
  });
  const hash = parseHashReturnValue(outcome);
  console.log("blob hash", hash);

  // save blob hash to local file
  appendFileSync(`blobhash-${env}`, hash);

  const proposalArgs = {
    proposal: {
      description: `Upgrade phoenix bonds contract to ${v}`,
      kind: {
        UpgradeRemote: {
          receiver_id: config.contractId,
          method_name: "upgrade",
          hash,
        },
      },
    },
  };
  console.log(JSON.stringify(proposalArgs, undefined, 4));

  await account.functionCall({
    contractId: config.ownerId,
    methodName: "add_proposal",
    args: proposalArgs,
    attachedDeposit: NEAR.parse("0.1"),
  });

  console.log("proposed!");
};

function parseHashReturnValue(outcome) {
  const status = outcome.status;
  const data = status.SuccessValue;
  if (!data) {
    throw new Error("bad return value");
  }

  const buff = Buffer.from(data, "base64");
  return buff.toString("ascii").replaceAll('"', "");
}
