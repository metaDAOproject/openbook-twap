use anchor_lang::prelude::*;
use openbook_v2::cpi;
use openbook_v2::state::{Market, EventHeap, BookSide};
use anchor_spl::token::{Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;

declare_id!("EgYfg4KUAbXP4UfTrsauxvs75QFf28b3MVEV8qFUGBRh");

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct OracleConfigParams {
    pub conf_filter: f32,
    pub max_staleness_slots: Option<u32>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<Market>(),
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(
        seeds = [b"Market".as_ref(), market.key().to_bytes().as_ref()],
        bump,
    )]
    /// CHECK:
    pub market_authority: UncheckedAccount<'info>,

    /// Accounts are initialized by client,
    /// anchor discriminator is set first when ix exits,
    #[account(zero)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(zero)]
    pub asks: AccountLoader<'info, BookSide>,
    #[account(zero)]
    pub event_heap: AccountLoader<'info, EventHeap>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = base_mint,
        associated_token::authority = market_authority,
    )]
    pub market_base_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = quote_mint,
        associated_token::authority = market_authority,
    )]
    pub market_quote_vault: Account<'info, TokenAccount>,

    pub base_mint: Box<Account<'info, Mint>>,
    pub quote_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: The oracle can be one of several different account types
    pub oracle_a: Option<UncheckedAccount<'info>>,
    /// CHECK: The oracle can be one of several different account types
    pub oracle_b: Option<UncheckedAccount<'info>>,

    /// CHECK:
    pub collect_fee_admin: UncheckedAccount<'info>,
    /// CHECK:
    pub open_orders_admin: Option<UncheckedAccount<'info>>,
    /// CHECK:
    pub consume_events_admin: Option<UncheckedAccount<'info>>,
    /// CHECK:
    pub close_market_admin: Option<UncheckedAccount<'info>>,
}

#[program]
pub mod openbook_twap {
    use super::*;

    pub fn create_market(
        ctx: Context<CreateMarket>,
        name: String,
        oracle_config: OracleConfigParams,
        quote_lot_size: i64,
        base_lot_size: i64,
        maker_fee: i64,
        taker_fee: i64,
        time_expiry: i64,
    ) -> Result<()> {
        Ok(())
    }

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    // create market
}

#[derive(Accounts)]
pub struct Initialize {}
