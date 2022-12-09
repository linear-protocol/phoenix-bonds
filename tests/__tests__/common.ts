import { NEAR, NearAccount, TransactionResult } from "near-workspaces";

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

export function daysToMs(days: number) {
  return days * 24 * 3600 * 1000;
}

export async function bond(
  account: NearAccount,
  phoenix: NearAccount,
  amount: NEAR
): Promise<number> {
  return account.call(
    phoenix,
    "bond",
    {},
    {
      attachedDeposit: amount,
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
