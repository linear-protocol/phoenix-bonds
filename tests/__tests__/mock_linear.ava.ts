import { NEAR } from "near-workspaces";
import { init } from "./init";

const test = init();

test("Stake at LiNEAR price 1", async (test) => {
  const { alice, linear } = test.context.accounts;
  const stakeAmount = NEAR.parse("1000");

  const shares: string = await alice.call(
    linear,
    "deposit_and_stake_v2",
    {},
    {
      attachedDeposit: stakeAmount,
    }
  );

  test.is(shares, stakeAmount.toString());
});

test("Stake at LiNEAR price 1.2", async (test) => {
  const { alice, linear } = test.context.accounts;
  const stakeAmount = NEAR.parse("1000");

  await linear.call(linear, "set_ft_price", {
    price: NEAR.parse("1.2"),
  });

  test.is(await linear.view("ft_price"), NEAR.parse("1.2").toString());

  const shares: string = await alice.call(
    linear,
    "deposit_and_stake_v2",
    {},
    {
      attachedDeposit: stakeAmount,
    }
  );

  test.is(shares, "833333333333333333333333333");
});

test("Transfer LiNEAR", async (test) => {
  const { alice, bob, linear } = test.context.accounts;
  const stakeAmount = NEAR.parse("1000");

  await alice.call(
    linear,
    "deposit_and_stake_v2",
    {},
    {
      attachedDeposit: stakeAmount,
    }
  );

  // to make bob registered
  await bob.call(
    linear,
    "deposit_and_stake_v2",
    {},
    {
      attachedDeposit: stakeAmount,
    }
  );

  await alice.call(
    linear,
    "ft_transfer",
    {
      receiver_id: bob.accountId,
      amount: NEAR.parse("10").toString(),
    },
    {
      attachedDeposit: NEAR.from("1"),
    }
  );
});
