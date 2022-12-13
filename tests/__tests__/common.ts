import Big from "big.js";
import { Gas, NEAR, NearAccount, TransactionResult } from "near-workspaces";

export interface BondNote {
  id: number;
  account_id: string;
  bond_amount: string;
  committed_pnear_amount: string;
  created_at: number;
  settled_at: number;
  status: string;
  cap: string;
  accrued_pnear: string;
}

export function daysToMs(n: number) {
  return n * 24 * 3600 * 1000;
}

export function applyNearDecimals(n: string) {
  return Big(n).mul(1e24);
}

export async function assertFailure(
  test: any,
  action: Promise<unknown>,
  errorMessage?: string
) {
  let failed = false;

  try {
    const results = await action;
    if (results && results instanceof TransactionResult) {
      for (const outcome of results.receipts_outcomes) {
        if (outcome.isFailure) {
          failed = true;
          if (errorMessage) {
            const actualErr = JSON.stringify(outcome.executionFailure);
            test.truthy(
              JSON.stringify(actualErr).includes(errorMessage),
              `Bad error message. expected: "${errorMessage}", actual: "${actualErr}"`
            );
          }
        }
      }
    }
  } catch (e) {
    if (errorMessage) {
      let msg: string = parseError(e);
      test.truthy(
        msg.includes(errorMessage),
        `Bad error message. expect: "${errorMessage}", actual: "${msg}"`
      );
    }
    failed = true;
  }

  test.is(failed, true, "Function call didn't fail");
}

function parseError(e: any): string {
  try {
    let status: any =
      e && e.parse ? e.parse().result.status : JSON.parse(e.message);
    return status.Failure.ActionError.kind.FunctionCallError.ExecutionError;
  } catch (_) {
    return e.message;
  }
}

// -- mock linear methods

export async function setLinearPrice(linear: NearAccount, price: string) {
  return linear.call(linear, "set_ft_price", {
    price,
  });
}

export async function getLinearPrice(linear: NearAccount): Promise<string> {
  return linear.view("ft_price", {});
}

export async function setLinearPanic(linear: NearAccount, panic: boolean) {
  return linear.call(linear, "set_panic", {
    panic,
  });
}

export async function ftStorageDeposit(ft: NearAccount, account: NearAccount) {
  return account.call(
    ft,
    "storage_deposit",
    {},
    {
      attachedDeposit: NEAR.parse("0.1"),
    }
  );
}

export async function getFtBalance(
  ft: NearAccount,
  account: NearAccount
): Promise<string> {
  return ft.view("ft_balance_of", { account_id: account.accountId });
}

// -- phoenix bonds methods

export async function ftTransfer(
  ft: NearAccount,
  from: NearAccount,
  to: NearAccount,
  amount: string
) {
  return from.call(
    ft,
    "ft_transfer",
    {
      receiver_id: to.accountId,
      amount,
    },
    {
      attachedDeposit: NEAR.from("1"),
    }
  );
}

export async function setTimestamp(phoenix: NearAccount, ts: number) {
  return phoenix.call(phoenix, "set_current_timestamp_ms", {
    ms: ts,
  });
}

export async function getPnearPrice(
  phoenix: NearAccount,
  linearPrice: string
): Promise<string> {
  return phoenix.view("get_pnear_price", {
    linear_price: linearPrice,
  });
}

export async function bond(
  account: NearAccount,
  phoenix: NearAccount,
  amount: NEAR
): Promise<number> {
  const storageDeposit = NEAR.parse("0.01");
  return account.call(
    phoenix,
    "bond",
    {},
    {
      attachedDeposit: NEAR.from(amount).add(storageDeposit).toString(),
      gas: Gas.parse("120 Tgas"),
    }
  );
}

export async function getBondNote(
  phoenix: NearAccount,
  account: NearAccount,
  noteId: number,
  linearPrice: string
): Promise<BondNote> {
  return phoenix.view("get_bond_note", {
    account_id: account.accountId,
    note_id: noteId,
    linear_price: linearPrice,
  });
}

export async function cancel(
  phoenix: NearAccount,
  account: NearAccount,
  noteId: number
): Promise<string> {
  return account.call(
    phoenix,
    "cancel",
    {
      note_id: noteId,
    },
    {
      attachedDeposit: NEAR.from("1"),
      gas: Gas.parse("160 Tgas"),
    }
  );
}

export async function commit(
  phoenix: NearAccount,
  account: NearAccount,
  noteId: number
): Promise<string> {
  return account.call(
    phoenix,
    "commit",
    {
      note_id: noteId,
    },
    {
      attachedDeposit: NEAR.from("1"),
      gas: Gas.parse("90 Tgas"),
    }
  );
}

export async function redeem(
  phoenix: NearAccount,
  account: NearAccount,
  amount: string
): Promise<string> {
  return account.call(
    phoenix,
    "redeem",
    {
      amount,
    },
    {
      attachedDeposit: NEAR.from("1"),
      gas: Gas.parse("160 Tgas"),
    }
  );
}

export async function getUserLostAndFound(
  phoenix: NearAccount,
  account: NearAccount
): Promise<string> {
  return phoenix.view("user_lost_and_found", { account_id: account.accountId });
}

export async function claimLostAndFound(
  phoenix: NearAccount,
  account: NearAccount
): Promise<string> {
  return account.call(
    phoenix,
    "claim_lost_and_found",
    {},
    {
      gas: Gas.parse("100 Tgas"),
    }
  );
}
