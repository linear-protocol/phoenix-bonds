const { getEnvConfig, daysToMs, hoursToMs } = require("../helper");
const initNear = require("../near");
const fs = require('fs');

exports.command = 'deploy';
exports.desc = 'Deploy and init new contract';
exports.builder = yargs => {
  yargs
    .option('env', {
      describe: 'Env name',
      default: 'dev'
    })
}

exports.handler = async function (yargs) {
  const { env, } = yargs;
  const near = await initNear.init(env);

  const config = getEnvConfig(env);
  const contractId = config.contractId;
  const account = await near.account(contractId);

  await account.deployContract(fs.readFileSync('res/phoenix_bonds.wasm'));
  console.log(`Contract deployed to ${contractId}`);

  const tau = parseInt(config.tau * 10000);
  const bootstrap_ends = (parseInt(Date.now() / 3600000) + 1) * 3600000 
    + hoursToMs(config.bootstrapLength);
  const accrual = {
    alpha: daysToMs(config.accrual.alpha),
    min_alpha: daysToMs(config.accrual.minAlpha),
    target_mean_length: daysToMs(config.accrual.targetMeanLength),
    adjust_interval: hoursToMs(config.accrual.adjustInterval),
    adjust_rate: config.accrual.adjustRate * 10000,
  }

  const args = {
    owner_id: config.ownerId,
    linear_address: config.linearAddress,
    tau,
    bootstrap_ends,
    accrual,
  };
  console.dir(args);

  // await account.functionCall({
  //   contractId,
  //   methodName: 'new',
  //   args,
  // });

  console.log('contract initialized');
}
