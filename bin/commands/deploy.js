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

exports.handler = async function (args) {
  const { env, } = args;
  const near = await initNear.init(env);

  const config = getEnvConfig(env);
  const contractId = config.contractId;
  const account = await near.account(contractId);

  await account.deployContract(fs.readFileSync('res/phoenix_bonds.wasm'));
  console.log(`Contract deployed to ${contractId}`);

  const tau = parseInt(config.tau * 100);
  const bootstrap_ends = Date.now() + daysToMs(config.bootstrapLength);
  const accrual = {
    alpha: daysToMs(config.accrual.alpha),
    min_alpha: daysToMs(config.accrual.minAlpha),
    target_mean_length: daysToMs(config.accrual.targetMeanLength),
    adjust_interval: hoursToMs(config.accrual.adjustInterval),
    adjust_rate: config.accrual.adjustRate * 100,
  }

  await account.functionCall({
    contractId,
    methodName: 'new',
    args: {
      owner_id: config.ownerId,
      linear_address: config.linearAddress,
      tau,
      bootstrap_ends,
      accrual,
    }
  });

  console.log('contract initialized');
}
