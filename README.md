# phoenix-bonds
Phoenix Bonds -- a new bonding mechanism for LiNEAR.

To learn more about how Phoenix Bonds works, please visit: https://docs.linearprotocol.org/phoenix-bonds

## Contracts

The Phoenix Bonds smart contracts are implemented with [NEAR Rust SDK](https://near-sdk.io/). The core contract is located in `contracts/phoenix-bonds`, and a mock LiNEAR contract is made for testing various scenarios via simulation test.

The code has been audited by [BlockSec](https://www.blocksecteam.com/). According to [BlockSec's auditing report](https://github.com/linear-protocol/audits/blob/main/BlockSec%20-%20Security%20Audit%20Report%20for%20Phoenix%20Bonds%20-%20202301.pdf), no issues were found, and a few recommendations and notes were reported and have been acknowledged.

## Test
- `npm i`
- To run all tests: `make test`
- To run contract unit tests: `make test-unit`
- To run integration tests: `make test-integration`

## Build & Deploy
- Build release artifact: `make`
- Create a `config.js` file under `bin/env/{env}` folder
- Deploy and initialize contract: `./bin/cli.js deploy --env {env}`
- Deploy code only: `./bin/cli.js deploy --noInit --env {env}`
