use anchor_lang::prelude::*;

declare_id!("AyPvtsSXi8iszQtzhaQFL174UQi5phbBjAe56APLeW4T");

#[program]
pub mod rebate_arbitrage {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fee_rate: u64,
        token_a_mint: Pubkey,
        token_b_mint: Pubkey,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.fee_rate = fee_rate;
        pool.liquidity = 0;
        pool.staked_liquidity = 0;
        pool.token_a_mint = token_a_mint;
        pool.token_b_mint = token_b_mint;
        Ok(())
    }

    pub fn initialize_user(ctx: Context<InitializeUser>, initial_balance: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.token_a_balance = initial_balance;
        user.token_b_balance = 0;
        user.rebates_earned = 0;
        user.last_trade_timestamp = 0;
        user.trade_volume = 0;
        Ok(())
    }

    pub fn provide_liquidity(ctx: Context<ProvideLiquidity>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user = &mut ctx.accounts.user;

        require!(user.token_a_balance >= amount, ErrorCode::InsufficientFunds);
        user.token_a_balance -= amount;
        pool.liquidity += amount;

        // Emit liquidity event
        emit!(LiquidityEvent {
            user_pubkey: user.key(),
            amount,
            liquidity: pool.liquidity,
        });

        Ok(())
    }

    pub fn stake_liquidity_with_lock(ctx: Context<StakeLiquidityWithLock>, amount: u64, lock_duration: i64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user = &mut ctx.accounts.user;
        let staked_liquidity = &mut ctx.accounts.staked_liquidity;

        require!(user.token_a_balance >= amount, ErrorCode::InsufficientFunds);
        user.token_a_balance -= amount;
        pool.staked_liquidity += amount;

        // Lock liquidity for specified duration
        let current_time = Clock::get()?.unix_timestamp;
        staked_liquidity.amount = amount;
        staked_liquidity.lock_expiration = current_time + lock_duration;

        // Reward based on lock duration
        let reward = amount * lock_duration as u64 / (30 * 24 * 60 * 60); // Reward proportional to lock time in days
        user.rebates_earned += reward;

        // Emit staking event
        emit!(StakingEvent {
            user_pubkey: user.key(),
            amount,
            lock_expiration: staked_liquidity.lock_expiration,
            staked_liquidity: pool.staked_liquidity,
        });

        Ok(())
    }

    pub fn trade_with_slippage(ctx: Context<Trade>, trade_amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user = &mut ctx.accounts.user;
        let current_time = Clock::get()?.unix_timestamp;

        // Enforce cooldown period between trades
        let cooldown_period = 60; // 60 seconds
        require!(
            current_time >= user.last_trade_timestamp + cooldown_period,
            ErrorCode::CooldownNotElapsed
        );

        // Calculate slippage based on trade size relative to pool liquidity
        let slippage_factor = (trade_amount as f64 / pool.liquidity as f64) * 100.0; // Example slippage calculation
        let effective_trade_amount = trade_amount as f64 * (1.0 - slippage_factor / 100.0);
        let fee = effective_trade_amount as u64 * pool.fee_rate / 10000;
        let rebate = fee / 2;

        // Update user balances and pool liquidity
        require!(user.token_a_balance >= trade_amount, ErrorCode::InsufficientFunds);
        user.token_a_balance -= trade_amount;
        user.token_b_balance += (effective_trade_amount as u64) - fee;
        pool.liquidity += fee;
        user.rebates_earned += rebate;
        user.trade_volume += trade_amount;
        user.last_trade_timestamp = current_time;

        // Reward users based on cumulative trade volume
        let reward_threshold = 10_000; // Example threshold
        if user.trade_volume >= reward_threshold {
            user.rebates_earned += user.trade_volume / 100; // 1% rebate
            user.trade_volume = 0; // Reset trade volume after distributing reward
        }

        // Emit trade event
        emit!(TradeEvent {
            user_pubkey: user.key(),
            trade_amount,
            fee,
            rebate,
            token_a_balance: user.token_a_balance,
            token_b_balance: user.token_b_balance,
        });

        // Record trade in trade history
        let trade_history = &mut ctx.accounts.trade_history;
        trade_history.timestamp = current_time;
        trade_history.trade_amount = trade_amount;
        trade_history.fee = fee;
        trade_history.rebate = rebate;
        trade_history.user_pubkey = user.key();

        Ok(())
    }

    pub fn unstake_liquidity(ctx: Context<UnstakeLiquidity>, amount: u64) -> Result<()> {
        let staked_liquidity = &mut ctx.accounts.staked_liquidity;
        let user = &mut ctx.accounts.user;
        let pool = &mut ctx.accounts.pool;
        let current_time = Clock::get()?.unix_timestamp;

        require!(staked_liquidity.amount >= amount, ErrorCode::InsufficientFunds);

        // Apply penalty if unstaking before lock expiration
        if current_time < staked_liquidity.lock_expiration {
            let penalty = amount / 10; // 10% penalty
            user.token_a_balance += amount - penalty;
            pool.staked_liquidity -= amount;
        } else {
            user.token_a_balance += amount;
            pool.staked_liquidity -= amount;
        }

        // Emit unstaking event
        emit!(UnstakingEvent {
            user_pubkey: user.key(),
            amount,
            staked_liquidity: pool.staked_liquidity,
        });

        Ok(())
    }

    pub fn adjust_fee_rate(ctx: Context<AdjustFeeRate>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        if pool.liquidity > 0 {
            let utilization_ratio = pool.staked_liquidity as f64 / pool.liquidity as f64;
            if utilization_ratio > 0.8 {
                pool.fee_rate = 50; // Lower fee rate when utilization is high
            } else {
                pool.fee_rate = 100; // Higher fee rate when utilization is low
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
    pub fee_rate: u64,
    pub liquidity: u64,
    pub staked_liquidity: u64,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
}

#[account]
pub struct User {
    pub rebates_earned: u64,
    pub token_a_balance: u64,
    pub token_b_balance: u64,
    pub last_trade_timestamp: i64,
    pub trade_volume: u64,
}

#[account]
pub struct StakedLiquidity {
    pub amount: u64,
    pub lock_expiration: i64,
}

#[account]
pub struct TradeHistory {
    pub timestamp: i64,
    pub trade_amount: u64,
    pub fee: u64,
    pub rebate: u64,
    pub user_pubkey: Pubkey,
}

// Event definitions
#[event]
pub struct TradeEvent {
    pub user_pubkey: Pubkey,
    pub trade_amount: u64,
    pub fee: u64,
    pub rebate: u64,
    pub token_a_balance: u64,
    pub token_b_balance: u64,
}

#[event]
pub struct LiquidityEvent {
    pub user_pubkey: Pubkey,
    pub amount: u64,
    pub liquidity: u64,
}

#[event]
pub struct StakingEvent {
    pub user_pubkey: Pubkey,
    pub amount: u64,
    pub lock_expiration: i64,
    pub staked_liquidity: u64,
}

#[event]
pub struct UnstakingEvent {
    pub user_pubkey: Pubkey,
    pub amount: u64,
    pub staked_liquidity: u64,
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds for the requested operation.")]
    InsufficientFunds,
    #[msg("Cooldown period has not elapsed.")]
    CooldownNotElapsed,
}