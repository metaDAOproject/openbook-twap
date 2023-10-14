import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookTwap } from "../target/types/openbook_twap";
import { OpenBookV2Client, BooksideSpace, EventHeapSpace } from "@openbook-dex/openbook-v2";

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

const OPENBOOK_PROGRAM_ID = new PublicKey("opnbkNkqux64GppQhwbyEVc3axhssFhVYuwar8rDHCu");

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
    let baseMint = await createMint(connection, provider.wallet.payer, mintAuthority.publicKey, null, 9);
    let quoteMint = await createMint(connection, provider.wallet.payer, mintAuthority.publicKey, null, 9);

    await openbook.createMarket(provider.wallet.payer, "MARKET0", quoteMint, baseMint, new BN(1), new BN(1), new BN(0), new BN(0), new BN(0), null, null, null, null, null);
    const tx = await openbookTwap.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
