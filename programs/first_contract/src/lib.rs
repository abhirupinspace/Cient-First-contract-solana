#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};

// Program ID for deployment
declare_id!("D3LMDue6hQpkjM5SUFcFnc5i2GH9Qk2FjNwngGG5Zhfe");

/// Token Distribution Program
/// Implements the formula: Ri = (Ti/Ttotal) × X where:
/// - Ri: Rewards for account i
/// - Ti: Number of tokens held by account i
/// - Ttotal: Total tokens held by all eligible accounts (>1K tokens)
/// - X: Total rewards to distribute
#[program]
pub mod token_distributor {
    use super::*;

    /// Initialize the distributor program
    /// Sets up initial state and configuration for token distribution
    /// Parameters required:
    /// - State account to store distribution data
    /// - Token mint for the rewards
    /// - Authority who can manage distributions
    /// - Vault authority PDA for secure token management
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        
        // Set the admin authority who can manage distributions
        state.authority = ctx.accounts.authority.key();
        // Set token mint for reward tracking
        state.token_mint = ctx.accounts.token_mint.key();
        // Set minimum token requirement to 1K tokens
        state.min_token_threshold = 1000; // Only accounts with >1K tokens are eligible
        // Initialize Ttotal (total eligible tokens) to 0
        state.total_eligible_tokens = 0;   
        // Set initial timestamp for distribution tracking
        state.last_distribution = Clock::get()?.unix_timestamp;
        // Set distribution interval to 10 minutes (600 seconds)
        state.distribution_interval = 600; 
        // Initialize distribution state flag
        state.is_distribution_active = false;
        
        Ok(())
    }

    /// Start a new distribution cycle
    /// This must be called before calculating rewards
    /// Checks:
    /// - Enough time has passed since last distribution
    /// - No distribution is currently active
    pub fn start_distribution(ctx: Context<StartDistribution>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let clock = Clock::get()?;

        // Verify distribution interval has passed
        require!(
            clock.unix_timestamp >= state.last_distribution + state.distribution_interval,
            DistributorError::TooEarlyForDistribution
        );

        // Ensure no other distribution is in progress
        require!(
            !state.is_distribution_active,
            DistributorError::DistributionInProgress
        );

        // Reset Ttotal for new calculation
        state.total_eligible_tokens = 0;
        // Mark distribution as active
        state.is_distribution_active = true;

        Ok(())
    }

    /// Calculate total eligible tokens (Ttotal)
    /// Processes a batch of token accounts to sum up total eligible tokens
    /// Only includes accounts with more than 1K tokens
    /// This implements the summation of Ti for all eligible accounts
    pub fn calculate_total_eligible_tokens<'info>(
        ctx: Context<'_, '_, 'info, 'info, CalculateTotal<'info>>,
        _batch_size: u64,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        
        // Verify distribution is active
        require!(
            state.is_distribution_active,
            DistributorError::DistributionNotStarted
        );

        // Calculate Ttotal by summing Ti for all eligible accounts
        for account_info in ctx.remaining_accounts {
            let token_account = Account::<'info, TokenAccount>::try_from(account_info)?;
            
            // Only include Ti if account has more than 1K tokens
            if token_account.amount >= state.min_token_threshold {
                // Add Ti to Ttotal
                state.total_eligible_tokens = state.total_eligible_tokens
                    .checked_add(token_account.amount)
                    .ok_or(DistributorError::CalculationError)?;
            }
        }

        Ok(())
    }

    /// Distribute rewards to a single token holder
    /// Implements the formula: Ri = (Ti/Ttotal) × X
    /// Where:
    /// - Ri = rewards for this account
    /// - Ti = tokens held by this account
    /// - Ttotal = total eligible tokens (calculated previously)
    /// - X = total rewards available to distribute
    pub fn distribute_rewards(ctx: Context<DistributeRewards>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        
        // Verify distribution is active
        require!(
            state.is_distribution_active,
            DistributorError::DistributionNotStarted
        );

        // Get Ti (tokens held by this account)
        let tokens_held_by_account = ctx.accounts.holder_token_account.amount;
        
        // Verify account meets minimum token requirement
        require!(
            tokens_held_by_account >= state.min_token_threshold,
            DistributorError::InsufficientBalance
        );

        // Get X (total rewards available to distribute)
        let total_rewards_to_distribute = ctx.accounts.reward_vault.amount;

        // Calculate Ri using the formula: Ri = (Ti/Ttotal) × X
        // First multiply Ti × X to maintain precision
        let rewards = (tokens_held_by_account  // Ti
            .checked_mul(total_rewards_to_distribute)  // × X
            .ok_or(DistributorError::CalculationError)?
        )
            // Then divide by Ttotal
            .checked_div(state.total_eligible_tokens)  // ÷ Ttotal
            .ok_or(DistributorError::CalculationError)?;

        // Only transfer if account is eligible for rewards
        if rewards > 0 {
            // Set up PDA signer for secure vault access
            let seeds = &[
                b"vault".as_ref(),
                &[ctx.bumps.vault_authority],
            ];
            let signer = &[&seeds[..]];

            // Transfer Ri tokens to holder
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    token::Transfer {
                        from: ctx.accounts.reward_vault.to_account_info(),
                        to: ctx.accounts.holder_token_account.to_account_info(),
                        authority: ctx.accounts.vault_authority.to_account_info(),
                    },
                    signer,
                ),
                rewards,  // Amount = Ri from formula
            )?;
        }

        Ok(())
    }

    /// End the current distribution cycle
    /// Updates timestamps and resets distribution state
    pub fn end_distribution(ctx: Context<EndDistribution>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        
        // Verify distribution is active
        require!(
            state.is_distribution_active,
            DistributorError::DistributionNotStarted
        );

        // Mark distribution as complete
        state.is_distribution_active = false;
        // Update last distribution timestamp
        state.last_distribution = Clock::get()?.unix_timestamp;

        Ok(())
    }
}

/// Initialize instruction accounts
#[derive(Accounts)]
pub struct Initialize<'info> {
    // State account to store distribution data
    #[account(
        init,
        payer = authority,
        space = DistributorState::SIZE
    )]
    pub state: Account<'info, DistributorState>,
    
    // Token mint for the rewards
    pub token_mint: Account<'info, Mint>,
    
    // Authority who manages distributions
    #[account(mut)]
    pub authority: Signer<'info>,

    // PDA that acts as vault authority
    #[account(
        init,
        payer = authority,
        space = 8,
        seeds = [b"vault"],
        bump
    )]
    pub vault_authority: Account<'info, VaultAuthority>,
    
    pub system_program: Program<'info, System>,
}

/// Start distribution instruction accounts
#[derive(Accounts)]
pub struct StartDistribution<'info> {
    // Must be signed by authority
    #[account(mut)]
    pub authority: Signer<'info>,
    
    // State account must be mutable and owned by authority
    #[account(
        mut,
        has_one = authority,
    )]
    pub state: Account<'info, DistributorState>,
}

/// Calculate total supply instruction accounts
#[derive(Accounts)]
pub struct CalculateTotal<'info> {
    // Must be signed by authority
    #[account(mut)]
    pub authority: Signer<'info>,
    
    // State account must be mutable and owned by authority
    #[account(
        mut,
        has_one = authority,
    )]
    pub state: Account<'info, DistributorState>,
}

/// Distribute rewards instruction accounts
#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    // State account storing distribution data
    #[account(mut)]
    pub state: Account<'info, DistributorState>,
    
    // Token mint for verification
    pub token_mint: Account<'info, Mint>,
    
    // Holder's token account receiving rewards
    #[account(
        mut,
        constraint = holder_token_account.mint == token_mint.key()
    )]
    pub holder_token_account: Account<'info, TokenAccount>,
    
    // Vault holding rewards to distribute
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    
    // PDA that controls the reward vault
    /// CHECK: PDA used as vault authority
    #[account(
        seeds = [b"vault"],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    
    // Token program for transfers
    pub token_program: Program<'info, Token>,
}

/// End distribution instruction accounts
#[derive(Accounts)]
pub struct EndDistribution<'info> {
    // Must be signed by authority
    #[account(mut)]
    pub authority: Signer<'info>,
    
    // State account must be mutable and owned by authority
    #[account(
        mut,
        has_one = authority,
    )]
    pub state: Account<'info, DistributorState>,
}

/// State account storing distribution configuration and tracking
#[account]
pub struct DistributorState {
    pub authority: Pubkey,             // Admin who can manage distributions
    pub token_mint: Pubkey,            // Token mint being distributed
    pub min_token_threshold: u64,      // Minimum tokens required (1K)
    pub total_eligible_tokens: u64,    // Ttotal in the formula
    pub distribution_interval: i64,    // Time between distributions
    pub last_distribution: i64,        // Last distribution timestamp
    pub is_distribution_active: bool,  // Distribution state flag
}

/// Empty account that acts as vault authority
#[account]
pub struct VaultAuthority {}

/// Calculate required account sizes
impl DistributorState {
    pub const SIZE: usize = 8 +    // Account discriminator
        32 +   // authority: Pubkey
        32 +   // token_mint: Pubkey
        8 +    // min_token_threshold: u64
        8 +    // total_eligible_tokens: u64
        8 +    // distribution_interval: i64
        8 +    // last_distribution: i64
        1;     // is_distribution_active: bool
}

/// Program error codes
#[error_code]
pub enum DistributorError {
    #[msg("Distribution cannot start yet - interval not elapsed")]
    TooEarlyForDistribution,
    
    #[msg("Error in reward calculation")]
    CalculationError,

    #[msg("Distribution is already in progress")]
    DistributionInProgress,

    #[msg("Distribution has not been started")]
    DistributionNotStarted,

    #[msg("Insufficient token balance - minimum 1K tokens required")]
    InsufficientBalance,
}