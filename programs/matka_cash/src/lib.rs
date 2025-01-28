use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("BZqt9Fj99H91QYo5thySX3XSPYAwgA3RdQLZSV8x3V3D");

#[program]
pub mod matka_cash {
    use super::*;

    pub fn initialize_game(
        ctx: Context<InitializeGame>,
        initial_supply: u64,
        game_settings: GameSettings,
    ) -> Result<()> {
        let game_state = &mut ctx.accounts.game_state;
        game_state.authority = ctx.accounts.authority.key();
        game_state.matka_mint = ctx.accounts.matka_mint.key();
        game_state.settings = game_settings;
        game_state.total_supply = initial_supply;
        game_state.initialized = true;
        Ok(())
    }

    pub fn reward_player(
        ctx: Context<RewardPlayer>,
        score: u64,
        level: u8,
    ) -> Result<()> {
        let game_state = &ctx.accounts.game_state;
        
        // Calculate rewards based on score and level
        let reward_amount = calculate_rewards(score, level, &game_state.settings);
        
        // Transfer MatkaCash tokens to player
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.reward_vault.to_account_info(),
                    to: ctx.accounts.player_token_account.to_account_info(),
                    authority: ctx.accounts.game_state.to_account_info(),
                },
            ),
            reward_amount,
        )?;

        emit!(GameReward {
            player: ctx.accounts.player.key(),
            amount: reward_amount,
            score,
            level,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn create_nft_reward(
        ctx: Context<CreateNFTReward>,
        metadata: ArtifactMetadata,
    ) -> Result<()> {
        let game_state = &ctx.accounts.game_state;
        // NFT creation logic here
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + GameState::SPACE
    )]
    pub game_state: Account<'info, GameState>,
    
    /// The MatkaCash token mint
    pub matka_mint: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RewardPlayer<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    
    /// The vault holding MatkaCash tokens for rewards
    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,
    
    /// The player's MatkaCash token account
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    
    pub player: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateNFTReward<'info> {
    #[account(mut)]
    pub game_state: Account<'info, GameState>,
    pub authority: Signer<'info>,
}

/// Stores the state of the MatkaCash game and token system
#[account]
pub struct GameState {
    pub authority: Pubkey,
    pub matka_mint: Pubkey,
    pub settings: GameSettings,
    pub total_supply: u64,
    pub initialized: bool,
}

impl GameState {
    pub const SPACE: usize = 32 + 32 + 64 + GameSettings::SPACE + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GameSettings {
    pub base_reward_rate: u64,
    pub level_multiplier: u8,
    pub min_score_threshold: u64,
    pub max_daily_rewards: u64,
}

impl GameSettings {
    pub const SPACE: usize = 8 + 1 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ArtifactMetadata {
    pub name: String,
    pub artifact_type: String,
    pub rarity: u8,
    pub power_level: u8,
}

#[event]
pub struct GameReward {
    pub player: Pubkey,
    pub amount: u64,
    pub score: u64,
    pub level: u8,
    pub timestamp: i64,
}

fn calculate_rewards(score: u64, level: u8, settings: &GameSettings) -> u64 {
    if score < settings.min_score_threshold {
        return 0;
    }
    
    let base_reward = score.saturating_mul(settings.base_reward_rate);
    let level_bonus = (level as u64).saturating_mul(settings.level_multiplier as u64);
    
    base_reward.saturating_add(level_bonus)
}
