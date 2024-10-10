# rebate_arbitrage
# Rebate Arbitrage Program

The Rebate Arbitrage program is a smart contract on the Solana blockchain that allows users to interact with a liquidity pool by providing liquidity, staking with lock periods, and trading with slippage while potentially earning rebates. The program also supports adjusting fee rates based on liquidity utilization.

## Program Overview

The main functions of the program include:

1. **Initialize Pool**: Sets up a new liquidity pool with a specified fee rate and token pair.
2. **Initialize User**: Creates a user account with an initial token balance.
3. **Provide Liquidity**: Allows a user to add liquidity to the pool.
4. **Stake Liquidity With Lock**: Enables users to stake liquidity in the pool for a fixed lock period, potentially earning rewards.
5. **Trade With Slippage**: Allows trading with slippage calculations based on the pool's current liquidity.
6. **Unstake Liquidity**: Users can withdraw staked liquidity, with or without penalties based on lock expiration.
7. **Adjust Fee Rate**: Dynamically adjusts the pool's fee rate based on utilization.

## Accounts

### Pool

The `Pool` account represents a liquidity pool, including details such as the fee rate, total liquidity, staked liquidity, and the associated token pair.

- `fee_rate` (u64): The fee rate charged for trades in basis points.
- `liquidity` (u64): Total liquidity in the pool.
- `staked_liquidity` (u64): Total staked liquidity in the pool.
- `token_a_mint` (Pubkey): The mint address of Token A.
- `token_b_mint` (Pubkey): The mint address of Token B.
