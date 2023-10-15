# OpenBook TWAP

A program that allows you to fetch on-chain TWAPs from OpenBook markets.

## Oracle

We provide a [Uniswap V2](https://uniswap.org/whitepaper.pdf)-style oracle.
This means:
- For each market, there is a running price aggregator.
- Before the first trade in a slot, we add the current spot price (defined as the
average of the best bid and the best offer) to the aggregator.
- To fetch a TWAP between two points, one can compute (current_aggregator - past_aggregator) / slots_elapsed.
Just like in Uniswap V2, the client is responsible for storing past aggregator points.

## Interacting with a TWAP market

The TWAP market program decorates the OpenBook v2 program. It does this by having
a wrapper `TWAPMarket` account that stores the TWAP and is the `open_orders_admin`
of the underlying `openbook_v2` market. That way, all order book state transitions
are forced to proxy through the `twap_market` program.
