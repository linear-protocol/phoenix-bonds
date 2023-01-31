const { Gas } = require("near-workspaces");
const { storageDeposit, getEnvConfig } = require("../helper");
const { init, funcCallProposal } = require("../near");

exports.command = "transfer-rewards";
exports.desc = "Propose to transfer LiNEAR farming rewards from DAO";
exports.builder = (yargs) => {
  yargs
    .option("env", {
      describe: "Env name",
      default: "dev",
    })
    .option("signer", {
      describe: "Signer account ID",
    })
    .option("receiver", {
      describe: "Receiver account ID",
    })
    .option("amount", {
      describe: "LiNEAR amount in full decimals",
    })
    .demandOption(["signer", "receiver", "amount"]);
};

exports.handler = async function (yargs) {
  const { env, signer, receiver, amount } = yargs;
  const near = await init(env);

  const config = getEnvConfig(env);
  const signerAccount = await near.account(signer);

  await storageDeposit(signerAccount, config.linearAddress, receiver);

  await funcCallProposal(
    signerAccount,
    config.ownerId,
    "Transfer treasury LiNEAR",
    config.linearAddress,
    "ft_transfer",
    {
      receiver_id: receiver,
      amount,
    },
    "1",
    Gas.parse("150 Tgas")
  );

  console.log("proposed");
};
