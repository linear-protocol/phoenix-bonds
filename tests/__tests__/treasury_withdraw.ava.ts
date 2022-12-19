import { Gas, NEAR } from "near-workspaces";
import {
  applyNearDecimals,
  assertFailure,
  bond,
  commit,
  daysToMs,
  ftStorageDeposit,
  getFtBalance,
  setLinearPrice,
  setTimestamp,
} from "./common";
import { init, tau } from "./init";

const test = init();

test("Only owner can withdraw", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  await assertFailure(
    test,
    alice.call(
      phoenix,
      "withdraw_treasury",
      {},
      {
        attachedDeposit: NEAR.from("1"),
        gas: Gas.parse("160 Tgas"),
      }
    ),
    "Not owner"
  );
});

test("Withdraw", async (test) => {
  const { alice, phoenix, linear, owner } = test.context.accounts;
  await ftStorageDeposit(linear, owner);

  await setTimestamp(phoenix, daysToMs(20));
  const noteId = await bond(alice, phoenix, NEAR.parse("4000"));

  await setTimestamp(phoenix, daysToMs(30));
  await commit(phoenix, alice, noteId);

  await setLinearPrice(linear, NEAR.parse("1.1").toString());

  await owner.call(
    phoenix,
    "withdraw_treasury",
    {},
    {
      attachedDeposit: NEAR.from("1"),
      gas: Gas.parse("160 Tgas"),
    }
  );

  test.is(
    await getFtBalance(linear, owner),
    applyNearDecimals("4000").mul(tau).div(1.1).toFixed(0)
  );
});
