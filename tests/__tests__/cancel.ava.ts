import { NEAR, NearAccount } from "near-workspaces";
import {
  assertFailure,
  bond,
  cancel,
  commit,
  daysToMs,
  ftStorageDeposit,
  getBondNote,
  getLinearPrice,
  setLinearPrice,
  setSmallChange,
  setTimestamp,
} from "./common";
import { init } from "./init";

const test = init();

async function verifyCancel(
  test: any,
  phoenix: NearAccount,
  linear: NearAccount,
  account: NearAccount,
  noteId: number
) {
  const linearPrice = await getLinearPrice(linear);
  const bondNote = await getBondNote(phoenix, account, noteId, linearPrice);
  const linearBalanceBeforeCancel: string = await linear.view("ft_balance_of", {
    account_id: account.accountId,
  });

  const returnedLinearAmount = await cancel(phoenix, account, noteId);
  const linearBalanceAfterCancel: string = await linear.view("ft_balance_of", {
    account_id: account.accountId,
  });

  // 1. make sure the value of returned LiNEAR amount is equal to bond NEAR amount
  const expectLinearAmount = NEAR.from(bondNote.bond_amount)
    .mul(NEAR.parse("1"))
    .div(NEAR.from(linearPrice));
  test.is(
    returnedLinearAmount,
    expectLinearAmount.toString(),
    "wrong linear amount returned"
  );

  // 2. make sure correct amount of linear is transferred
  test.is(
    returnedLinearAmount,
    NEAR.from(linearBalanceAfterCancel)
      .sub(NEAR.from(linearBalanceBeforeCancel))
      .toString(),
    "wrong amount of linear transferred"
  );
}

test("Cancel should receive equivalent LiNEAR", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);

  // linear price set to 1
  let linearPrice = NEAR.parse("1").toString();
  await setLinearPrice(linear, linearPrice);

  // cancel right after bond
  const noteId1 = await bond(alice, phoenix, NEAR.parse("100"));
  await verifyCancel(test, phoenix, linear, alice, noteId1);

  // cancel after a while when linear price increased
  const noteId2 = await bond(alice, phoenix, NEAR.parse("99999"));

  // set timestamp
  await setTimestamp(phoenix, daysToMs(20));
  // linear price set to 1.2
  linearPrice = NEAR.parse("1.2").toString();
  await setLinearPrice(linear, linearPrice);

  await verifyCancel(test, phoenix, linear, alice, noteId2);
});

test("Cancel before any bond", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);

  await assertFailure(
    test,
    cancel(phoenix, alice, 0),
    "Bond note doesn't exist"
  );
});

test("Cancel wrong note id", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);

  const noteId = await bond(alice, phoenix, NEAR.parse("2000"));

  await assertFailure(
    test,
    cancel(phoenix, alice, noteId + 1),
    "Bond note doesn't exist"
  );
});

test("Cancel same note twice", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);

  const noteId = await bond(alice, phoenix, NEAR.parse("2000"));
  await verifyCancel(test, phoenix, linear, alice, noteId);

  await assertFailure(
    test,
    cancel(phoenix, alice, noteId),
    "Bond is not pending"
  );
});

test("Cancel after commit", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);

  const noteId = await bond(alice, phoenix, NEAR.parse("2000"));

  // set timestamp
  await setTimestamp(phoenix, daysToMs(20));
  await commit(phoenix, alice, noteId);
  await assertFailure(
    test,
    cancel(phoenix, alice, noteId),
    "Bond is not pending"
  );
});

test("Cancel the only bond with small change in LiNEAR", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);
  await setSmallChange(linear, true);

  const noteId = await bond(alice, phoenix, NEAR.parse("2000"));
  const refunded = await cancel(phoenix, alice, noteId);

  test.is(refunded, NEAR.parse("2000").subn(10).toString(10));
});

test("Cancel all pending bonds with small change in LiNEAR", async (test) => {
  const { alice, bob, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);
  await ftStorageDeposit(linear, bob);

  await setSmallChange(linear, true);

  const aliceNoteId1 = await bond(alice, phoenix, NEAR.parse("2000"));
  const aliceNoteId2 = await bond(alice, phoenix, NEAR.parse("100"));
  const bobNoteId1 = await bond(bob, phoenix, NEAR.parse("1000"));

  test.is(
    await cancel(phoenix, alice, aliceNoteId1),
    NEAR.parse("2000").toString(10)
  );
  test.is(
    await cancel(phoenix, alice, aliceNoteId2),
    NEAR.parse("100").toString(10)
  );
  test.is(
    await cancel(phoenix, bob, bobNoteId1),
    NEAR.parse("1000").subn(30).toString(10)
  );
});
