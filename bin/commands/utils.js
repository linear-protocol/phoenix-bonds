const readline = require('readline');

async function confirm () {
  const rl = readline.createInterface({ input: process.stdin, output: process.stdout });
  await new Promise((resolve, _) => {
    rl.question('Confirm?', resolve);
  });
}

module.exports = {
  confirm,
}
