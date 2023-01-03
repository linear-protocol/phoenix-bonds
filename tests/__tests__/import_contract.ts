import { NearAccount } from "near-workspaces";
import fs from "fs";
import { RecordBuilder } from "near-workspaces/dist/record";

const IMPORTED_CONTRACT_CACHE_DIR = "./tests/imported-contracts/";

export async function importContract({
  creator,
  mainnetContract,
  testnetContract,
  blockId,
  withData = false,
  initialBalance,
}: {
  creator: NearAccount;
  mainnetContract?: string;
  testnetContract?: string;
  blockId: number;
  withData?: boolean;
  initialBalance?: string;
}): Promise<NearAccount> {
  if (
    (testnetContract && mainnetContract) ||
    !(testnetContract || mainnetContract)
  ) {
    throw new TypeError(
      "Provide `mainnetContract` or `testnetContract` but not both."
    );
  }

  // contract account can be from mainnet or testnet
  let contractAccount = (mainnetContract ?? testnetContract)!;
  let accountFilePath =
    IMPORTED_CONTRACT_CACHE_DIR +
    contractAccount +
    "_" +
    blockId +
    "_account.json";
  let codeFilePath =
    IMPORTED_CONTRACT_CACHE_DIR +
    contractAccount +
    "_" +
    blockId +
    "_code.base64";
  let statFilePath =
    IMPORTED_CONTRACT_CACHE_DIR +
    contractAccount +
    "_" +
    blockId +
    "_state.json";

  if (!fs.existsSync(codeFilePath)) {
    // Import contract from archival node
    const contract = await creator.importContract({
      mainnetContract,
      testnetContract,
      blockId,
      withData,
    });

    const accountView = await contract.accountView();

    if (!fs.existsSync(IMPORTED_CONTRACT_CACHE_DIR)) {
      fs.mkdirSync(IMPORTED_CONTRACT_CACHE_DIR);
    }
    fs.writeFileSync(accountFilePath, JSON.stringify(accountView));
    const codeRaw = await contract.viewCodeRaw();
    fs.writeFileSync(codeFilePath, codeRaw);
    if (withData) {
      // Contract state is in the form of JSON (key-value pair)
      const stateRaw = await contract.viewStateRaw();
      fs.writeFileSync(statFilePath, JSON.stringify(stateRaw));
    }
    return contract;
  } else {
    const accountView = JSON.parse(fs.readFileSync(accountFilePath, "utf8"));
    accountView.amount = initialBalance ?? accountView.amount;
    const codeRaw = fs.readFileSync(codeFilePath, "utf8");

    // Load account with info and code
    const account = creator.getAccount(contractAccount);
    const pubKey = await account.setKey(undefined);
    const records = RecordBuilder.fromAccount(account)
      .account(accountView)
      .accessKey(pubKey);
    records.contract(codeRaw);
    await account.patchStateRecords(records);
    if (!(await account.viewCode())) {
      await account.patchStateRecords(records);
      if (!(await account.viewCode())) {
        throw new Error(
          `Account ${contractAccount} does not exist after trying to patch into sandbox.`
        );
      }
    }
    console.log(`  ✅ Loaded contract code cache of [${contractAccount}]`);

    if (withData) {
      // Load contract state
      const rawState = JSON.parse(fs.readFileSync(statFilePath, "utf8"));
      const state = rawState.map(
        ({ key, value }: { key: string; value: string }) => ({
          Data: {
            account_id: account.accountId,
            data_key: key,
            value,
          },
        })
      );
      await account.patchStateRecords({
        records: state,
      });
      console.log(`  ✅ Loaded contract state cache of [${contractAccount}]`);
    }
    return account;
  }
}
