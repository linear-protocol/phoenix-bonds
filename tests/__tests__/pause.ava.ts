import { NEAR, NearAccount } from "near-workspaces";
import {
  assertFailure,
  bond,
  cancel,
  commit,
  daysToMs,
  ftTransfer,
  redeem,
  setTimestamp,
} from "./common";
import { init } from "./init";

const test = init();

async function pause(phoenix: NearAccount, signer: NearAccount) {
  return signer.call(
    phoenix,
    "pause",
    {},
    {
      attachedDeposit: NEAR.from("1"),
    }
  );
}

async function resume(phoenix: NearAccount, signer: NearAccount) {
  return signer.call(
    phoenix,
    "resume",
    {},
    {
      attachedDeposit: NEAR.from("1"),
    }
  );
}

test("Only owner can pause/resume", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  await assertFailure(test, pause(phoenix, alice), "Not owner");

  await assertFailure(test, resume(phoenix, alice), "Not owner");
});

test("Pause should stop all user interactions", async (test) => {
  const { alice, owner, phoenix } = test.context.accounts;

  const noteId1 = await bond(alice, phoenix, NEAR.parse("100"));

  await setTimestamp(phoenix, daysToMs(20));
  await commit(phoenix, alice, noteId1);
  const noteId2 = await bond(alice, phoenix, NEAR.parse("1"));

  await pause(phoenix, owner);

  await assertFailure(
    test,
    bond(alice, phoenix, NEAR.parse("2")),
    "Contract paused. Please try again later"
  );

  await assertFailure(
    test,
    cancel(phoenix, alice, noteId2),
    "Contract paused. Please try again later"
  );

  await assertFailure(
    test,
    commit(phoenix, alice, noteId2),
    "Contract paused. Please try again later"
  );

  await assertFailure(
    test,
    redeem(phoenix, alice, NEAR.parse("1").toString(10)),
    "Contract paused. Please try again later"
  );

  await assertFailure(
    test,
    ftTransfer(phoenix, alice, phoenix, NEAR.parse("1").toString(10)),
    "Contract paused. Please try again later"
  );
});

test("Resume", async (test) => {
  const { alice, owner, phoenix } = test.context.accounts;
  await pause(phoenix, owner);
  await resume(phoenix, owner);

  await bond(alice, phoenix, NEAR.parse("10"));
});
