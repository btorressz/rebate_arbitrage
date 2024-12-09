# rebate_arbitrage
# Rebate Arbitrage Program

The Rebate Arbitrage program is an Anchor-based program deployed on the Solana blockchain. It allows users to participate in a liquidity pool through a variety of operations such as providing liquidity, staking assets with lock periods, and trading while accounting for slippage. Users can earn rebates and rewards through these activities, and the program can dynamically adjust fee rates based on the utilization of the pool's liquidity.

The program was developed using the Anchor framework, which simplifies Solana smart contract development by providing a Rust-based programming model. This program was created in the Solana Playground IDE, a web-based development environment designed for quick prototyping and testing on the Solana network.

devnet:(https://explorer.solana.com/address/AyPvtsSXi8iszQtzhaQFL174UQi5phbBjAe56APLeW4T?cluster=devnet)

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

### User

The `User` account represents a participant in the rebate arbitrage program.

- `rebates_earned` (u64): Total rebates earned by the user.
- `token_a_balance` (u64): The balance of Token A held by the user.
- `token_b_balance` (u64): The balance of Token B held by the user.
- `last_trade_timestamp` (i64): The timestamp of the last trade executed by the user.
- `trade_volume` (u64): The total trade volume for rebate calculations.

### StakedLiquidity

The `StakedLiquidity` account tracks the amount of liquidity staked and the lock expiration time.

- `amount` (u64): The amount of staked liquidity.
- `lock_expiration` (i64): The timestamp when the lock expires.

### TradeHistory

The `TradeHistory` account records the details of a trade.

- `timestamp` (i64): The timestamp of the trade.
- `trade_amount` (u64): The amount traded.
- `fee` (u64): The fee charged for the trade.
- `rebate` (u64): The rebate awarded.
  ## Functions

### `initialize_pool`

Initializes a new liquidity pool with the given fee rate and token mint addresses.

**Parameters:**
- `fee_rate` (u64): Fee rate for trades in basis points.
- `token_a_mint` (Pubkey): Mint address for Token A.
- `token_b_mint` (Pubkey): Mint address for Token B.

### `initialize_user`

Creates a new user account with an initial balance of Token A.

**Parameters:**
- `initial_balance` (u64): Initial balance of Token A for the user.

### `provide_liquidity`

Allows a user to provide liquidity to the pool. The user's Token A balance is reduced by the provided amount.

**Parameters:**
- `amount` (u64): Amount of liquidity to provide.

### `stake_liquidity_with_lock`

Stakes liquidity in the pool for a fixed lock period. Users can earn rewards based on the lock duration.

**Parameters:**
- `amount` (u64): Amount of liquidity to stake.
- `lock_duration` (i64): Lock duration in seconds.

### `trade_with_slippage`

Allows trading with a specified trade amount while considering slippage based on pool liquidity. It enforces a cooldown period between trades.

**Parameters:**
- `trade_amount` (u64): Amount to trade.

### `unstake_liquidity`

Unstakes liquidity from the pool. If the lock period has not expired, a penalty is applied.

**Parameters:**
- `amount` (u64): Amount of liquidity to unstake.

### `adjust_fee_rate`

Dynamically adjusts the fee rate based on the ratio of staked liquidity to total liquidity.

## Events

The program emits the following events:

### `TradeEvent`
- Emitted when a trade is executed.
- Includes the user's public key, trade amount, fee, rebate, and updated token balances.

### `LiquidityEvent`
- Emitted when liquidity is added to the pool.
- Includes the user's public key, amount added, and updated pool liquidity.

### `StakingEvent`
- Emitted when liquidity is staked with a lock.
- Includes the user's public key, amount staked, lock expiration, and updated staked liquidity.

### `UnstakingEvent`
- Emitted when liquidity is unstaked.
- Includes the user's public key, amount unstaked, and updated staked liquidity.

## Error Codes

- `InsufficientFunds`: Raised when the user does not have enough funds to perform an action.
- `CooldownNotElapsed`: Raised when a user tries to trade before the cooldown period has elapsed.

## Testing

The `client.ts` and `anchor.test.ts` files are mainly for internal testing purposes mainly for me to use for. They are used to:

- Interact with the `rebate_arbitrage` program.
- Perform actions such as initializing pools, users, providing liquidity, staking, and trading.
- Verify the correct behavior of the program and handle edge cases.
- You're welcome to read over them.

These scripts include logic to request airdrops for testing accounts to ensure they have enough SOL for transactions and rent exemption.
- `user_pubkey` (Pubkey): The public key of the user who made the trade.

  ## LICENSE
  This project is under the **MIT LICENSE**
