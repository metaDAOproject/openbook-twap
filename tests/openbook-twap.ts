import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookTwap } from "../target/types/openbook_twap";
import {
  OpenBookV2Client,
  BooksideSpace,
  EventHeapSpace,
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
    let baseMint = await createMint(
      connection,
      provider.wallet.payer,
      mintAuthority.publicKey,
      null,
      6
    );
    let quoteMint = await createMint(
      connection,
      provider.wallet.payer,
      mintAuthority.publicKey,
      null,
      6
    );

    let quoteAccount = await createAccount(
      connection,
      provider.wallet.payer,
      quoteMint,
      provider.wallet.payer.publicKey,
    );

    let baseAccount = await createAccount(
      connection,
      provider.wallet.payer,
      baseMint,
      provider.wallet.payer.publicKey,
    );

    await mintTo(
      connection,
      provider.wallet.payer,
      quoteMint,
      quoteAccount,
      mintAuthority,
      1_000_000_000_000_000n
    );

    await mintTo(
      connection,
      provider.wallet.payer,
      baseMint,
      baseAccount,
      mintAuthority,
      1_000_000_000_000_000n
    );

    let marketKP = Keypair.generate();

    let [twapMarket] = PublicKey.findProgramAddressSync(
      [anchor.utils.bytes.utf8.encode("twap_market"), marketKP.publicKey.toBuffer()],
      openbookTwap.programId
    );

    let market = await openbook.createMarket(
      provider.wallet.payer,
      "MARKET0",
      quoteMint,
      baseMint,
      new BN(1),
      new BN(1),
      new BN(0),
      new BN(0),
      new BN(0),
      null,
      null,
      twapMarket,
      null,
      twapMarket,
      undefined,
      marketKP
    );

    
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
    //  "MARKET1",
    //  quoteMint,
    //  baseMint,
    //  new BN(10),
    //  new BN(100),
    //  new BN(-200),
    //  new BN(400),
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
      baseAccount,
      quoteAccount,
      new BN(1_000_000_000),
      new BN(1_000_000_000),
    );

    let placeOrderArgs = {
      side: { bid: {} },
      priceLots: I80F48.fromNumber(1000).toTwos(),
      maxBaseLots: new BN(1),
      maxQuoteLotsIncludingFees: new BN(10000),
      clientOrderId: new BN(0),
      orderType: { limit: {} },
      expiryTimestamp: new BN(0),
      selfTradeBehavior: { decrementTake: {} },
      limit: 10,
    };


    //await openbookProgram.methods
    //  .placeOrder(placeOrderArgs)
    //  .accounts({
    //    signer: payer.publicKey,
    //    asks: storedMarket.asks,
    //    bids: storedMarket.bids,
    //    marketVault: storedMarket.marketQuoteVault,
    //    eventHeap: storedMarket.eventHeap,
    //    market: market,
    //    oracleA: null,
    //    oracleB: null,
    //    openOrdersAdmin: null,
    //    openOrdersAccount: openOrders,
    //    userTokenAccount: quoteAccount,
    //    tokenProgram: TOKEN_PROGRAM_ID,
    //  })
    //  .rpc();

    //console.log(await openbook.getOpenOrders(openOrders));

    //await openbook.placeOrder(openOrders, market, storedMarket, quoteAccount, null, placeOrderArgs);
    //await openbookTwapClient.placeOrder(openOrders, nonTwapMarket, storedMarket, quoteAccount, twapMarket, placeOrderArgs);
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
        userTokenAccount: quoteAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        twapMarket,
        openbookProgram: OPENBOOK_PROGRAM_ID,
      })
      .rpc();

    console.log((await openbook.getOpenOrders(openOrders)).position);

    //console.log(await openbook.getBookSide(storedMarket.bids));

    //console.log(await openbook.getLeafNodes(await openbook.getBookSide(storedMarket.bids)));

    //console.log(await getAccount(connection, quoteAccount));





    //public async placeOrder(
    //  openOrdersPublicKey: PublicKey,
    //  marketPublicKey: PublicKey,
    //  market: MarketAccount,
    //  userTokenAccount: PublicKey,
    //  openOrdersAdmin: PublicKey | null,
    //  args: PlaceOrderArgs,
    //  openOrdersDelegate?: Keypair,
    //): Promise<TransactionSignature> {
  });
});
