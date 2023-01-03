import { NEAR, NearAccount, Gas } from "near-workspaces";
import {
  assertFailure,
  bond,
  commit,
  setLinearPrice,
  setTimestamp,
} from "./common";
import { bootstrapEnds, init } from "./init";

const ERR_NO_ENOUGH_BALANCE = "The account doesn't have enough balance";
const ERR_UNREGISTER_WITH_BALANCE =
  "Can't unregister the account with the positive balance without force";

const ONE_YOCTO = NEAR.from("1");

async function transfer(
  contract: NearAccount,
  sender: NearAccount,
  receiver: NearAccount,
  amount: NEAR
) {
  await sender.call(
    contract,
    "ft_transfer",
    {
      receiver_id: receiver,
      amount,
    },
    {
      attachedDeposit: ONE_YOCTO,
    }
  );
}

async function transferCall(
  contract: NearAccount,
  sender: NearAccount,
  receiver: NearAccount,
  amount: NEAR,
  msg: String,
  memo?: String
) {
  await sender.call(
    contract,
    "ft_transfer_call",
    {
      receiver_id: receiver,
      amount,
      memo,
      msg,
    },
    {
      gas: Gas.parse("75 Tgas"),
      attachedDeposit: ONE_YOCTO,
    }
  );
}

async function register(
  ft: NearAccount,
  user: NearAccount,
  storage_cost?: NEAR
) {
  const storage_balance = (await ft.view("storage_balance_bounds", {})) as any;
  await user.call(
    ft,
    "storage_deposit",
    { account_id: user },
    {
      attachedDeposit:
        storage_cost?.toString() || storage_balance.min.toString(),
    }
  );
}

async function bondAndCommit(
  phoenix: NearAccount,
  linear: NearAccount,
  user: NearAccount,
  timestamp: number = bootstrapEnds,
  linearPrice: number = 1.01,
  amount: number = 100
) {
  const aliceNoteId = await bond(user, phoenix, NEAR.parse(amount.toFixed()));

  await setTimestamp(phoenix, timestamp);
  await setLinearPrice(linear, NEAR.parse(linearPrice.toFixed(0)).toString());

  return NEAR.from(await commit(phoenix, user, aliceNoteId));
}

const test = init();

test("read ft metadata", async (test) => {
  const { phoenix } = test.context.accounts;
  const metadata = (await phoenix.view("ft_metadata", {})) as any;
  test.is(metadata.symbol, "pNEAR");
  test.is(metadata.decimals, 24);
});

test("pNEAR price", async (test) => {
  const { phoenix } = test.context.accounts;
  const price = (await phoenix.view("get_pnear_price", {
    linear_price: NEAR.parse("1"),
  })) as any;
  test.is(NEAR.from(price).toString(), NEAR.parse("1").toString());
});

test("cannot transfer with no balance", async (test) => {
  const { phoenix, alice, bob } = test.context.accounts;
  await register(phoenix, alice);

  await assertFailure(
    test,
    transfer(phoenix, alice, bob, NEAR.parse("1")),
    ERR_NO_ENOUGH_BALANCE
  );
});

test("commit and transfer pNEAR", async (test) => {
  const { phoenix, linear, alice, bob } = test.context.accounts;

  await register(phoenix, alice);
  await register(phoenix, bob);

  const committedPNEAR = await bondAndCommit(phoenix, linear, alice);

  // transfer 2 pNEAR from alice to bob
  const transferAmount1 = NEAR.parse("2");
  await transfer(phoenix, alice, bob, transferAmount1);
  test.is(
    await phoenix.view("ft_balance_of", { account_id: alice }),
    committedPNEAR.sub(transferAmount1).toString()
  );
  test.is(
    await phoenix.view("ft_balance_of", { account_id: bob }),
    transferAmount1.toString()
  );

  // transfer 1 pNEAR from bob to alice
  const transferAmount2 = NEAR.parse("1");
  await transfer(phoenix, bob, alice, transferAmount2);
  test.is(
    await phoenix.view("ft_balance_of", { account_id: alice }),
    committedPNEAR.sub(transferAmount1).add(transferAmount2).toString()
  );
  test.is(
    await phoenix.view("ft_balance_of", { account_id: bob }),
    transferAmount1.sub(transferAmount2).toString()
  );

  // cannot transfer 2 pNEAR from bob
  await assertFailure(
    test,
    transfer(phoenix, bob, alice, NEAR.parse("2")),
    ERR_NO_ENOUGH_BALANCE
  );
});

// Ensure pNEAR transfer work well with NEAR Wallet
test("register pNEAR with 0.00125Ⓝ storage balance", async (test) => {
  const { phoenix, linear, alice, bob } = test.context.accounts;

  await register(phoenix, alice, NEAR.parse("0.00125"));
  await register(phoenix, bob, NEAR.parse("0.00125"));

  const committedPNEAR = await bondAndCommit(phoenix, linear, alice);

  // transfer 2 pNEAR from alice to bob
  const transferAmount1 = NEAR.parse("2");
  await transfer(phoenix, alice, bob, transferAmount1);
  test.is(
    await phoenix.view("ft_balance_of", { account_id: alice }),
    committedPNEAR.sub(transferAmount1).toString()
  );
  test.is(
    await phoenix.view("ft_balance_of", { account_id: bob }),
    transferAmount1.toString()
  );
});

test("storage unregister", async (test) => {
  const { phoenix, linear, alice, bob } = test.context.accounts;

  await register(phoenix, alice);
  await register(phoenix, bob);

  test.is(
    ((await phoenix.view("storage_balance_of", { account_id: alice })) as any)
      .total,
    NEAR.parse("0.00125").toString()
  );

  // Unregister Alice
  await alice.call(
    phoenix,
    "storage_unregister",
    {},
    { attachedDeposit: ONE_YOCTO }
  );
  test.is(
    await phoenix.view("storage_balance_of", { account_id: alice }),
    null
  );

  // Alice bond NEAR and commit pNEAR
  await bondAndCommit(phoenix, linear, alice);

  // Force unregister Alice successfully.
  // The $pNEAR owned by Alice are all burnt. Now $pNEAR price increased to 2 $NEAR.
  await alice.call(
    phoenix,
    "storage_unregister",
    { force: true },
    { attachedDeposit: ONE_YOCTO }
  );
  test.is(
    await phoenix.view("storage_balance_of", { account_id: alice }),
    null
  );
  test.is(await phoenix.view("ft_balance_of", { account_id: alice }), "0");

  // Alice bond NEAR and commit pNEAR
  const committedPNEAR = await bondAndCommit(
    phoenix,
    linear,
    alice,
    bootstrapEnds * 2,
    1.03,
    100
  );

  // transfer 1 pNEAR from alice to bob
  await transfer(phoenix, alice, bob, NEAR.parse("1"));

  // unregister failed because balance is not zero
  await assertFailure(
    test,
    alice.call(
      phoenix,
      "storage_unregister",
      {},
      { attachedDeposit: ONE_YOCTO }
    ),
    ERR_UNREGISTER_WITH_BALANCE
  );

  // transfer all pNEAR from alice to bob
  await transfer(phoenix, alice, bob, committedPNEAR.sub(NEAR.parse("1")));

  // Now Alice could unregister successfully
  await alice.call(
    phoenix,
    "storage_unregister",
    {},
    { attachedDeposit: ONE_YOCTO }
  );
  test.is(
    await phoenix.view("storage_balance_of", { account_id: alice }),
    null
  );
});
