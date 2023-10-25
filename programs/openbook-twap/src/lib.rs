use anchor_lang::prelude::*;
use openbook_v2::cpi;
use openbook_v2::program::OpenbookV2;
use openbook_v2::state::{BookSide, Market};
use num::integer::Average;
use std::cell::Ref;
//use openbook_v2::PlaceOrderArgs;

const MAX_BPS: u16 = 10_000;
const TWAP_MARKET: &[u8] = b"twap_market";

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
// DONE: 
// `place_order` that wraps `place_order`
// `edit_order` that wraps `edit_order` and stores
// `cancel_order` that wraps `cancel_order`

#[account]
pub struct TWAPMarket {
    pub market: Pubkey,
    pub pda_bump: u8,
    pub twap_oracle: TWAPOracle,
}

impl TWAPMarket {
    pub fn get_twap_market_seeds<'a>(
        market: &'a Pubkey, 
        bump: &'a u8
    ) -> [&'a [u8]; 3] {
        [TWAP_MARKET.as_ref(), market.as_ref(), bytemuck::bytes_of(bump)]
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TWAPOracle {
    pub last_updated_slot: u64,
    pub last_observed_slot: u64,
    pub last_observation: u64,
    pub observation_aggregator: u128,
    /// The most, in basis points, an observation can change per update.
    /// For example, if it is 100 (1%), then the new observation can be between
    /// last_observation * 0.99 and last_observation * 1.01
    pub max_observation_change_per_update_bps: u16,
    pub _padding: [u8; 6],
}

impl Default for TWAPOracle {
    fn default() -> Self {
        // Get the current slot at TWAPOracle initialization - if this fails we'll load the default clock
        // Since the default Clock.slot is 0 (because slots are expressed as u64)
        // This will give us 0 as the slot which we had anyways
        let clock = Clock::get().unwrap_or(Clock::default());
        Self {
            last_updated_slot: clock.slot,
            last_observed_slot: 0,
            last_observation: 0,
            observation_aggregator: 0,
            max_observation_change_per_update_bps: 100,
            _padding: Default::default(),
        }
    }
}

impl TWAPOracle {
    pub fn update_oracle(&mut self, bids: Ref<'_, BookSide>, asks: Ref<'_, BookSide>) {
        let clock = Clock::get().unwrap();

        if self.last_observed_slot < clock.slot {
            self.last_observed_slot = clock.slot;

            let unix_ts: u64 = clock.unix_timestamp.try_into().unwrap();

            let best_bid = bids.best_price(unix_ts, None);
            let best_ask = asks.best_price(unix_ts, None);

            if let (Some(best_bid), Some(best_ask)) = (best_bid, best_ask) {
                // we use average_ceil because (best_bid + best_ask) / 2 can overflow
                let spot_price = best_bid.average_ceil(&best_ask) as u64;

                let last_observation = self.last_observation;

                let observation = if spot_price > last_observation {
                    let max_observation = last_observation
                        .saturating_mul((MAX_BPS + self.max_observation_change_per_update_bps) as u64)
                        .saturating_div(MAX_BPS as u64)
                        .saturating_add(1);        

                    std::cmp::min(spot_price, max_observation)
                } else {
                    let min_observation = last_observation
                        .saturating_mul((MAX_BPS - self.max_observation_change_per_update_bps) as u64)
                        .saturating_div(MAX_BPS as u64);
                    
                    std::cmp::max(spot_price, min_observation)
                };

                let weighted_observation = observation * (clock.slot - self.last_updated_slot);

                self.last_updated_slot = clock.slot;
                self.last_observation = observation;
                self.observation_aggregator += weighted_observation as u128;


            }
        }
    }
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
    /// CHECK: verified in CPI
    #[account(mut)]
    pub open_orders_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub twap_market: Account<'info, TWAPMarket>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub user_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub event_heap: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub market_vault: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    pub token_program: UncheckedAccount<'info>,
    pub openbook_program: Program<'info, OpenbookV2>,
}

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub twap_market: Account<'info, TWAPMarket>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub open_orders_account: UncheckedAccount<'info>,
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids:  AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    pub openbook_program: Program<'info, OpenbookV2>,
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

impl From<PlaceOrderArgs> for openbook_v2::PlaceOrderArgs {
    fn from(args: PlaceOrderArgs) -> Self {
        Self {
            side: args.side.into(),
            price_lots: args.price_lots,
            max_base_lots: args.max_base_lots,
            max_quote_lots_including_fees: args.max_quote_lots_including_fees,
            client_order_id: args.client_order_id,
            order_type: args.order_type.into(),
            expiry_timestamp: args.expiry_timestamp.into(),
            self_trade_behavior: args.self_trade_behavior.into(),
            limit: args.limit,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum SelfTradeBehavior {
    DecrementTake = 0,
    CancelProvide = 1,
    AbortTransaction = 2,
}

impl From<SelfTradeBehavior> for openbook_v2::state::SelfTradeBehavior {
    fn from(behavior: SelfTradeBehavior) -> Self {
        match behavior {
            SelfTradeBehavior::DecrementTake => Self::DecrementTake,
            SelfTradeBehavior::CancelProvide => Self::CancelProvide,
            SelfTradeBehavior::AbortTransaction => Self::AbortTransaction,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum PlaceOrderType {
    Limit = 0,
    ImmediateOrCancel = 1,
    PostOnly = 2,
    Market = 3,
    PostOnlySlide = 4,
}

impl From<PlaceOrderType> for openbook_v2::state::PlaceOrderType {
    fn from(order_type: PlaceOrderType) -> Self {
        match order_type {
            PlaceOrderType::Limit => Self::Limit,
            PlaceOrderType::ImmediateOrCancel => Self::ImmediateOrCancel,
            PlaceOrderType::PostOnly => Self::PostOnly,
            PlaceOrderType::Market => Self::Market,
            PlaceOrderType::PostOnlySlide => Self::PostOnlySlide,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}

impl From<Side> for openbook_v2::state::Side {
    fn from(side: Side) -> Self {
        match side {
            Side::Bid => Self::Bid,
            Side::Ask => Self::Ask,
        }
    }
}

#[program]
pub mod openbook_twap {
    use super::*;

    pub fn create_twap_market(ctx: Context<CreateTWAPMarket>) -> Result<()> {
        let market = ctx.accounts.market.load()?;
        let twap_market = &mut ctx.accounts.twap_market;

        require!(
            market.open_orders_admin == twap_market.key(),
            OpenBookTWAPError::InvalidOpenOrdersAdmin
        );
        require!(
            market.close_market_admin == twap_market.key(),
            OpenBookTWAPError::InvalidCloseMarketAdmin
        );
        require!(market.time_expiry == 0, OpenBookTWAPError::NonZeroExpiry);
        require!(
            market.oracle_a.is_none(),
            OpenBookTWAPError::NoOracles
        );
        require!(
            market.oracle_b.is_none(),
            OpenBookTWAPError::NoOracles
        );

        twap_market.pda_bump = *ctx.bumps.get("twap_market").unwrap();
        twap_market.market = ctx.accounts.market.key();
        twap_market.twap_oracle = TWAPOracle::default();

        Ok(())
    }

    pub fn place_order(ctx: Context<PlaceOrder>, place_order_args: PlaceOrderArgs) -> Result<()> {
        let oracle = &mut ctx.accounts.twap_market.twap_oracle;

        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        oracle.update_oracle(bids, asks);

        let market_key = ctx.accounts.market.key();
        let twap_market_seeds = TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer = &[&twap_market_seeds[..]];

        let cpi_program = ctx.accounts.openbook_program.to_account_info();
        let cpi_accs = cpi::accounts::PlaceOrder {
            signer: ctx.accounts.signer.to_account_info(),
            open_orders_account: ctx.accounts.open_orders_account.to_account_info(),
            open_orders_admin: Some(ctx.accounts.twap_market.to_account_info()),
            user_token_account: ctx.accounts.user_token_account.to_account_info(),
            market: ctx.accounts.market.to_account_info(),
            bids: ctx.accounts.bids.to_account_info(),
            asks: ctx.accounts.asks.to_account_info(),
            event_heap: ctx.accounts.event_heap.to_account_info(),
            market_vault: ctx.accounts.market_vault.to_account_info(),
            oracle_a: None,
            oracle_b: None,
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accs, signer);

        openbook_v2::cpi::place_order(cpi_ctx, place_order_args.into())?;

        Ok(())
    }

    pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
        let oracle = &mut ctx.accounts.twap_market.twap_oracle;

        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        oracle.update_oracle(bids, asks);

        let market_key = ctx.accounts.market.key();

        let seeds = TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer_seeds = &[&seeds[..]];

        openbook_v2::cpi::cancel_order(
            CpiContext::new_with_signer(
            ctx.accounts.openbook_program.to_account_info(),
            cpi::accounts::CancelOrder {
                        signer: ctx.accounts.signer.to_account_info(),
                        open_orders_account: ctx.accounts.open_orders_account.to_account_info(),
                        market: ctx.accounts.market.to_account_info(),
                        bids:  ctx.accounts.bids.to_account_info(),
                        asks: ctx.accounts.asks.to_account_info(),
                    },
                    signer_seeds,
            ), 
            order_id
        )?;

        Ok(())
    }
}

#[error_code]
pub enum OpenBookTWAPError {
    #[msg(
        "The `open_orders_admin` of the underlying market must be equal to the `TWAPMarket` PDA"
    )]
    InvalidOpenOrdersAdmin,
    #[msg(
        "The `close_market_admin` of the underlying market must be equal to the `TWAPMarket` PDA"
    )]
    InvalidCloseMarketAdmin,
    #[msg("Market must not expire (have `time_expiry` == 0)")]
    NonZeroExpiry,
    #[msg("Oracle-pegged trades mess up the TWAP so oracles and oracle-pegged trades aren't allowed")]
    NoOracles,
}
