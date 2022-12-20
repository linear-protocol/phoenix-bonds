import { NEAR } from "near-workspaces";
import {
  applyNearDecimals,
  assertFailure,
  bond,
  commit,
  daysToMs,
  setLinearPrice,
  setTimestamp,
} from "./common";
import { alpha, bootstrapEnds, init, tau } from "./init";

const test = init();

test("Cannot commit when still bootstrapping", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  const noteId = await bond(alice, phoenix, NEAR.parse("1000"));
  await assertFailure(
    test,
    commit(phoenix, alice, noteId),
    "Commit and redeem are not allowed now"
  );
});

test("Commit right after bond would get nothing", async (test) => {
  const { alice, phoenix } = test.context.accounts;
  await setTimestamp(phoenix, daysToMs(20));

  const noteId = await bond(alice, phoenix, NEAR.parse("1000"));
  const pnearAmount = await commit(phoenix, alice, noteId);

  test.is(pnearAmount, "0");
});

test("Cannot commit twice", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  const noteId = await bond(alice, phoenix, NEAR.parse("1000"));
  await setTimestamp(phoenix, daysToMs(20));
  await commit(phoenix, alice, noteId);

  await assertFailure(
    test,
    commit(phoenix, alice, noteId),
    "Bond is not pending"
  );
});

test("Commit wrong note id", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  const noteId = await bond(alice, phoenix, NEAR.parse("1000"));
  await setTimestamp(phoenix, daysToMs(20));
  await assertFailure(
    test,
    commit(phoenix, alice, noteId + 1),
    "Bond note doesn't exist"
  );
});

test("Commit at time that equals to alpha should get half cap", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  // bond at day 20
  await setTimestamp(phoenix, daysToMs(20));
  const noteId = await bond(alice, phoenix, NEAR.parse("1000"));

  // commit at day 20 + alpha
  await setTimestamp(phoenix, daysToMs(20) + alpha);
  const pnearAmount = await commit(phoenix, alice, noteId);

  // since it's the first commit, pnear price would be 1
  // so alice can get (1000 * (1 - tau) / 2) pNEAR
  test.is(
    pnearAmount,
    applyNearDecimals("1000")
      .mul(1 - tau)
      .div(2)
      .toFixed()
  );
});

test("Commit multiple bonds", async (test) => {
  const { alice, bob, phoenix, linear } = test.context.accounts;

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
  await setTimestamp(phoenix, bootstrapEnds);
  await setLinearPrice(linear, NEAR.parse("1.02").toString());
  test.is(
    await commit(phoenix, alice, aliceNoteId),
    // 100 * (1 - tau) * length / (length + alpha)
    // where length is 15 days
    applyNearDecimals("100")
      .mul(1 - tau)
      .mul(bootstrapEnds)
      .div(bootstrapEnds + alpha)
      .toFixed(0)
  );

  // day 17
  // - linear price increase to 1.03
  // - bob commits
  await setTimestamp(phoenix, daysToMs(17));
  await setLinearPrice(linear, NEAR.parse("1.03").toString());
  test.is(
    await commit(phoenix, bob, bobNoteId),
    // pnear price is about 1.0736, so cap is: 500 * (1 - tau) / pnear_price ~= 451.75
    // committed pnear would be cap * length / (length + alpha) ~= 316.22, where length is 7 days
    "316221549314521496073472772"
  );
});

test("Commit after 10 years", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  // bond at day 20
  await setTimestamp(phoenix, daysToMs(20));
  const noteId = await bond(alice, phoenix, NEAR.parse("1000"));

  // commit after 10 years
  await setTimestamp(phoenix, daysToMs(365 * 10 + 20));
  const pnearAmount = await commit(phoenix, alice, noteId);

  const summary: any = await phoenix.view("get_summary", {
    linear_price: NEAR.parse("1.01").toString(),
  });
  // current alpha has decreased to the minimum after 10 years
  const curAlpha = summary.alpha;
  test.is(
    curAlpha,
    1 // the minimum alpha
  );

  // since it's the first commit, pnear price would be 1
  const TEN_YEARS = daysToMs(365 * 10);
  test.is(
    pnearAmount,
    applyNearDecimals("1000")
      .mul(1 - tau)
      .mul(TEN_YEARS)
      .div(TEN_YEARS + curAlpha)
      .toFixed(0)
  );
});
