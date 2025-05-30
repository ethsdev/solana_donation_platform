use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, rent, system_instruction};

declare_id!("ESAXaQuTVApRKEVhgNPd3EnLgH7fNQhLRSu9pP5CJZ23");

#[program]
pub mod solana_donation_platform {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, owner: Pubkey) -> Result<()> {
        ctx.accounts.base_account.owner = owner;
        Ok(())
    }

    pub fn donation(ctx: Context<Donation>, amount: u64) -> Result<()> {
        require!(amount > 0, DonationError::InvalidAmount);

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.donator.key(),
                &ctx.accounts.donation.key(),
                amount,
            ),
            &[ctx.accounts.donator.to_account_info(), ctx.accounts.donation.to_account_info()],
        ).map_err(Into::<error::Error>::into)?;

        let donation_data = &mut ctx.accounts.donation_data;
        donation_data.donator = ctx.accounts.donator.key();
        donation_data.donation = ctx.accounts.donation.key();
        donation_data.amount = donation_data.amount.saturating_add(amount);

        emit!(DonationEvent {
            donation: ctx.accounts.donation.key(),
            donator: ctx.accounts.donator.key(),
            amount,
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let rent_exempt = rent::Rent::get()?.minimum_balance(BaseAccount::LEN);
        let available = ctx.accounts.donation.to_account_info().lamports()
            .saturating_sub(rent_exempt);

        require!(available > 0, DonationError::Noamount);

        **ctx.accounts.dest.to_account_info().try_borrow_mut_lamports()? += available;
        **ctx.accounts.donation.to_account_info().try_borrow_mut_lamports()? -= available;

        emit!(WithdrawEvent {
            donation: ctx.accounts.donation.key(),
            dest: ctx.accounts.dest.key(),
            amount: available,
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(owner: Pubkey, bump: u8)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = BaseAccount::LEN, seeds = [owner.as_ref()], bump)]
    pub base_account: Account<'info, BaseAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Donation<'info> {
    #[account(mut)]
    pub donation: Account<'info, BaseAccount>,

    #[account(init_if_needed, payer = donator, space = 64 + 1024, seeds = [donator.key().as_ref()], bump)]
    pub donation_data: Account<'info, DonationData>,

    #[account(mut)]
    pub donator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub donation: Account<'info, BaseAccount>,
    pub owner: Signer<'info>,
    #[account(mut)]
    pub dest: Account<'info, BaseAccount>,
    #[account(mut)]
    pub bank: Account<'info, DonationData>,
}

#[account]
pub struct BaseAccount {
    pub owner: Pubkey,
}

impl BaseAccount {
    pub const LEN: usize = 32 + 8;
}

#[account]
pub struct DonationData {
    pub donation: Pubkey,
    pub donator: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum DonationError {
    #[msg("amount of amount should be more than zero!!")]
    InvalidAmount,

    #[msg("The donation bank is empty")]
    Noamount,
}

#[event]
pub struct DonationEvent {
    pub donation: Pubkey,
    pub donator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawEvent {
    pub donation: Pubkey,
    pub dest: Pubkey,
    pub amount: u64,
}
