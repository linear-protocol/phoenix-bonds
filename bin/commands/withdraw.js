const { Gas } = require("near-workspaces");
const { getEnvConfig, storageDeposit } = require("../helper");
const { init, funcCallProposal } = require("../near");

exports.command = 'withdraw';
exports.desc = 'Propose to withdraw treasury pool'
exports.builder = yargs => {
  yargs
    .option('env', {
      describe: 'Env name',
      default: 'dev'
    })
    .option('signer', {
      describe: 'Signer account ID'
    })
    .demandOption(['signer'])
};

exports.handler = async function (yargs) {
  const { env, signer } = yargs;
  const near = await init(env);

  const config = getEnvConfig(env);
  const contractId = config.contractId;
  const signerAccount = await near.account(signer);

  await storageDeposit(signerAccount, config.linearAddress, contractId);

  await funcCallProposal(
    signerAccount,
    config.ownerId,
    'Withdraw treasury pool (160 Tgas)',
    contractId,
    'withdraw_treasury',
    {},
    '1',
    Gas.parse('160 Tgas')
  );

  console.log('proposed');
}
