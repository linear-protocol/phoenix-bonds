import { NEAR } from "near-workspaces";
import {
  bond,
  getBondNote,
  getLinearPrice,
  notesCount,
  pendingNotesCount,
  cancel,
  setTimestamp,
  daysToMs,
  commit,
  listPendingNotes,
} from "./common";
import { init } from "./init";

const test = init();

test("notes count increase when a new note is created", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  test.deepEqual(await notesCount(phoenix, alice), 0);

  const noteCount = 2;
  for (let n = 0; n < noteCount; n++) {
    await bond(alice, phoenix, NEAR.parse("100"));
    test.deepEqual(await notesCount(phoenix, alice), n + 1);
  }
});

test("pending notes count increase when a new note is created", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  test.deepEqual(await pendingNotesCount(phoenix, alice), 0);

  const noteCount = 2;
  for (let n = 0; n < noteCount; n++) {
    await bond(alice, phoenix, NEAR.parse("100"));
    test.deepEqual(await pendingNotesCount(phoenix, alice), n + 1);
  }
});

test("pending notes count decrease when a note is canceled", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  test.deepEqual(await pendingNotesCount(phoenix, alice), 0);

  const noteCount = 2;
  for (let n = 0; n < noteCount; n++) {
    await bond(alice, phoenix, NEAR.parse("100"));
  }

  test.deepEqual(await pendingNotesCount(phoenix, alice), noteCount);
  await cancel(phoenix, alice, 0);
  test.deepEqual(await pendingNotesCount(phoenix, alice), noteCount - 1);
});

test("pending notes count decrease when a note is committed", async (test) => {
  const { alice, phoenix } = test.context.accounts;

  test.deepEqual(await pendingNotesCount(phoenix, alice), 0);

  const noteCount = 2;
  for (let n = 0; n < noteCount; n++) {
    await bond(alice, phoenix, NEAR.parse("100"));
  }

  await setTimestamp(phoenix, daysToMs(20));

  test.deepEqual(await pendingNotesCount(phoenix, alice), noteCount);
  await commit(phoenix, alice, 0);
  test.deepEqual(await pendingNotesCount(phoenix, alice), noteCount - 1);
});

test("list pending notes", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;

  test.deepEqual(await pendingNotesCount(phoenix, alice), 0);

  const linearPrice = await getLinearPrice(linear);

  const notes = [];
  const noteCount = 3;
  for (let n = 0; n < noteCount; n++) {
    const noteId = await bond(alice, phoenix, NEAR.parse("100"));
    notes.push(await getBondNote(phoenix, alice, noteId, linearPrice));
  }

  test.deepEqual(
    notes,
    await listPendingNotes(phoenix, alice, linearPrice, 0, 3)
  );
  test.deepEqual(
    notes,
    await listPendingNotes(phoenix, alice, linearPrice, 0, 100)
  );
  test.deepEqual(
    [notes[0]],
    await listPendingNotes(phoenix, alice, linearPrice, 0, 1)
  );
  test.deepEqual(
    [notes[1]],
    await listPendingNotes(phoenix, alice, linearPrice, 1, 1)
  );
  test.deepEqual(
    [notes[2]],
    await listPendingNotes(phoenix, alice, linearPrice, 2, 1)
  );

  await cancel(phoenix, alice, 2);
  notes.pop();
  test.deepEqual(
    notes,
    await listPendingNotes(phoenix, alice, linearPrice, 0, 100)
  );

  await setTimestamp(phoenix, daysToMs(20));
  await commit(phoenix, alice, 1);

  notes.pop();
  notes[0].accrued_pnear = "84890497305586676567721853";
  test.deepEqual(
    notes,
    await listPendingNotes(phoenix, alice, linearPrice, 0, 100)
  );
});
