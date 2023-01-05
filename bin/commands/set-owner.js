const { getEnvConfig } = require("../helper");
const initNear = require("../near");
const fs = require("fs");
const { confirm } = require("./utils");

exports.command = "set-owner";
exports.desc = "Set new owner";
exports.builder = (yargs) => {
  yargs
    .option("env", {
      describe: "Env name",
      default: "dev",
    })
    .option("owner", {
      describe: "new owner id",
    })
    .demandOption(["owner"]);
};

exports.handler = async function (yargs) {
  const { env, owner } = yargs;
  const near = await initNear.init(env);

  const config = getEnvConfig(env);
  const contractId = config.contractId;
  const account = await near.account(config.ownerId);

  const args = {
    new_owner_id: owner,
  };
  console.table(args);

  await confirm();

  await account.functionCall({
    contractId,
    methodName: "change_owner",
    args,
    attachedDeposit: "1",
  });

  console.log("done");
  process.exit();
};
