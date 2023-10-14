import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookTwap } from "../target/types/openbook_twap";
import {
  OpenBookV2Client,
  BooksideSpace,
  EventHeapSpace,
} from "@openbook-dex/openbook-v2";

import { expect, assert } from "chai";

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
  //const openbookProgram = new Program(
  //  IDL,
  //  OPENBOOK_PROGRAM_ID
  //);

  it("Is initialized!", async () => {
    let mintAuthority = Keypair.generate();
    let baseMint = await createMint(
      connection,
      provider.wallet.payer,
      mintAuthority.publicKey,
      null,
      9
    );
    let quoteMint = await createMint(
      connection,
      provider.wallet.payer,
      mintAuthority.publicKey,
      null,
      9
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
      null,
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
  });
});
