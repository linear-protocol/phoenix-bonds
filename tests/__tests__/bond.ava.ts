import { NEAR, NearAccount } from "near-workspaces";
import Big from "big.js";
import {
  assertFailure,
  bond,
  BondNote,
  getBondNote,
  getLinearPrice,
} from "./common";
import { init, tau } from "./init";

const test = init();

test("Bond with small amount", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  await assertFailure(
    test,
    bond(alice, phoenix, NEAR.parse("0.09")),
    "Bond requires at least 0.1 NEAR"
  );
});

test("Bond with multiple accounts", async (test) => {
  const { alice, bob, phoenix, linear } = test.context.accounts;
  const linearPrice = await getLinearPrice(linear);

  const noteId = await bond(alice, phoenix, NEAR.parse("100"));
  const aliceNote = await getBondNote(phoenix, alice, noteId, linearPrice);
  verifyNewBondNote(test, aliceNote, alice, NEAR.parse("100").toString(), tau);

  const bobNoteId = await bond(bob, phoenix, NEAR.parse("30000"));
  const bobNote = await getBondNote(phoenix, bob, bobNoteId, linearPrice);
  verifyNewBondNote(test, bobNote, bob, NEAR.parse("30000").toString(), tau);
});

test("Bond multiple times", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  const linearPrice = await getLinearPrice(linear);

  const noteId1 = await bond(alice, phoenix, NEAR.parse("200"));
  const note1 = await getBondNote(phoenix, alice, noteId1, linearPrice);
  verifyNewBondNote(test, note1, alice, NEAR.parse("200").toString(), tau);

  const noteId2 = await bond(alice, phoenix, NEAR.parse("9999"));
  const note2 = await getBondNote(phoenix, alice, noteId2, linearPrice);
  verifyNewBondNote(test, note2, alice, NEAR.parse("9999").toString(), tau);
});

function verifyNewBondNote(
  test: any,
  note: BondNote,
  account: NearAccount,
  amount: string,
  tau: number
) {
  test.is(note.account_id, account.accountId);
  test.is(note.bond_amount, amount.toString());
  test.is(note.committed_pnear_amount, "0");
  test.is(note.settled_at, 0);
  test.is(note.status, "Pending");
  test.is(
    note.cap,
    Big(amount)
      .mul(1 - tau)
      .toFixed()
  );
}
