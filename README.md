# OpenBook TWAP

A program that allows you to fetch on-chain TWAPs from OpenBook V2 markets.

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

## Deployed versions

| tag  | network | program ID                                  |
| ---- | ------- | ------------------------------------------- |
| v0 | mainnet | TWAPjDPjuGaRMrRzW186n94RrZFU4tdWAL1Mk1NMWgk |
| v0 | devnet  | TWAPjDPjuGaRMrRzW186n94RrZFU4tdWAL1Mk1NMWgk |

All programs are immutable.

## Verifying

The program was compiled with [solana-verifiable-build](https://github.com/Ellipsis-Labs/solana-verifiable-build), which means that anyone can verify that the on-chain program matches the source code. To do so, install the CLI and run:
```
$ solana-verify verify-from-repo -um --program-id TWAPjDPjuGaRMrRzW186n94RrZFU4tdWAL1Mk1NMWgk https://github.com/metaDAOproject/openbook-twap --library-name openbook_twap -b ellipsislabs/solana:1.16.9
```