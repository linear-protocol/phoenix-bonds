#!/usr/bin/env node
require('yargs/yargs')(process.argv.slice(2))
  .commandDir('commands')
  .env("PHOENIX")
  .demandCommand()
  .help()
  .argv
