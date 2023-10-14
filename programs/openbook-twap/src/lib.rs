use anchor_lang::prelude::*;
use openbook_v2::cpi;
use openbook_v2::state::{Market, BookSide};

declare_id!("EgYfg4KUAbXP4UfTrsauxvs75QFf28b3MVEV8qFUGBRh");

// `create_twap_market` that verifies that the `open_orders_authority` and 
// `close_market_admin` are set to `twap_market`
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

#[program]
pub mod openbook_twap {
    use super::*;

    pub fn create_twap_market(
        ctx: Context<CreateTWAPMarket>,
    ) -> Result<()> {
        let market = ctx.accounts.market.load()?;
        let twap_market = &mut ctx.accounts.twap_market;

        msg!("{:?}", market.open_orders_admin);
        require!(market.open_orders_admin == twap_market.key(), OpenBookTWAPError::InvalidOpenOrdersAuthority);

        twap_market.market = ctx.accounts.market.key();

        Ok(())
    }
}

#[error_code]
pub enum OpenBookTWAPError {
    #[msg("The `open_orders_authority` of the underlying market must be equal to the `TWAPMarket` PDA")]
    InvalidOpenOrdersAuthority,
}
