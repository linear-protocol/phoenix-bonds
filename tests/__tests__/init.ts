import anyTest, { TestFn } from "ava";
import { NEAR, NearAccount, Worker } from "near-workspaces";
import { daysToMs } from "./common";

export const tau = 0.03;
export const alpha = 3 * 24 * 3600 * 1000; // 3 days in ms
export const bootstrapEnds = daysToMs(15);

export function init() {
  const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
  }>;

  test.beforeEach(async (t) => {
    const worker = await Worker.init();
    const accounts = await initFixtures(worker.rootAccount);
    t.context = {
      ...t.context,
      worker,
      accounts,
    };
  });

  test.afterEach(async (t) => {
    // Stop Sandbox server
    await t.context.worker.tearDown().catch((error) => {
      console.log("Failed to tear down the worker:", error);
    });
  });

  return test;
}

async function initFixtures(root: NearAccount) {
  const alice = await root.createSubAccount("alice", {
    initialBalance: NEAR.parse("1000000").toString(),
  });
  const bob = await root.createSubAccount("bob", {
    initialBalance: NEAR.parse("1000000").toString(),
  });

  const linear = await initMockLinear(root);
  const { owner, phoenix } = await initPhoenixBonds(root, linear);

  return {
    alice,
    bob,
    linear,
    owner,
    phoenix,
  };
}

async function initMockLinear(root: NearAccount) {
  return createAndDeploy(
    root,
    "linear",
    "tests/compiled-contracts/mock_linear.wasm",
    {
      method: "new",
      args: {},
    }
  );
}

async function initPhoenixBonds(root: NearAccount, linear: NearAccount) {
  const owner = await root.createSubAccount("owner");

  const phoenix = await createAndDeploy(
    root,
    "phoenix",
    "tests/compiled-contracts/phoenix_bonds_test.wasm",
    {
      method: "new",
      args: {
        owner_id: owner.accountId,
        linear_address: linear.accountId,
        tau: tau * 100 * 100, // 0.03 -> 3% -> 300 basis point
        bootstrap_ends: bootstrapEnds,
        accrual: {
          alpha,
          min_alpha: 1,
          target_mean_length: daysToMs(15),
          adjust_interval: daysToMs(1),
          adjust_rate: 100, // 1%
        },
      },
    }
  );

  return {
    owner,
    phoenix,
  };
}

async function createAndDeploy(
  root: NearAccount,
  accountId: string,
  wasmFile: string,
  option?: {
    method: string;
    args: any;
  }
): Promise<NearAccount> {
  const contract = await root.createSubAccount(accountId, {
    initialBalance: NEAR.parse("1000000").toString(),
  });
  await contract.deploy(wasmFile);
  if (option) {
    await contract.call(contract, option.method, option.args);
  }
  return contract;
}
