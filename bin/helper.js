function getEnvConfig(env) {
  return require(`./env/${env}/config.js`);
}

function daysToMs(n) {
  return parseInt(n * 24 * 3600 * 1000);
}

function hoursToMs(n) {
  return parseInt(n * 3600 * 1000);
}

module.exports = {
  getEnvConfig,
  daysToMs,
  hoursToMs,
};
