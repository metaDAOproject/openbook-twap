import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookTwap } from "../target/types/openbook_twap";
import {
  OpenBookV2Client,
  BooksideSpace,
  EventHeapSpace,
  PlaceOrderArgs,
  PlaceOrderPeggedArgs,
  Side,
  OrderType,
  SelfTradeBehavior,
  PlaceTakeOrderArgs,
} from "@openbook-dex/openbook-v2";

import { expect, assert } from "chai";
import { I80F48 } from "@blockworks-foundation/mango-client";

const { PublicKey, Signer, Keypair, SystemProgram } = anchor.web3;
const { BN, Program } = anchor;

import { IDL, OpenbookV2 } from "./fixtures/openbook_v2";

import {
  createMint,
  createAccount,
  createAssociatedTokenAccount,
  mintTo,
  getAccount,
  mintToOverride,
  getMint,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

const OPENBOOK_PROGRAM_ID = new PublicKey(
  "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb"
);

export type OpenBookProgram = Program<OpenBookIDL>;

const META_AMOUNT = 100n * 1_000_000_000n;
const USDC_AMOUNT = 1000n * 1_000_000n;

describe("openbook-twap", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const payer = provider.wallet.payer;

  const openbookTwap = anchor.workspace.OpenbookTwap as Program<OpenbookTwap>;
  const openbook = new OpenBookV2Client(provider);
  const openbookProgram = new Program(
    IDL,
    OPENBOOK_PROGRAM_ID
  );

  it("Is initialized!", async () => {
    let mintAuthority = Keypair.generate();
    let META = await createMint(
      connection,
      provider.wallet.payer,
      mintAuthority.publicKey,
      null,
      6
    );

    let USDC = await createMint(
      connection,
      provider.wallet.payer,
      mintAuthority.publicKey,
      null,
      6
    );

    let usdcAccount = await createAccount(
      connection,
      provider.wallet.payer,
      USDC,
      provider.wallet.payer.publicKey,
    );

    let metaAccount = await createAccount(
      connection,
      provider.wallet.payer,
      META,
      provider.wallet.payer.publicKey,
    );

    await mintTo(
      connection,
      provider.wallet.payer,
      META,
      metaAccount,
      mintAuthority,
      META_AMOUNT * 50n
    );

    await mintTo(
      connection,
      provider.wallet.payer,
      USDC,
      usdcAccount,
      mintAuthority,
      USDC_AMOUNT * 50n
    );
    
    let marketKP = Keypair.generate();
    
    let [twapMarket] = PublicKey.findProgramAddressSync(
      [anchor.utils.bytes.utf8.encode("twap_market"), marketKP.publicKey.toBuffer()],
      openbookTwap.programId
    );

    let market = await openbook.createMarket(
      provider.wallet.payer,
      "META/USDC",
      USDC,
      META,
      new BN(100),
      new BN(1e9),
      new BN(0),
      new BN(0),
      new BN(0),
      null,
      null,
      twapMarket,
      null,
      twapMarket,
      { confFilter: 0.1, maxStalenessSlots: 100 },
      marketKP
    );

    await openbookTwap.methods.createTwapMarket(new BN(550))
      .accounts({
        market,
        twapMarket,
      })
      .rpc();

    let storedTwapMarket = await openbookTwap.account.twapMarket.fetch(twapMarket);

    assert.ok(storedTwapMarket.market.equals(market));

    let storedMarket = await openbook.getMarket(market);

    let oos = [];

    const NUM_ORDERS = 96;

    for (let i = 0; i < Math.floor(NUM_ORDERS / 24) ; i++) {
      let openOrders = await openbook.createOpenOrders(market, new BN(i + 1), `oo${i}`)
      oos.push(openOrders)
      console.log(`Created oo${i}`)
      await openbook.deposit(
        oos[i],
        await openbook.getOpenOrders(oos[i]),
        storedMarket,
        metaAccount,
        usdcAccount,
        new BN(META_AMOUNT),
        new BN(USDC_AMOUNT),
      );

      console.log(`Deposited to oo${i}`)
    }

    let buyArgs: PlaceOrderArgs = {
      side: Side.Bid,
      priceLots: new BN(500), // 1 META for 1 USDC
      maxBaseLots: new BN(1),
      maxQuoteLotsIncludingFees: new BN(500),
      clientOrderId: new BN(1),
      orderType: OrderType.Limit,
      expiryTimestamp: new BN(0),
      selfTradeBehavior: SelfTradeBehavior.DecrementTake,
      limit: 255,
    };

    let sellArgs: PlaceOrderArgs = {
      side: Side.Ask,
      priceLots: new BN(550), // 1 META for 1.2 USDC
      maxBaseLots: new BN(1),
      maxQuoteLotsIncludingFees: new BN(550),
      clientOrderId: new BN(2),
      orderType: OrderType.Limit,
      expiryTimestamp: new BN(0),
      selfTradeBehavior: SelfTradeBehavior.DecrementTake,
      limit: 255,
    };

    let manipulatedBuyArgs: PlaceOrderArgs = {
      side: Side.Bid,
      priceLots: new BN(1), 
      maxBaseLots: new BN(1),
      maxQuoteLotsIncludingFees: new BN(500),
      clientOrderId: new BN(1),
      orderType: OrderType.Limit,
      expiryTimestamp: new BN(0),
      selfTradeBehavior: SelfTradeBehavior.DecrementTake,
      limit: 255,
    };

    let manipulatedSellArgs: PlaceOrderArgs = {
      side: Side.Ask,
      priceLots: new BN(100_000_000_000_000), 
      maxBaseLots: new BN(1),
      maxQuoteLotsIncludingFees: new BN(550),
      clientOrderId: new BN(2),
      orderType: OrderType.Limit,
      expiryTimestamp: new BN(0),
      selfTradeBehavior: SelfTradeBehavior.DecrementTake,
      limit: 255,
    };

    for (let i = 0; i < oos.length; i++) {
      for (let j = 0; j < 12; j++) {
        let idx: number = j + (i * 12)

        if ((i > 0) && (i % 2 == 0)) {
          await openbookTwap.methods
            .placeOrder(manipulatedBuyArgs)
            .accounts({
              signer: payer.publicKey,
              asks: storedMarket.asks,
              bids: storedMarket.bids,
              marketVault: storedMarket.marketQuoteVault,
              eventHeap: storedMarket.eventHeap,
              market,
              openOrdersAccount: oos[i],
              userTokenAccount: usdcAccount,
              tokenProgram: TOKEN_PROGRAM_ID,
              twapMarket,
              openbookProgram: OPENBOOK_PROGRAM_ID,
            })
            .rpc();

          await openbookTwap.methods
            .placeOrder(manipulatedSellArgs)
            .accounts({
              signer: payer.publicKey,
              asks: storedMarket.asks,
              bids: storedMarket.bids,
              marketVault: storedMarket.marketBaseVault,
              eventHeap: storedMarket.eventHeap,
              market,
              openOrdersAccount: oos[i],
              userTokenAccount: metaAccount,
              tokenProgram: TOKEN_PROGRAM_ID,
              twapMarket,
              openbookProgram: OPENBOOK_PROGRAM_ID,
            })
            .rpc();
    
            let manipulatedMarket = await openbookTwap.account.twapMarket.fetch(twapMarket);
            console.log("Manipulated observation: " + idx + " = " + manipulatedMarket.twapOracle.lastObservation.toNumber());
      
        } else {
          await openbookTwap.methods
          .placeOrder(buyArgs)
          .accounts({
            signer: payer.publicKey,
            asks: storedMarket.asks,
            bids: storedMarket.bids,
            marketVault: storedMarket.marketQuoteVault,
            eventHeap: storedMarket.eventHeap,
            market,
            openOrdersAccount: oos[i],
            userTokenAccount: usdcAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            twapMarket,
            openbookProgram: OPENBOOK_PROGRAM_ID,
          })
          .rpc();

          await openbookTwap.methods
            .placeOrder(sellArgs)
            .accounts({
              signer: payer.publicKey,
              asks: storedMarket.asks,
              bids: storedMarket.bids,
              marketVault: storedMarket.marketBaseVault,
              eventHeap: storedMarket.eventHeap,
              market,
              openOrdersAccount: oos[i],
              userTokenAccount: metaAccount,
              tokenProgram: TOKEN_PROGRAM_ID,
              twapMarket,
              openbookProgram: OPENBOOK_PROGRAM_ID,
            })
            .rpc();

          let healthyMarket = await openbookTwap.account.twapMarket.fetch(twapMarket);
          console.log("Healthy sell observation: " + idx + " = " + healthyMarket.twapOracle.lastObservation.toNumber());
        }
      }
    }

    let storedTwapMarket2 = await openbookTwap.account.twapMarket.fetch(twapMarket);
    console.log("Final oracle observation = " + storedTwapMarket2.twapOracle.lastObservation.toNumber());
  });
});
