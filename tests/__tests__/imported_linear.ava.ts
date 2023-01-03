import { NEAR } from "near-workspaces";
import { bond, cancel, ftStorageDeposit } from "./common";
import { init } from "./init";

const test = init(true);

test("Cancel the only bond with imported LiNEAR", async (test) => {
  const { alice, phoenix, linear } = test.context.accounts;
  await ftStorageDeposit(linear, alice);

  const noteId = await bond(alice, phoenix, NEAR.parse("100"));

  await cancel(phoenix, alice, noteId);
});
