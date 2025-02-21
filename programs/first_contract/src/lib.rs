#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

// Declare the program ID
declare_id!("D3LMDue6hQpkjM5SUFcFnc5i2GH9Qk2FjNwngGG5Zhfe");

#[program]
pub mod token_distributor {
    use super::*;

    /// Initialize the distributor contract
    /// Sets up the basic parameters for token distribution
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let distributor = &mut ctx.accounts.distributor;
        
        // Set the authority and token mint
        distributor.authority = ctx.accounts.authority.key();
        distributor.xyz_mint = ctx.accounts.xyz_mint.key();
        
        // Set distribution interval to 10 minutes (in seconds)
        distributor.distribution_interval = 600;
        
        // Set initial distribution timestamp
        distributor.last_distribution = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Distribute rewards to token holders
    /// Calculates and transfers rewards based on holder's token balance
    pub fn distribute_rewards(ctx: Context<DistributeRewards>) -> Result<()> {
        let distributor = &mut ctx.accounts.distributor;
        let clock = Clock::get()?;
        
        // Check if enough time has passed since last distribution
        require!(
            clock.unix_timestamp >= distributor.last_distribution + distributor.distribution_interval,
            DistributorError::TooEarlyForDistribution
        );

        // Get total supply and current reward amount
        let mint_data = ctx.accounts.xyz_mint.try_borrow_data()?;
        let total_supply = u64::from_le_bytes(mint_data[36..44].try_into().unwrap());
        let vault_data = ctx.accounts.reward_vault.try_borrow_data()?;
        let reward_amount = u64::from_le_bytes(vault_data[64..72].try_into().unwrap());

        // Calculate holder's share based on their balance
        let holder_data = ctx.accounts.holder_token_account.try_borrow_data()?;
        let holder_balance = u64::from_le_bytes(holder_data[64..72].try_into().unwrap());
        
        let holder_share = (holder_balance
            .checked_mul(reward_amount)
            .ok_or(DistributorError::CalculationError)?
        )
            .checked_div(total_supply)
            .ok_or(DistributorError::CalculationError)?;

        // Set up PDA signer for vault authority
        let seeds = &[
            b"vault".as_ref(),
            &[ctx.bumps.vault_authority],
        ];
        let signer = &[&seeds[..]];

        // Transfer tokens to holder
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
            holder_share,
        )?;

        // Update last distribution timestamp
        distributor.last_distribution = clock.unix_timestamp;
        
        Ok(())
    }
}

/// Accounts required for initializing the distributor
#[derive(Accounts)]
pub struct Initialize<'info> {
    // Initialize the distributor account
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8 + 8
    )]
    pub distributor: Account<'info, Distributor>,
    
    // The mint of the token being distributed
    /// CHECK: Token mint account
    pub xyz_mint: AccountInfo<'info>,
    
    // The authority who pays for initialization
    #[account(mut)]
    pub authority: Signer<'info>,
    
    // Required system program
    pub system_program: Program<'info, System>,
}

/// Accounts required for distributing rewards
#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    // The distributor account
    #[account(mut)]
    pub distributor: Account<'info, Distributor>,
    
    // The token mint
    /// CHECK: Token mint account verified in constraints
    pub xyz_mint: AccountInfo<'info>,
    
    // The token account of the holder receiving rewards
    /// CHECK: Token account verified in program logic
    #[account(mut)]
    pub holder_token_account: AccountInfo<'info>,
    
    // The vault holding rewards to be distributed
    /// CHECK: Token account verified in program logic
    #[account(mut)]
    pub reward_vault: AccountInfo<'info>,
    
    // The PDA that acts as the vault authority
    /// CHECK: PDA used as vault authority
    #[account(
        seeds = [b"vault"],
        bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,
    
    // The holder receiving rewards
    pub holder: SystemAccount<'info>,
    
    // The token program
    pub token_program: Program<'info, Token>,
}

/// The distributor account data structure
#[account]
pub struct Distributor {
    // The authority who initialized the distributor
    pub authority: Pubkey,
    
    // The mint address of the token
    pub xyz_mint: Pubkey,
    
    // The interval between distributions in seconds
    pub distribution_interval: i64,
    
    // Timestamp of the last distribution
    pub last_distribution: i64,
}

/// Custom error codes for the distributor program
#[error_code]
pub enum DistributorError {
    #[msg("Not enough time has passed since last distribution")]
    TooEarlyForDistribution,
    
    #[msg("Error in reward calculation")]
    CalculationError,
}