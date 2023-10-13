import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookTwap } from "../target/types/openbook_twap";

import { BankrunProvider } from "anchor-bankrun";
import { startAnchor, Clock } from "solana-bankrun";

const { PublicKey, Signer, Keypair, SystemProgram } = anchor.web3;
const { BN, Program } = anchor;

import * as OpenbookTWAPIDL from "../target/idl/openbook_twap.json";

const OPENBOOK_TWAP_PROGRAM_ID = new PublicKey(
  "EgYfg4KUAbXP4UfTrsauxvs75QFf28b3MVEV8qFUGBRh"
);

describe("openbook-twap", async function () {
  let provider,
    connection,
    payer,
    context,
    banksClient,
    openbookTwap;

  //const program = anchor.workspace.OpenbookTwap as Program<OpenbookTwap>;
  before(async function () {
    context = await startAnchor("./", [], []);
    banksClient = context.banksClient;
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);

    openbookTwap = new Program<AutocratProgram>(
      OpenbookTWAPIDL,
      OPENBOOK_TWAP_PROGRAM_ID,
      provider
    );
  });

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await openbookTwap.methods.createTwapMarket().rpc();
    console.log("Your transaction signature", tx);
  });
});
