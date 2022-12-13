import { NEAR } from "near-workspaces";
import {
  assertFailure,
  bond,
  cancel,
  claimLostAndFound,
  commit,
  daysToMs,
  ftStorageDeposit,
  ftTransfer,
  redeem,
  setLinearPanic,
  setTimestamp,
} from "./common";
import { init } from "./init";

const test = init();

test("LiNEAR transfer failed when cancel", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;

  const noteId = await bond(alice, phoenix, NEAR.parse("9999"));
  // linear transfer would fail due to no storage deposit
  const transferredAmount = await cancel(phoenix, alice, noteId);
  test.is(transferredAmount, "0");

  await ftStorageDeposit(linear, alice);
  const claimedLostAndFound = await claimLostAndFound(phoenix, alice);
  test.is(claimedLostAndFound, NEAR.parse("9999").toString());

  // try to claim again
  await assertFailure(
    test,
    claimLostAndFound(phoenix, alice),
    "No lost and found LiNEAR to claim"
  );
});

test("LiNEAR transfer failed when redeem", async (test) => {
  const { alice, bob, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);

  const noteId = await bond(alice, phoenix, NEAR.parse("9999"));
  await setTimestamp(phoenix, daysToMs(20));
  await commit(phoenix, alice, noteId);

  // transfer pNEAR to bob
  await ftStorageDeposit(phoenix, bob);
  await ftTransfer(phoenix, alice, bob, NEAR.parse("10").toString());

  // bob redeem
  // linear transfer would fail due to no storage deposit
  const redeemedLinear = await redeem(
    phoenix,
    bob,
    NEAR.parse("10").toString()
  );
  test.is(redeemedLinear, "0");

  await ftStorageDeposit(linear, bob);
  const claimedLostAndFound = await claimLostAndFound(phoenix, bob);
  test.is(claimedLostAndFound, NEAR.parse("10").toString());

  // try to claim again
  await assertFailure(
    test,
    claimLostAndFound(phoenix, bob),
    "No lost and found LiNEAR to claim"
  );
});
