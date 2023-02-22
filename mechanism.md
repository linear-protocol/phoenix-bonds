# Phoenix Bonds Math

## Tokens
- `NEAR`
- `LiNEAR`
- `pNEAR`: Boosted NEAR in Phoenix Bonds

## Symbols
 - ### Variables
     - $N_p$: total staked NEAR amount in pending pool
     - $N_d$: total NEAR amount in permanent pool
     - $N_r$: total NEAR amount in reserve pool (this is not stored but calculated)
     - $N_t$: total NEAR amount in treasury pool
     - $L$: total LiNEAR the protocol holds
     - $P_l$: LiNEAR/NEAR price
     - $P_r$: pNEAR/NEAR redeem price
     - $S$: pNEAR total supply
    
 - ### Parameters
     - $\alpha$: pNEAR accrue parameter
     - $\tau$: max percentage of bond amount that goes to permanent pool when a user commits, e.g. 3%


## Scenarios

### I. Bond
User bonds $n$ NEAR:
- create a new bond note with bonding amount set to $n$
- $N_p = N_p + n$
- Stake all $n$ NEAR via LiNEAR, get $l$ LiNEAR in return
- $L = L + l$

### II. Cancel
User cancels a bond with amount $n$, we return LiNEAR instead of NEAR to him
- the amount of LiNEAR the user could get back: $m = n / P_l$
- decrease pending pool: $N_p = N_p - n$
- decrease linear balance: $L = L - m$
- Transfer $m$ LiNEAR to user

### III. Commit
User commits a bond note with amount $n$ NEAR to pNEAR, we mint pNEAR to him.

- Amount of NEAR that goes to treasury (1st priority)
    - If it's first commit: $N_t' = n' + (L * P_l - N_p)$
    - otherwise: $N_t' = n'$
    - where $n' = n * \tau$
- Amount of NEAR that *should* go to reserve pool (2nd priority)
    - $N_r' = (n - n') * \frac{t}{t + \alpha}$
- pNEAR to mint
    - $A = N_r' / P_r$
        - if it's the first time a user commits, $P_r = 1$
        - else $P_r = N_r / S = (L * P_l - N_p - N_t - N_d) / S$
    - this keeps the pNEAR price consistent
- Amount of NEAR that goes to permanent pool
    - $N_d' = n - n' - N_r'$
- increase treasury: $N_t = N_t + N_t'$
- increase permanent pool: $N_d = N_d + N_d'$
- decrease pending pool: $N_p = N_p - n$
- mint pNEAR: $S = S + A$

### IV. Redeem
User redeems $p$ pNEAR, we return LiNEAR instead of NEAR to him.
- Amount of LiNEAR that needs to be redeemed: $r = E_p / P_l$, where
    - $E_p$ is the equivalent amount of NEAR that $p$ pNEAR worth
    - $E_p = p * P_r = p * N_r / S$
    - Remember that $N_r = L * P_l - N_p - N_d - N_t$
- decrease linear balance: $L = L - r$
- burn pNEAR: $S = S - p$
- Transfer $r$ LiNEAR to user


## How to compute volume-weighted average bonding length

### Variables
- $V_i$: bond amount of bond $i$.
- $L_i$: How long has bond $i$ been created.
- $W$: sum of volume weighted bonding length, which should be equal to $V_1 L_1 + V_2 L_2 + ...$, initially set to 0.
- $V$: total volume pending, which should be equal to $V_1 + V_2 + ...$, initially set to 1 to avoid divided by 0.
- $va$: volume-weighted average bonding length, which is $W/V$
- $t_a$：last time when any of these variables is updated, which is set to the first bonding time initially

### I. Update $W$
At any give time $t$
- $va$ should be $W/V + (t - t_a)$
- since the average bonding time will increase just as the time elapsed from last updated time
- thus, we can adjust $W$ to make $W' = va * V$, so that $W'/V$ results in the correct $va$ value.
- and update $t_a = t$

### I. Bond
User bonds $n$ NEAR:
- If it's the first bond, then set $W = 0$ and $t_a = now$, else update $W$ and $t_a$ as mentioned above
- because the newly bonded NEAR has length of 0
- we just need to update $V' = V + n$
- so that $W/V'$ will be the correct value


### II. Commit/Cancel
User commits/cancels a bond of $n$ NEAR with length $l$
- First update $W$ and $t_a$ as mentioned above
- update $W' = W - n*l$
- update $V' = V - n$
