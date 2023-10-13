use anchor_lang::prelude::*;
use openbook_v2::cpi;
use openbook_v2::state::Market;

declare_id!("EgYfg4KUAbXP4UfTrsauxvs75QFf28b3MVEV8qFUGBRh");

#[account]
pub struct TWAPMarket {
    pub underlying_market: Pubkey,
}

#[program]
pub mod openbook_twap {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn create_twap_market(ctx: Context<CreateTWAPMarket>) -> Result<()> {
        Ok(())
    }

    // create market
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct CreateTWAPMarket<'info> {
    pub underlying_market: AccountLoader<'info, Market>,
}
