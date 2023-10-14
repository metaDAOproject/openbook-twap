use anchor_lang::prelude::*;
use openbook_v2::cpi;
use openbook_v2::state::{Market, BookSide};
//use openbook_v2::PlaceOrderArgs;

declare_id!("EgYfg4KUAbXP4UfTrsauxvs75QFf28b3MVEV8qFUGBRh");

// `create_twap_market` that verifies that the `open_orders_authority` and 
// `close_market_admin` are set to `twap_market`, `time_expiry` == 0, store
// `base_lot_size` and `quote_lot_size`
//
// wrappers:
// - place_order
// - edit_order
// - edit_order_pegged
// - cancel_and_place_orders
// - place_order_pegged
// - place_take_order
// - cancel_order
// - cancel_order_by_client_id
// - cancel_all_orders
// `place_order` that wraps `place_order` 
// `edit_order` that wraps `edit_order` and stores 

#[account]
pub struct TWAPMarket {
    pub market: Pubkey,
}

#[derive(Accounts)]
pub struct CreateTWAPMarket<'info> {
    pub market: AccountLoader<'info, Market>,
    #[account(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<TWAPMarket>(),
        seeds = [b"twap_market", market.key().as_ref()],
        bump
    )]
    pub twap_market: Account<'info, TWAPMarket>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub open_orders_account: UncheckedAccount<'info>,
    pub open_orders_admin: Account<'info, TWAPMarket>,
    #[account(mut)]
    pub user_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub event_heap: UncheckedAccount<'info>,
    #[account(mut)]
    pub market_vault: UncheckedAccount<'info>,
    pub oracle_a: Option<UncheckedAccount<'info>>,
    pub oracle_b: Option<UncheckedAccount<'info>>,
    pub token_program: UncheckedAccount<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Copy, Clone)]
pub struct PlaceOrderArgs {
    pub side: Side,
    pub price_lots: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub client_order_id: u64,
    pub order_type: PlaceOrderType,
    pub expiry_timestamp: u64,
    pub self_trade_behavior: SelfTradeBehavior,
    pub limit: u8,
}

#[derive(
    Eq,
    PartialEq,
    Copy,
    Clone,
    Debug,
    AnchorSerialize,
    AnchorDeserialize,
)]
#[repr(u8)]
pub enum SelfTradeBehavior {
    DecrementTake = 0,
    CancelProvide = 1,
    AbortTransaction = 2,
}

#[derive(
    Eq,
    PartialEq,
    Copy,
    Clone,
    Debug,
    AnchorSerialize,
    AnchorDeserialize,
)]
#[repr(u8)]
pub enum PlaceOrderType {
    Limit = 0,
    ImmediateOrCancel = 1,
    PostOnly = 2,
    Market = 3,
    PostOnlySlide = 4,
}

#[derive(
    Eq,
    PartialEq,
    Copy,
    Clone,
    Debug,
    AnchorSerialize,
    AnchorDeserialize,
)]
#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}

#[program]
pub mod openbook_twap {
    use super::*;

    pub fn create_twap_market(
        ctx: Context<CreateTWAPMarket>,
    ) -> Result<()> {
        let market = ctx.accounts.market.load()?;
        let twap_market = &mut ctx.accounts.twap_market;

        require!(market.open_orders_admin == twap_market.key(), OpenBookTWAPError::InvalidOpenOrdersAdmin);
        require!(market.close_market_admin == twap_market.key(), OpenBookTWAPError::InvalidCloseMarketAdmin);
        require!(market.time_expiry == 0, OpenBookTWAPError::NonZeroExpiry);

        twap_market.market = ctx.accounts.market.key();

        Ok(())
    }

    pub fn place_order(
        ctx: Context<PlaceOrder>,
        place_order_args: PlaceOrderArgs
    ) -> Result<()> {
        Ok(())
    }
}

#[error_code]
pub enum OpenBookTWAPError {
    #[msg("The `open_orders_admin` of the underlying market must be equal to the `TWAPMarket` PDA")]
    InvalidOpenOrdersAdmin,
    #[msg("The `close_market_admin` of the underlying market must be equal to the `TWAPMarket` PDA")]
    InvalidCloseMarketAdmin,
    #[msg("Market must not expire (have `time_expiry` == 0)")]
    NonZeroExpiry,
}
