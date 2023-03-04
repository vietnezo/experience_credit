use anchor_lang::prelude::*;
use borsh::BorshDeserialize;
use anchor_lang::{
    solana_program::program::invoke_signed
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
    instruction::create_metadata_accounts_v3, pda::find_metadata_account, ID as MetadataID,
};

declare_id!("CfUigSE7puwu2YuTRKsxM1Cs5bFsMG2nvKsTDvp2bKmn");

#[program]
pub mod solana_experience_credit {
    use super::*;

    pub fn add_Review(
        ctx: Context<AddReview>,
        title: String,
        description: String,
    ) -> Result<()> {
        rating: u8,
        
        require!(rating >= 1 && rating <= 5, ReviewError::InvalidRating);
        
        let review = &mut ctx.accounts.review;
        review.reviewer = ctx.accounts.initializer.key();
        review.title = title;
        review.description = description;
        review.rating = rating;

        msg!("Review account created");

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo { 
                    authority: ctx.accounts.mint.to_account_info(), 
                    to: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info()
                },
                &[&[
                    b"mint",
                    &[*ctx.bumps.get("mint").unwrap()],
                ]]
            ),
            1000000    // 1 token
        )?;

        msg!("Minted tokens");
        
        Ok(())
    }

    pub fn update_review(ctx: Context<UpdateReview>, title: String, description: String, rating: u8) -> Result<()> {
        msg!("Review account space reallocated");
        msg!("Title: {}", title);
        msg!("Description: {}", description);
        msg!("Rating: {}", rating);

        require!(rating >= 1 && rating <= 5, ReviewError::InvalidRating);
        
        let review = &mut ctx.accounts.review;
        review.description = description;
        review.rating = rating;
        
        Ok(())
    }

    pub fn delete_review(_ctx: Context<DeleteReview>, title: String) -> Result<()> {
        msg!("Review for {} deleted", title);
        Ok(())
    }

    pub fn initialize_token_mint(
        ctx: Context<InitializeMint>,
        uri: String,
        name: String,
        symbol: String,
        _decimals: u8,
    ) -> Result<()> {
        
        let seeds = &["mint".as_bytes(), &[*ctx.bumps.get("mint").unwrap()]];
        let signer = [&seeds[..]];

        let account_info = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        invoke_signed(
            &create_metadata_accounts_v3(
                ctx.accounts.token_metadata_program.key(),       // token metadata program
                ctx.accounts.metadata.key(),                     // metadata account PDA for mint
                ctx.accounts.mint.key(),                         // mint account
                ctx.accounts.mint.key(),                         // mint authority
                ctx.accounts.user.key(),                         // payer for transaction
                ctx.accounts.mint.key(),                         // update authority
                name,                                            // name
                symbol,                                          // symbol
                uri,                                             // uri (offchain metadata)
                None,                                            // (optional) creators
                0,                                               // seller free basis points
                true,                                            // (bool) update authority is signer
                true,                                            // (bool) is mutable
                None,                                            // (optional) collection
                None,                                            // (optional) uses
                None,                                            // (optional) collection details
            ),
            account_info.as_slice(),
            &signer,
        )?;

        msg!("Token mint initialized");

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(title: String, description: String)]
pub struct AddReview<'info> {
    #[account(
        init, 
        seeds=[title.as_bytes(), initializer.key().as_ref()], 
        bump, 
        payer = initializer, 
        space = 8 + 32 + 1 + 4 + title.len() + 4 + description.len()
    )]
    pub review: Account<'info, AccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(
        seeds=[b"mint"],
        bump,
        mut
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = mint,
        associated_token::authority = initializer
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(title: String, description: String)]
pub struct UpdateReview<'info> {
    #[account(
        mut,
        seeds=[title.as_bytes(), initializer.key().as_ref()],
        bump,
        realloc = 8 + 32 + 1 + 4 + title.len() + 4 + description.len(),
        realloc::payer = initializer,
        realloc::zero = true
    )]
    pub review: Account<'info, AccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct DeleteReview<'info> {
    #[account(
        mut,
        seeds=[title.as_bytes(), initializer.key().as_ref()],
        bump,
        close=initializer
    )]
    pub review: Account<'info, AccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(
    uri: String,
    name: String,
    symbol: String,
    decimals: u8,
)]
pub struct InitializeMint<'info> {
    #[account(
        init,
        seeds = [b"mint"],
        bump,
        payer = user,
        mint::decimals = decimals,
        mint::authority = mint,
    )]
    pub mint: Account<'info, Mint>,
    /// CHECK: Using "address" constraint to validate metadata account address
    #[account(
        mut,
        address = find_metadata_account(&mint.key()).0
    )]
    pub metadata: UncheckedAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, TokenMetaData>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct AccountState {
    pub reviewer: Pubkey,
    pub rating: u8,
    pub title: String,
    pub description: String,
}

#[derive(Clone)]
pub struct TokenMetaData;
impl anchor_lang::Id for TokenMetaData {
    fn id() -> Pubkey {
        MetadataID
    }
}

#[error_code]
enum ReviewError {
    #[msg("Rating must be between 1 and 5")]
    InvalidRating
}
