import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookTwap } from "../target/types/openbook_twap";
import { OpenBookV2Client } from "@openbook-dex/openbook-v2";

const { PublicKey, Signer, Keypair, SystemProgram } = anchor.web3;
const { BN, Program } = anchor;

const OPENBOOK_PROGRAM_ID = new PublicKey("8qkavBpvoHVYkmPhu6QRpXRX39Kcop9uMXvZorBAz43o");

export type OpenBookProgram = Program<OpenBookIDL>;

describe("openbook-twap", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const openbookTwap = anchor.workspace.OpenbookTwap as Program<OpenbookTwap>;
  const openbook = new OpenBookV2Client(OPENBOOK_PROGRAM_ID, provider);

  it("Is initialized!", async () => {
    //await openbook.createMarket(provider.wallet.payer);
    const tx = await openbookTwap.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
