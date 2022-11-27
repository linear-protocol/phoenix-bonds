import anyTest, { TestFn } from "ava";
import { NEAR, NearAccount, Worker } from "near-workspaces";

export const tau = 0.03;
export const alpha = 30 * 24 * 3600 * 1000; // 30 days in ms

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
    "tests/compiled-contracts/phoenix_bond_test.wasm",
    {
      method: "new",
      args: {
        owner_id: owner.accountId,
        linear_address: linear.accountId,
        alpha,
        tau: tau * 100 * 100, // 0.03 -> 3% -> 300 basis point
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
