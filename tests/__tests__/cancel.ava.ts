import { NEAR, NearAccount } from "near-workspaces";
import {
  assertFailure,
  bond,
  cancel,
  daysToMs,
  ftStorageDeposit,
  getBondNote,
  getLinearPrice,
  setLinearPrice,
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
