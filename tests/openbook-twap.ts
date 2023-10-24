import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookTwap } from "../target/types/openbook_twap";
import {
  OpenBookV2Client,
  BooksideSpace,
  EventHeapSpace,
  PlaceOrderArgs,
  Side
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
  "opnbkNkqux64GppQhwbyEVc3axhssFhVYuwar8rDHCu"
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
  const openbook = new OpenBookV2Client(OPENBOOK_PROGRAM_ID, provider);
  const openbookTwapClient = new OpenBookV2Client(openbookTwap.programId, provider);
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
      META_AMOUNT
    );

    await mintTo(
      connection,
      provider.wallet.payer,
      USDC,
      usdcAccount,
      mintAuthority,
      USDC_AMOUNT
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

    //
    await openbookTwap.methods.createTwapMarket()
      .accounts({
        market,
        twapMarket,
      })
      .rpc();

    let storedTwapMarket = await openbookTwap.account.twapMarket.fetch(twapMarket);

    assert.ok(storedTwapMarket.market.equals(market));

    //let market = await openbook.createMarket(
    //  provider.wallet.payer,
    //  "META/USDC",
    //  USDC,
    //  META,
    //  new BN(100),
    //  new BN(1e9),
    //  new BN(0),
    //  new BN(0),
    //  new BN(0),
    //  null,
    //  null,
    //  null,
    //  null,
    //  null,
    //);

    let storedMarket = await openbook.getMarket(market);
    let openOrders = await openbook.createOpenOrders(market, new BN(1));

    await openbook.deposit(
      openOrders,
      await openbook.getOpenOrders(openOrders),
      storedMarket,
      metaAccount,
      usdcAccount,
      new BN(META_AMOUNT),
      new BN(USDC_AMOUNT),
    );

    let placeOrderArgs: PlaceOrderArgs = {
      side: Side.Bid,
      priceLots: new BN(10_000), // 1 META for 1 USDC
      maxBaseLots: new BN(1),
      maxQuoteLotsIncludingFees: new BN(10_000),
      clientOrderId: new BN(1),
      orderType: { limit: {} },
      expiryTimestamp: new BN(0),
      selfTradeBehavior: { decrementTake: {} },
      limit: 255,
    };
    //await openbook.placeOrder(openOrders, market, storedMarket, usdcAccount, null, placeOrderArgs);

    ////console.log(await openbook.getOpenOrders(openOrders));

    await openbookTwap.methods
      .placeOrder(placeOrderArgs)
      .accounts({
        signer: payer.publicKey,
        asks: storedMarket.asks,
        bids: storedMarket.bids,
        marketVault: storedMarket.marketQuoteVault,
        eventHeap: storedMarket.eventHeap,
        market: market,
        openOrdersAccount: openOrders,
        userTokenAccount: usdcAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        twapMarket,
        openbookProgram: OPENBOOK_PROGRAM_ID,
      })
      .rpc();

    console.log((await openbook.getOpenOrders(openOrders)).position);

    //console.log(await openbook.getBookSide(storedMarket.bids));

    //console.log(await openbook.getLeafNodes(await openbook.getBookSide(storedMarket.bids)));

    //console.log(await getAccount(connection, quoteAccount));
  });
});
