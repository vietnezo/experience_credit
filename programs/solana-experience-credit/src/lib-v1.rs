use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use mpl_token_metadata::instruction::create_metadata_accounts_v2;

declare_id!("1385JT6hVus5EWzut6BtQWLfEeWo2qWgT9FzUTBqJcSK");

#[program]
pub mod solana_experience_credit {
    use super::*;

    pub fn add_review(
        ctx: Context<AddReview>,
        title: String,
        description: String,
        rating: u8,
    ) -> Result<()> {
        msg!("Review Account Created");
        msg!("Title: {}", title);
        msg!("Description: {}", description);
        msg!("Rating: {}", rating);

        if rating > 5 || rating < 1 {
            msg!("Rating cannot be higher than 5");
            return err!(ErrorCode::InvalidRating);
        }

        let review = &mut ctx.accounts.review;
        review.reviewer = ctx.accounts.initializer.key();
        review.title = title;
        review.rating = rating;
        review.description = description;

        msg!("Comment Counter Account Created");
        let comment_counter = &mut ctx.accounts.comment_counter;
        comment_counter.counter = 0;
        msg!("Counter: {}", comment_counter.counter);

        let seeds = &["mint".as_bytes(), &[*ctx.bumps.get("reward_mint").unwrap()]];

        let signer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.reward_mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.reward_mint.to_account_info(),
            },
            &signer,
        );

        token::mint_to(cpi_ctx, 10000000)?;
        msg!("Minted Tokens");
        Ok(())
    }

    pub fn update_review(
        ctx: Context<UpdateReview>,
        title: String,
        description: String,
        rating: u8,
    ) -> Result<()> {
        msg!("Updating  Review Account");
        msg!("Title: {}", title);
        msg!("Description: {}", description);
        msg!("Rating: {}", rating);

        if rating > 5 || rating < 1 {
            msg!("Rating cannot be higher than 5");
            return err!(ErrorCode::InvalidRating);
        }

        let review = &mut ctx.accounts.review;
        review.rating = rating;
        review.description = description;

        Ok(())
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        Ok(())
    }

    pub fn add_comment(ctx: Context<AddComment>, comment: String) -> Result<()> {
        msg!("Comment Account Created");
        msg!("Comment: {}", comment);

        let comment = &mut ctx.accounts.comment;
        let comment_counter = &mut ctx.accounts.comment_counter;

        comment.review = ctx.accounts.review.key();
        comment.commenter = ctx.accounts.initializer.key();
        comment.comment = comment;
        comment.count = comment_counter.counter;

        comment_counter.counter += 1;

        let seeds = &["mint".as_bytes(), &[*ctx.bumps.get("reward_mint").unwrap()]];

        let signer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.reward_mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.reward_mint.to_account_info(),
            },
            &signer,
        );

        token::mint_to(cpi_ctx, 5000000)?;
        msg!("Minted Tokens");

        Ok(())
    }

    pub fn create_reward_mint(
        ctx: Context<CreateTokenReward>,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        msg!("Create Reward Token");

        let seeds = &["mint".as_bytes(), &[*ctx.bumps.get("reward_mint").unwrap()]];

        let signer = [&seeds[..]];

        let account_info = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.reward_mint.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ];

        invoke_signed(
            &create_metadata_accounts_v2(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.reward_mint.key(),
                ctx.accounts.reward_mint.key(),
                ctx.accounts.user.key(),
                ctx.accounts.user.key(),
                name,
                symbol,
                uri,
                None,
                0,
                true,
                true,
                None,
                None,
            ),
            account_info.as_slice(),
            &signer,
        )?;

        Ok(())
    }
}



#[derive(Accounts)]
#[instruction(title:String, description:String)]
pub struct AddReview<'info> {
    #[account(
        init,
        seeds = [title.as_bytes(), initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + 32 + 1 + 4 + title.len() + 4 + description.len()
    )]
    pub review: Account<'info, AccountState>,
    #[account(
        init,
        seeds = ["counter".as_bytes(), review.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + 8
    )]
    pub comment_counter: Account<'info, CommentCounter>,
    #[account(mut,
        seeds = ["mint".as_bytes().as_ref()],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = reward_mint,
        associated_token::authority = initializer
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(title:String, description:String)]
pub struct UpdateReview<'info> {
    #[account(
        mut,
        seeds = [title.as_bytes(), initializer.key().as_ref()],
        bump,
        realloc = 8 + 32 + 1 + 4 + title.len() + 4 + description.len(),
        realloc::payer = initializer,
        realloc::zero = false,
    )]
    pub _review: Account<'info, AccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut, close = reviewer, has_one = reviewer)]
    _review: Account<'info, AccountState>,
    #[account(mut)]
    reviewer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(comment:String)]
pub struct AddComment<'info> {
    #[account(
        init,
        seeds = [review.key().as_ref(), &comment_counter.counter.to_le_bytes()],
        bump,
        payer = initializer,
        space = 8 + 32 + 32 + 4 + comment.len() + 8
    )]
    pub _comment: Account<'info, Comment>,
    pub _review: Account<'info, AccountState>,
    #[account(
        mut,
        seeds = ["counter".as_bytes(), review.key().as_ref()],
        bump,
    )]
    pub comment_counter: Account<'info, CommentCounter>,
    #[account(mut,
        seeds = ["mint".as_bytes().as_ref()],
        bump
    )]
    pub reward_mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = reward_mint,
        associated_token::authority = initializer
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTokenReward<'info> {
    #[account(
        init,
        seeds = ["mint".as_bytes().as_ref()],
        bump,
        payer = user,
        mint::decimals = 6,
        mint::authority = reward_mint,
    )]
    pub reward_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,

    /// CHECK:
    #[account(mut)]
    pub metadata: AccountInfo<'info>,
    /// CHECK:
    pub token_metadata_program: AccountInfo<'info>,
}



#[account]
pub struct AccountState {
    pub reviewer: Pubkey,    // 32
    pub rating: u8,          // 1
    pub title: String,       // 4 + len()
    pub description: String, // 4 + len()
}

#[account]
pub struct CommentCounter {
    pub counter: u64,
}

#[account]
pub struct Comment {
    pub review: Pubkey,    // 32
    pub commenter: Pubkey, // 32
    pub comment: String,   // 4 + len()
    pub count: u64,        // 8
}

#[error_code]
pub enum ErrorCode {
    #[msg("Rating greater than 5 or less than 1")]
    InvalidRating,
}
