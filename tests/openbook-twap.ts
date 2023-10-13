import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookTwap } from "../target/types/openbook_twap";

import { BankrunProvider } from "anchor-bankrun";
import { startAnchor, Clock } from "solana-bankrun";

describe("openbook-twap", async function () {
  let provider,
    connection,
    payer,
    context,
    banksClient;
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  //const program = anchor.workspace.OpenbookTwap as Program<OpenbookTwap>;
  before(async function () {
    context = await startAnchor("./", [], []);
    banksClient = context.banksClient;
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);
  });

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.createTwapMarket().rpc();
    console.log("Your transaction signature", tx);
  });
});
