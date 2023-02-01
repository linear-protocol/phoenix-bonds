import { Gas, NEAR, NearAccount } from "near-workspaces";
import Big from "big.js";
import {
  assertFailure,
  bond,
  BondNote,
  bondWithLinear,
  ftStorageDeposit,
  getBondNote,
  getFtBalance,
  getLinearPrice,
  setLinearPanic,
} from "./common";
import { init, tau } from "./init";

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

const test = init();

// ---- Bond with NEAR

test("Bond with small amount", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  // bond less than 0.01
  await assertFailure(
    test,
    alice.call(
      phoenix,
      "bond",
      {},
      {
        gas: Gas.parse("120 Tgas"),
        attachedDeposit: NEAR.parse("0.009").toString(),
      }
    ),
    "Bond requires 0.01 NEAR as storage deposit"
  );

  await assertFailure(
    test,
    bond(alice, phoenix, NEAR.parse("0.09")),
    "Bond amount must be at least 0.1 NEAR"
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

test("Bond failed and all deposits are refunded", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;

  // bond will panic
  await setLinearPanic(linear, true);

  const balanceBefore = await alice.availableBalance();
  // 900K NEAR + 0.01 NEAR storage fee
  const noteId: any = await bond(alice, phoenix, NEAR.parse("900000.01"));
  test.is(noteId, null);

  const balanceAfter = await alice.availableBalance();
  // the gas cost should be less than 0.002N
  test.true(
    Big(balanceBefore.sub(balanceAfter).toString()).lt(
      Big(NEAR.parse("0.002").toString())
    )
  );
});

// ---- Bond with LiNEAR

async function mintLinear(
  account: NearAccount,
  linear: NearAccount,
  amount: string
) {
  await account.call(
    linear,
    "deposit_and_stake",
    {},
    {
      attachedDeposit: amount,
    }
  );
}

test("Wrong token transferred", async (test) => {
  const { alice, phoenix, fakeLinear } = test.context.accounts;
  const amount = NEAR.parse("1000");

  // mint some fake linear
  await mintLinear(alice, fakeLinear, amount.toString(10));
  await ftStorageDeposit(fakeLinear, phoenix);

  await bondWithLinear(alice, phoenix, fakeLinear, amount.toString(10));

  // all fake tokens should be refunded
  test.is(await getFtBalance(fakeLinear, alice), amount.toString(10));
});

test("LiNEAR amount too low", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  const amount = NEAR.parse("1");

  await mintLinear(alice, linear, amount.toString(10));
  await ftStorageDeposit(linear, phoenix);

  await bondWithLinear(alice, phoenix, linear, NEAR.parse("0.09").toString(10));

  test.is(await getFtBalance(linear, alice), amount.toString(10));
});

test("Bond with LiNEAR", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  const amount = NEAR.parse("1000");

  await mintLinear(alice, linear, amount.toString(10));
  await ftStorageDeposit(linear, phoenix);

  const usedAmount = await bondWithLinear(
    alice,
    phoenix,
    linear,
    amount.toString(10)
  );

  test.is(usedAmount, amount.toString(10));

  const note = await getBondNote(
    phoenix,
    alice,
    0,
    NEAR.parse("1").toString(10)
  );
  test.is(
    note.bond_amount,
    amount.sub(NEAR.parse("0.01")).toString(10) // 0.01 NEAR as storage deposit
  );
});
