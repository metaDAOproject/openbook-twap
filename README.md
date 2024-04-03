# OpenBook TWAP

![License LGPLv2.1](https://img.shields.io/badge/License-LGPLv2.1-violet.svg)

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
| v0.2 | mainnet | twAP5sArq2vDS1mZCT7f4qRLwzTfHvf5Ay5R5Q5df1m |
| v0.2 | devnet  | twAP5sArq2vDS1mZCT7f4qRLwzTfHvf5Ay5R5Q5df1m |
| v0.1 | mainnet | TWAPrdhADy2aTKN5iFZtNnkQYXERD9NvKjPFVPMSCNN |
| v0.1 | devnet  | TWAPrdhADy2aTKN5iFZtNnkQYXERD9NvKjPFVPMSCNN |
| v0 | mainnet | TWAP7frdvD3ia7TWc8e9SxZMmrpd2Yf3ifSPAHS8VG3 |
| v0 | devnet  | TWAP7frdvD3ia7TWc8e9SxZMmrpd2Yf3ifSPAHS8VG3 |

All programs are immutable.

## Verifying

The program was compiled with [solana-verifiable-build](https://github.com/Ellipsis-Labs/solana-verifiable-build), which means that anyone can verify that the on-chain program matches the source code. To do so, install the CLI and run:
```
$ solana-verify verify-from-repo -um --program-id twAP5sArq2vDS1mZCT7f4qRLwzTfHvf5Ay5R5Q5df1m https://github.com/metaDAOproject/openbook-twap --library-name openbook_twap -b ellipsislabs/solana:1.16.10
```

You can also see OtterSec's attestation of this verification [here](https://verify.osec.io/status/twAP5sArq2vDS1mZCT7f4qRLwzTfHvf5Ay5R5Q5df1m).