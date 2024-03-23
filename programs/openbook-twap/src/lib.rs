use anchor_lang::prelude::*;
use num::integer::Average;
use openbook_v2::program::OpenbookV2;
use openbook_v2::state::{BookSide, Market};
use std::cell::Ref;

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "openbook_twap",
    project_url: "https://www.openbook-solana.com",
    contacts: "discord:binye,discord:skrrb,email:soundsonacid@gmail.com,email:metaproph3t@protonmail.com",
    policy: "A bug bounty may or may not be awarded.",
    source_code: "https://github.com/metaDAOproject/openbook-twap",
    source_release: "v0.1",
    auditors: "None"
}

const TWAP_MARKET: &[u8] = b"twap_market";

declare_id!("twAP5sArq2vDS1mZCT7f4qRLwzTfHvf5Ay5R5Q5df1m");

#[account]
pub struct TWAPMarket {
    pub market: Pubkey,
    pub pda_bump: u8,
    pub twap_oracle: TWAPOracle,
    pub close_market_rent_receiver: Pubkey,
}

impl TWAPMarket {
    pub fn get_twap_market_seeds<'a>(market: &'a Pubkey, bump: &'a u8) -> [&'a [u8]; 3] {
        [
            TWAP_MARKET.as_ref(),
            market.as_ref(),
            bytemuck::bytes_of(bump),
        ]
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TWAPOracle {
    pub expected_value: u64,
    pub initial_slot: u64,
    pub last_updated_slot: u64,
    pub last_observed_slot: u64,
    pub last_observation: u64,
    pub observation_aggregator: u128,
    pub max_observation_change_per_update_lots: u64,
}

impl TWAPOracle {
    pub fn new(expected_value: u64, max_observation_change_per_update_lots: u64) -> Self {
        // Get the current slot at TWAPOracle initialization
        // If we cannot get the clock the transaction should fail. Unwise to catch the error.
        // Starting with a time of 0 (initial solana blockchain slot) messes up later logic in unpredictable ways

        let clock = Clock::get().unwrap();
        Self {
            expected_value,
            initial_slot: clock.slot,
            last_updated_slot: clock.slot,
            last_observed_slot: clock.slot,
            last_observation: expected_value,
            observation_aggregator: expected_value as u128,
            max_observation_change_per_update_lots,
        }
    }

    pub fn update_oracle(&mut self, bids: Ref<'_, BookSide>, asks: Ref<'_, BookSide>) {
        let clock = Clock::get().unwrap();

        if self.last_observed_slot < clock.slot {
            self.last_observed_slot = clock.slot;

            let unix_ts: u64 = clock.unix_timestamp.try_into().unwrap();

            let best_bid = bids.best_price(unix_ts, None);
            let best_ask = asks.best_price(unix_ts, None);

            if let (Some(best_bid), Some(best_ask)) = (best_bid, best_ask) {
                // we don't record prices if there's a spread of more than 20%
                if best_ask > best_bid.saturating_mul(12).saturating_div(10) {
                    return;
                }

                // we use average_ceil because (best_bid + best_ask) / 2 can overflow
                let spot_price = best_bid.average_ceil(&best_ask) as u64;

                let last_observation = self.last_observation;

                let observation = if spot_price > last_observation {
                    let max_observation = last_observation
                        .saturating_add(self.max_observation_change_per_update_lots);

                    std::cmp::min(spot_price, max_observation)
                } else {
                    let min_observation = last_observation
                        .saturating_sub(self.max_observation_change_per_update_lots);

                    std::cmp::max(spot_price, min_observation)
                };

                msg!("Observation: {:?}", observation);

                let weighted_observation =
                    observation as u128 * (clock.slot - self.last_updated_slot) as u128;

                msg!("Weighted observation: {:?}", weighted_observation);

                self.last_updated_slot = clock.slot;
                self.last_observation = observation;
                self.observation_aggregator += weighted_observation;
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
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    pub openbook_program: Program<'info, OpenbookV2>,
}

#[derive(Accounts)]
pub struct PruneOrders<'info> {
    pub twap_market: Account<'info, TWAPMarket>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub open_orders_account: UncheckedAccount<'info>,
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    pub openbook_program: Program<'info, OpenbookV2>,
}

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    /// CHECK: This is a permissionless function but could be made to require the close_market_rent_receiver's signature
    #[account(mut)]
    pub close_market_rent_receiver: UncheckedAccount<'info>,
    #[account(has_one = close_market_rent_receiver)]
    pub twap_market: Account<'info, TWAPMarket>,
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
    pub token_program: UncheckedAccount<'info>,
    pub openbook_program: Program<'info, OpenbookV2>,
}

#[derive(Accounts)]
pub struct PlaceTakeOrder<'info> {
    #[account(mut)]
    pub twap_market: Account<'info, TWAPMarket>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,
    /// CHECK: verified in CPI
    pub market_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub market_base_vault: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub market_quote_vault: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub event_heap: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub user_base_account: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub user_quote_account: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    pub token_program: UncheckedAccount<'info>,
    pub openbook_program: Program<'info, OpenbookV2>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelAndPlaceOrders<'info> {
    pub signer: Signer<'info>,
    #[account(mut)]
    pub twap_market: Account<'info, TWAPMarket>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub open_orders_account: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub user_quote_account: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub user_base_account: UncheckedAccount<'info>,
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
    pub market_quote_vault: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    #[account(mut)]
    pub market_base_vault: UncheckedAccount<'info>,
    /// CHECK: verified in CPI
    pub token_program: UncheckedAccount<'info>,
    pub openbook_program: Program<'info, OpenbookV2>,
}

#[derive(Accounts)]
pub struct GetBestBidAndAsk<'info> {
    #[account(has_one = bids, has_one = asks)]
    pub market: AccountLoader<'info, Market>,
    pub bids: AccountLoader<'info, BookSide>,
    pub asks: AccountLoader<'info, BookSide>,
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

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Copy, Clone)]
pub struct PlaceTakeOrderArgs {
    pub side: Side,
    pub price_lots: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub order_type: PlaceOrderType,
    pub limit: u8,
}

impl From<PlaceTakeOrderArgs> for openbook_v2::PlaceTakeOrderArgs {
    fn from(args: PlaceTakeOrderArgs) -> Self {
        Self {
            side: args.side.into(),
            price_lots: args.price_lots,
            max_base_lots: args.max_base_lots,
            max_quote_lots_including_fees: args.max_quote_lots_including_fees,
            order_type: args.order_type.into(),
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

    /// `expected_value` will be the first observation of the TWAP, which is
    /// necessary for anti-manipulation
    pub fn create_twap_market(
        ctx: Context<CreateTWAPMarket>,
        expected_value: u64,
        max_observation_change_per_update_lots: u64,
    ) -> Result<()> {
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
        require!(
            market.consume_events_admin.is_none(),
            OpenBookTWAPError::InvalidConsumeEventsAdmin
        );
        require!(market.oracle_a.is_none(), OpenBookTWAPError::NoOracles);
        require!(market.oracle_b.is_none(), OpenBookTWAPError::NoOracles);
        require!(market.seq_num == 0, OpenBookTWAPError::InvalidSeqNum);
        require!(market.maker_fee == 0, OpenBookTWAPError::InvalidMakerFee);
        require!(market.taker_fee == 0, OpenBookTWAPError::InvalidTakerFee);

        twap_market.pda_bump = *ctx.bumps.get("twap_market").unwrap();
        twap_market.market = ctx.accounts.market.key();
        twap_market.twap_oracle =
            TWAPOracle::new(expected_value, max_observation_change_per_update_lots);
        twap_market.close_market_rent_receiver = ctx.accounts.payer.key();

        Ok(())
    }

    pub fn place_order(
        ctx: Context<PlaceOrder>,
        place_order_args: PlaceOrderArgs,
    ) -> Result<Option<u128>> {
        let oracle = &mut ctx.accounts.twap_market.twap_oracle;

        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        oracle.update_oracle(bids, asks);

        let market_key = ctx.accounts.market.key();
        let twap_market_seeds =
            TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer = &[&twap_market_seeds[..]];

        let cpi_program = ctx.accounts.openbook_program.to_account_info();
        let cpi_accs = openbook_v2::cpi::accounts::PlaceOrder {
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

        let retval = openbook_v2::cpi::place_order(cpi_ctx, place_order_args.into())?;

        Ok(retval.get())
    }

    pub fn edit_order<'info>(
        ctx: Context<'_, '_, '_, 'info, PlaceOrder<'info>>,
        client_order_id: u64,
        expected_cancel_size: i64,
        place_order: PlaceOrderArgs,
    ) -> Result<Option<u128>> {
        let oracle = &mut ctx.accounts.twap_market.twap_oracle;

        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        oracle.update_oracle(bids, asks);

        let market_key = ctx.accounts.market.key();

        let seeds =
            TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer_seeds = &[&seeds[..]];

        let retval = openbook_v2::cpi::edit_order(
            CpiContext::new_with_signer(
                ctx.accounts.openbook_program.to_account_info(),
                openbook_v2::cpi::accounts::PlaceOrder {
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
                },
                signer_seeds,
            ),
            client_order_id,
            expected_cancel_size,
            place_order.into(),
        )?;

        Ok(retval.get())
    }

    // Context<CancelOrder> endpoints
    // cancel_order_by_client_id
    // cancel_all_orders

    pub fn cancel_order_by_client_id(
        ctx: Context<CancelOrder>,
        client_order_id: u64,
    ) -> Result<i64> {
        let oracle = &mut ctx.accounts.twap_market.twap_oracle;

        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        oracle.update_oracle(bids, asks);

        let market_key = ctx.accounts.market.key();

        let seeds =
            TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer_seeds = &[&seeds[..]];

        let retval = openbook_v2::cpi::cancel_order_by_client_order_id(
            CpiContext::new_with_signer(
                ctx.accounts.openbook_program.to_account_info(),
                openbook_v2::cpi::accounts::CancelOrder {
                    signer: ctx.accounts.signer.to_account_info(),
                    open_orders_account: ctx.accounts.open_orders_account.to_account_info(),
                    market: ctx.accounts.market.to_account_info(),
                    bids: ctx.accounts.bids.to_account_info(),
                    asks: ctx.accounts.asks.to_account_info(),
                },
                signer_seeds,
            ),
            client_order_id,
        )?;

        Ok(retval.get())
    }

    pub fn cancel_all_orders(
        ctx: Context<CancelOrder>,
        side_option: Option<Side>,
        limit: u8,
    ) -> Result<()> {
        let oracle = &mut ctx.accounts.twap_market.twap_oracle;

        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        oracle.update_oracle(bids, asks);

        let market_key = ctx.accounts.market.key();

        let seeds =
            TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer_seeds = &[&seeds[..]];

        let cpi_side_option = side_option.map(|inner| match inner {
            Side::Bid => openbook_v2::state::Side::Bid,
            Side::Ask => openbook_v2::state::Side::Ask,
        });

        openbook_v2::cpi::cancel_all_orders(
            CpiContext::new_with_signer(
                ctx.accounts.openbook_program.to_account_info(),
                openbook_v2::cpi::accounts::CancelOrder {
                    signer: ctx.accounts.signer.to_account_info(),
                    open_orders_account: ctx.accounts.open_orders_account.to_account_info(),
                    market: ctx.accounts.market.to_account_info(),
                    bids: ctx.accounts.bids.to_account_info(),
                    asks: ctx.accounts.asks.to_account_info(),
                },
                signer_seeds,
            ),
            cpi_side_option,
            limit,
        )?;

        Ok(())
    }

    pub fn prune_orders(ctx: Context<PruneOrders>, limit: u8) -> Result<()> {
        let market_key = ctx.accounts.market.key();

        let seeds =
            TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer_seeds = &[&seeds[..]];

        openbook_v2::cpi::prune_orders(
            CpiContext::new_with_signer(
                ctx.accounts.openbook_program.to_account_info(),
                openbook_v2::cpi::accounts::PruneOrders {
                    close_market_admin: ctx.accounts.twap_market.to_account_info(),
                    open_orders_account: ctx.accounts.open_orders_account.to_account_info(),
                    market: ctx.accounts.market.to_account_info(),
                    bids: ctx.accounts.bids.to_account_info(),
                    asks: ctx.accounts.asks.to_account_info(),
                },
                signer_seeds,
            ),
            limit,
        )?;

        Ok(())
    }

    pub fn close_market<'info>(ctx: Context<CloseMarket>) -> Result<()> {
        let market_key = ctx.accounts.market.key();

        let seeds =
            TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer_seeds = &[&seeds[..]];

        openbook_v2::cpi::close_market(
            CpiContext::new_with_signer(
                ctx.accounts.openbook_program.to_account_info(),
                openbook_v2::cpi::accounts::CloseMarket {
                    close_market_admin: ctx.accounts.twap_market.to_account_info(),
                    market: ctx.accounts.market.to_account_info(),
                    bids: ctx.accounts.bids.to_account_info(),
                    asks: ctx.accounts.asks.to_account_info(),
                    event_heap: ctx.accounts.event_heap.to_account_info(),
                    sol_destination: ctx.accounts.close_market_rent_receiver.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                signer_seeds,
            )
        )?;
        Ok(())
    }

    // Other endpoints
    // place_take_order
    // cancel_and_place_orders

    pub fn place_take_order<'info>(
        ctx: Context<'_, '_, '_, 'info, PlaceTakeOrder<'info>>,
        args: PlaceTakeOrderArgs,
    ) -> Result<()> {
        let oracle = &mut ctx.accounts.twap_market.twap_oracle;

        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        oracle.update_oracle(bids, asks);

        let market_key = ctx.accounts.market.key();

        let seeds =
            TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer_seeds = &[&seeds[..]];

        openbook_v2::cpi::place_take_order(
            CpiContext::new_with_signer(
                ctx.accounts.openbook_program.to_account_info(),
                openbook_v2::cpi::accounts::PlaceTakeOrder {
                    signer: ctx.accounts.signer.to_account_info(),
                    penalty_payer: ctx.accounts.signer.to_account_info(),
                    market: ctx.accounts.market.to_account_info(),
                    market_authority: ctx.accounts.market_authority.to_account_info(),
                    bids: ctx.accounts.bids.to_account_info(),
                    asks: ctx.accounts.asks.to_account_info(),
                    market_base_vault: ctx.accounts.market_base_vault.to_account_info(),
                    market_quote_vault: ctx.accounts.market_quote_vault.to_account_info(),
                    event_heap: ctx.accounts.event_heap.to_account_info(),
                    user_base_account: ctx.accounts.user_base_account.to_account_info(),
                    user_quote_account: ctx.accounts.user_quote_account.to_account_info(),
                    referrer_account: None, // This acc is in the IX but idk if we are using so i marked None
                    oracle_a: None,
                    oracle_b: None,
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    open_orders_admin: Some(ctx.accounts.twap_market.to_account_info()),
                },
                signer_seeds,
            ),
            args.into(),
        )?;
        Ok(())
    }

    pub fn cancel_and_place_orders(
        ctx: Context<CancelAndPlaceOrders>,
        cancel_client_orders_ids: Vec<u64>,
        place_orders: Vec<PlaceOrderArgs>,
    ) -> Result<Vec<Option<u128>>> {
        let oracle = &mut ctx.accounts.twap_market.twap_oracle;

        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        oracle.update_oracle(bids, asks);

        let market_key = ctx.accounts.market.key();

        let seeds =
            TWAPMarket::get_twap_market_seeds(&market_key, &ctx.accounts.twap_market.pda_bump);
        let signer_seeds = &[&seeds[..]];

        let cpi_place_orders: Vec<openbook_v2::PlaceOrderArgs> =
            place_orders.iter().cloned().map(|arg| arg.into()).collect();

        let retval = openbook_v2::cpi::cancel_and_place_orders(
            CpiContext::new_with_signer(
                ctx.accounts.openbook_program.to_account_info(),
                openbook_v2::cpi::accounts::CancelAndPlaceOrders {
                    signer: ctx.accounts.signer.to_account_info(),
                    open_orders_account: ctx.accounts.open_orders_account.to_account_info(),
                    open_orders_admin: Some(ctx.accounts.twap_market.to_account_info()),
                    user_quote_account: ctx.accounts.user_quote_account.to_account_info(),
                    user_base_account: ctx.accounts.user_base_account.to_account_info(),
                    market: ctx.accounts.market.to_account_info(),
                    bids: ctx.accounts.bids.to_account_info(),
                    asks: ctx.accounts.asks.to_account_info(),
                    event_heap: ctx.accounts.event_heap.to_account_info(),
                    market_quote_vault: ctx.accounts.market_quote_vault.to_account_info(),
                    market_base_vault: ctx.accounts.market_base_vault.to_account_info(),
                    oracle_a: None,
                    oracle_b: None,
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
                signer_seeds,
            ),
            cancel_client_orders_ids,
            cpi_place_orders,
        )?;

        Ok(retval.get())
    }

    pub fn get_best_bid_and_ask(ctx: Context<GetBestBidAndAsk>) -> Result<Vec<u64>> {
        // would return a tuple but Anchor doesn't like it
        let bids = ctx.accounts.bids.load()?;
        let asks = ctx.accounts.asks.load()?;

        let clock = Clock::get()?;

        let unix_ts: u64 = clock.unix_timestamp.try_into().unwrap();

        let best_bid = bids.best_price(unix_ts, None).unwrap_or(0);
        let best_ask = asks.best_price(unix_ts, None).unwrap_or(0);

        Ok(vec![best_bid as u64, best_ask as u64])
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
    #[msg(
        "Oracle-pegged trades mess up the TWAP so oracles and oracle-pegged trades aren't allowed"
    )]
    NoOracles,
    #[msg("Maker fee must be zero")]
    InvalidMakerFee,
    #[msg("Taker fee must be zero")]
    InvalidTakerFee,
    #[msg("Seq num must be zero")]
    InvalidSeqNum,
    #[msg("Consume events admin must be None")]
    InvalidConsumeEventsAdmin,
}
