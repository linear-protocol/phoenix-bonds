import { NEAR } from "near-workspaces";
import Big from "big.js";
import {
  applyNearDecimals,
  assertFailure,
  bond,
  commit,
  daysToMs,
  ftStorageDeposit,
  getFtBalance,
  getPnearPrice,
  redeem,
  setLinearPrice,
  setTimestamp,
} from "./common";
import { bootstrapEnds, init } from "./init";

const test = init();

test("Cannot redeem when bootstrapping", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  await assertFailure(
    test,
    redeem(phoenix, alice, "0"),
    "Commit and redeem are not allowed now"
  );
});

test("Cannot redeem with zero balance", async (test) => {
  const { alice, phoenix } = test.context.accounts;
  await setTimestamp(phoenix, daysToMs(20));

  await assertFailure(
    test,
    redeem(phoenix, alice, "0"),
    "The account alice.test.near is not registered"
  );
});

test("Cannot redeem more than one's balance", async (test) => {
  const { alice, phoenix } = test.context.accounts;
  const noteId = await bond(alice, phoenix, NEAR.parse("100"));

  await setTimestamp(phoenix, daysToMs(20));
  await commit(phoenix, alice, noteId);

  const pnearBalance = await getFtBalance(phoenix, alice);

  await assertFailure(
    test,
    redeem(phoenix, alice, NEAR.from(pnearBalance).addn(1).toString()),
    "Not enough pNEAR balance"
  );
});

test("Cannot burn all pNEAR", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  const noteId = await bond(alice, phoenix, NEAR.parse("3000"));

  await setTimestamp(phoenix, daysToMs(20));
  const pnearBalance = await commit(phoenix, alice, noteId);

  await assertFailure(
    test,
    redeem(phoenix, alice, pnearBalance),
    "At least one pNEAR must be left"
  );
});

test("Redeem LiNEAR", async (test) => {
  const { alice, bob, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);
  await ftStorageDeposit(linear, bob);

  // day 0
  // - alice bond 100 NEAR
  const aliceNoteId = await bond(alice, phoenix, NEAR.parse("100"));

  // day 10
  // - linear price increase to 1.01
  // - bob bond 500 NEAR
  await setTimestamp(phoenix, daysToMs(10));
  await setLinearPrice(linear, NEAR.parse("1.01").toString());
  const bobNoteId = await bond(bob, phoenix, NEAR.parse("500"));

  // day 15 (bootstrap ends)
  // - linear price increase to 1.02
  // - alice commits
  // - pNEAR price will be 1
  await setTimestamp(phoenix, bootstrapEnds);
  await setLinearPrice(linear, NEAR.parse("1.02").toString());
  await commit(phoenix, alice, aliceNoteId);

  const aliceRedeemedLinear = await redeem(
    phoenix,
    alice,
    NEAR.parse("1").toString()
  );
  // alice should get LiNEAR whose value equals to exact 1 NEAR
  test.is(aliceRedeemedLinear, applyNearDecimals("1").div("1.02").toFixed(0));

  // day 17
  // - linear price increase to 1.03
  // - bob commits
  // - pNEAR price is about 1.0744 now
  await setTimestamp(phoenix, daysToMs(17));
  await setLinearPrice(linear, NEAR.parse("1.03").toString());
  await commit(phoenix, bob, bobNoteId);

  // bob redeem 1 pNEAR
  const bobRedeemedLinear = await redeem(
    phoenix,
    bob,
    NEAR.parse("1").toString()
  );
  test.is(bobRedeemedLinear, "1043120065605160302135142");
});
