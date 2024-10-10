use anchor_lang::prelude::*;

declare_id!("AyPvtsSXi8iszQtzhaQFL174UQi5phbBjAe56APLeW4T");

#[program]
pub mod rebate_arbitrage {
    use super::*;

    /// Initializes a new liquidity pool with the specified fee rate and token mints.
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fee_rate: u64,
        token_a_mint: Pubkey,
        token_b_mint: Pubkey,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.fee_rate = fee_rate; // Set the fee rate for trades (in basis points).
        pool.liquidity = 0; // Initialize liquidity to zero.
        pool.staked_liquidity = 0; // Initialize staked liquidity to zero.
        pool.token_a_mint = token_a_mint; // Set the mint address for Token A.
        pool.token_b_mint = token_b_mint; // Set the mint address for Token B.
        Ok(())
    }

    /// Initializes a new user account with an initial Token A balance.
    pub fn initialize_user(ctx: Context<InitializeUser>, initial_balance: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.token_a_balance = initial_balance; // Set the user's initial Token A balance.
        user.token_b_balance = 0; // Initialize Token B balance to zero.
        user.rebates_earned = 0; // Initialize rebates earned to zero.
        user.last_trade_timestamp = 0; // Initialize the last trade timestamp to zero.
        user.trade_volume = 0; // Initialize the cumulative trade volume to zero.
        Ok(())
    }

    /// Allows a user to add liquidity to the pool.
    pub fn provide_liquidity(ctx: Context<ProvideLiquidity>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user = &mut ctx.accounts.user;

        // Check if the user has enough Token A balance to provide liquidity.
        require!(user.token_a_balance >= amount, ErrorCode::InsufficientFunds);
        
        // Update user and pool balances.
        user.token_a_balance -= amount; // Deduct the provided amount from the user's balance.
        pool.liquidity += amount; // Add the provided amount to the pool's liquidity.

        // Emit an event for adding liquidity.
        emit!(LiquidityEvent {
            user_pubkey: user.key(),
            amount,
            liquidity: pool.liquidity,
        });

        Ok(())
    }

    /// Stakes liquidity in the pool for a specified lock duration.
    /// The user earns rewards based on the lock duration.
    pub fn stake_liquidity_with_lock(ctx: Context<StakeLiquidityWithLock>, amount: u64, lock_duration: i64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user = &mut ctx.accounts.user;
        let staked_liquidity = &mut ctx.accounts.staked_liquidity;

        // Ensure the user has enough Token A to stake.
        require!(user.token_a_balance >= amount, ErrorCode::InsufficientFunds);
        
        // Update balances.
        user.token_a_balance -= amount; // Deduct staked amount from the user's balance.
        pool.staked_liquidity += amount; // Add to the pool's staked liquidity.

        // Set the lock expiration time for the staked liquidity.
        let current_time = Clock::get()?.unix_timestamp;
        staked_liquidity.amount = amount;
        staked_liquidity.lock_expiration = current_time + lock_duration;

        // Calculate rewards based on the lock duration.
        let reward = amount * lock_duration as u64 / (30 * 24 * 60 * 60); // Reward proportional to lock time in days.
        user.rebates_earned += reward; // Update user's total rebates earned.

        // Emit an event for staking.
        emit!(StakingEvent {
            user_pubkey: user.key(),
            amount,
            lock_expiration: staked_liquidity.lock_expiration,
            staked_liquidity: pool.staked_liquidity,
        });

        Ok(())
    }

    /// Executes a trade considering slippage based on the pool's liquidity.
    /// Updates user balances and potentially gives rebates.
    pub fn trade_with_slippage(ctx: Context<Trade>, trade_amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user = &mut ctx.accounts.user;
        let current_time = Clock::get()?.unix_timestamp;

        // Ensure the user waits for the cooldown period before trading again.
        let cooldown_period = 60; // 60 seconds.
        require!(
            current_time >= user.last_trade_timestamp + cooldown_period,
            ErrorCode::CooldownNotElapsed
        );

        // Calculate the slippage and fee for the trade.
        let slippage_factor = (trade_amount as f64 / pool.liquidity as f64) * 100.0; // Example slippage calculation.
        let effective_trade_amount = trade_amount as f64 * (1.0 - slippage_factor / 100.0); // Adjusted for slippage.
        let fee = effective_trade_amount as u64 * pool.fee_rate / 10000; // Fee calculation based on the fee rate.
        let rebate = fee / 2; // Assume half of the fee is given as a rebate.

        // Update user balances and pool liquidity.
        require!(user.token_a_balance >= trade_amount, ErrorCode::InsufficientFunds);
        user.token_a_balance -= trade_amount; // Deduct trade amount from user's Token A balance.
        user.token_b_balance += (effective_trade_amount as u64) - fee; // Increase Token B balance, subtracting fee.
        pool.liquidity += fee; // Add the fee to the pool's liquidity.
        user.rebates_earned += rebate; // Update rebates earned by the user.
        user.trade_volume += trade_amount; // Update cumulative trade volume.
        user.last_trade_timestamp = current_time; // Update the timestamp for the last trade.

        // Provide additional rewards if the user's trade volume crosses a certain threshold.
        let reward_threshold = 10_000; // Example threshold.
        if user.trade_volume >= reward_threshold {
            user.rebates_earned += user.trade_volume / 100; // 1% rebate.
            user.trade_volume = 0; // Reset trade volume after distributing reward.
        }

        // Emit an event for the trade.
        emit!(TradeEvent {
            user_pubkey: user.key(),
            trade_amount,
            fee,
            rebate,
            token_a_balance: user.token_a_balance,
            token_b_balance: user.token_b_balance,
        });

        // Record the trade details in the trade history.
        let trade_history = &mut ctx.accounts.trade_history;
        trade_history.timestamp = current_time;
        trade_history.trade_amount = trade_amount;
        trade_history.fee = fee;
        trade_history.rebate = rebate;
        trade_history.user_pubkey = user.key();

        Ok(())
    }

    /// Unstakes the specified amount of liquidity from the pool.
    /// A penalty may be applied if the lock expiration hasn't passed.
    pub fn unstake_liquidity(ctx: Context<UnstakeLiquidity>, amount: u64) -> Result<()> {
        let staked_liquidity = &mut ctx.accounts.staked_liquidity;
        let user = &mut ctx.accounts.user;
        let pool = &mut ctx.accounts.pool;
        let current_time = Clock::get()?.unix_timestamp;

        // Check if the user has enough staked liquidity.
        require!(staked_liquidity.amount >= amount, ErrorCode::InsufficientFunds);

        // Determine if a penalty should be applied for early unstaking.
        if current_time < staked_liquidity.lock_expiration {
            let penalty = amount / 10; // 10% penalty for early unstaking.
            user.token_a_balance += amount - penalty; // Add the remaining amount after penalty.
            pool.staked_liquidity -= amount; // Decrease the pool's staked liquidity.
        } else {
            user.token_a_balance += amount; // Add full amount to user's balance.
            pool.staked_liquidity -= amount; // Decrease the pool's staked liquidity.
        }

        // Emit an event for unstaking.
        emit!(UnstakingEvent {
            user_pubkey: user.key(),
            amount,
            staked_liquidity: pool.staked_liquidity,
        });

        Ok(())
    }

    /// Adjusts the pool's fee rate based on the ratio of staked liquidity to total liquidity.
    pub fn adjust_fee_rate(ctx: Context<AdjustFeeRate>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        if pool.liquidity > 0 {
            let utilization_ratio = pool.staked_liquidity as f64 / pool.liquidity as f64;
            if utilization_ratio > 0.8 {
                pool.fee_rate = 50; // Lower fee rate when utilization is high (80%+).
            } else {
                pool.fee_rate = 100; // Higher fee rate when utilization is low.
            }
        }
        Ok(())
    }
}

// Account and data structures

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = user_signer, space = 8 + 64)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user_signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(init, payer = user_signer, space = 8 + 64)]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub user_signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProvideLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Account<'info, User>,
}

#[derive(Accounts)]
pub struct StakeLiquidityWithLock<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(init, payer = user, space = 8 + 32)]
    pub staked_liquidity: Account<'info, StakedLiquidity>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Trade<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(init, payer = user, space = 8 + 64)]
    pub trade_history: Account<'info, TradeHistory>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UnstakeLiquidity<'info> {
    #[account(mut)]
    pub staked_liquidity: Account<'info, StakedLiquidity>,
    #[account(mut)]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
}

#[derive(Accounts)]
pub struct AdjustFeeRate<'info> {
    #[account(mut)]
    pub pool: Account<'info, Pool>,
}

// Data structures

#[account]
pub struct Pool {
    pub fee_rate: u64,           // Fee rate for trading, in basis points.
    pub liquidity: u64,          // Total liquidity in the pool.
    pub staked_liquidity: u64,   // Total staked liquidity.
    pub token_a_mint: Pubkey,    // Token A mint address.
    pub token_b_mint: Pubkey,    // Token B mint address.
}

#[account]
pub struct User {
    pub rebates_earned: u64,     // Total rebates earned by the user.
    pub token_a_balance: u64,    // User's Token A balance.
    pub token_b_balance: u64,    // User's Token B balance.
    pub last_trade_timestamp: i64,// Timestamp of the last trade.
    pub trade_volume: u64,       // Total trade volume by the user.
}

#[account]
pub struct StakedLiquidity {
    pub amount: u64,             // Amount of liquidity staked.
    pub lock_expiration: i64,    // Timestamp for when the lock expires.
}

#[account]
pub struct TradeHistory {
    pub timestamp: i64,          // Timestamp of the trade.
    pub trade_amount: u64,       // Amount traded.
    pub fee: u64,                // Fee charged for the trade.
    pub rebate: u64,             // Rebate awarded for the trade.
    pub user_pubkey: Pubkey,     // Public key of the user who made the trade.
}

// Event definitions

#[event]
pub struct TradeEvent {
    pub user_pubkey: Pubkey,     // Public key of the user.
    pub trade_amount: u64,       // Amount traded.
    pub fee: u64,                // Fee charged for the trade.
    pub rebate: u64,             // Rebate given for the trade.
    pub token_a_balance: u64,    // Updated Token A balance.
    pub token_b_balance: u64,    // Updated Token B balance.
}

#[event]
pub struct LiquidityEvent {
    pub user_pubkey: Pubkey,     // Public key of the user.
    pub amount: u64,             // Amount of liquidity added.
    pub liquidity: u64,          // Updated pool liquidity.
}

#[event]
pub struct StakingEvent {
    pub user_pubkey: Pubkey,     // Public key of the user.
    pub amount: u64,             // Amount of liquidity staked.
    pub lock_expiration: i64,    // Lock expiration time.
    pub staked_liquidity: u64,   // Updated staked liquidity.
}

#[event]
pub struct UnstakingEvent {
    pub user_pubkey: Pubkey,     // Public key of the user.
    pub amount: u64,             // Amount of liquidity unstaked.
    pub staked_liquidity: u64,   // Updated staked liquidity.
}

// Error codes

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds for the requested operation.")]
    InsufficientFunds,
    #[msg("Cooldown period has not elapsed.")]
    CooldownNotElapsed,
}
