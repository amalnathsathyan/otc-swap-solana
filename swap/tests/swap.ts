import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Swap } from "../target/types/swap";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import * as token from "@solana/spl-token";
import { expect } from "chai";

describe("swap program - create offer", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Swap as Program<Swap>;
  const maker = Keypair.generate();
  
  let inputMint: PublicKey;
  let outputMint: PublicKey;
  let makerTokenAccount: PublicKey;
  let vaultTokenAccount: PublicKey;
  let vaultAuthority: PublicKey;

  before(async () => {
    // Restart local validator before each test suite
    try {
      const connection = provider.connection;
      
      // Airdrop SOL to maker with retry
      const airdropAmount = anchor.web3.LAMPORTS_PER_SOL * 10;
      const tx = await connection.requestAirdrop(maker.publicKey, airdropAmount);
      
      const latestBlockhash = await connection.getLatestBlockhash();
      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: tx
      });

      // Create test tokens with unique keypairs
      inputMint = await token.createMint(
        provider.connection, 
        provider.wallet.payer, 
        maker.publicKey, 
        null, 
        9
      );

      outputMint = await token.createMint(
        provider.connection, 
        provider.wallet.payer, 
        maker.publicKey, 
        null, 
        9
      );

      // Create maker's token account and mint tokens
      makerTokenAccount = await token.createAccount(
        provider.connection, 
        provider.wallet.payer, 
        inputMint, 
        maker.publicKey
      );

      await token.mintTo(
        provider.connection,
        provider.wallet.payer,
        inputMint,
        makerTokenAccount,
        maker.publicKey,
        1000000000 // 1000 tokens
      );

      // Find vault authority PDA
      [vaultAuthority] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), inputMint.toBuffer()],
        program.programId
      );

      // Create vault token account with explicit signer
      vaultTokenAccount = await token.createAccount(
        provider.connection,
        provider.wallet.payer,
        inputMint,
        vaultAuthority,
        maker
      );

    } catch (error) {
      console.error("Setup error:", error);
      throw error;
    }
  });

  it("should create an offer successfully", async () => {
    // Generate unique offer ID
    const offerId = new anchor.BN(Date.now());

    // Find offer PDA
    const [offerPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"),
        maker.publicKey.toBuffer(),
        offerId.toBuffer("le", 8)
      ],
      program.programId
    );

    // Find whitelist PDA
    const [whitelistPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("whitelist"),
        offerPDA.toBuffer()
      ],
      program.programId
    );

    // Find fee config PDA
    const [feeConfigPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("fee_config"),
        offerPDA.toBuffer()
      ],
      program.programId
    );

    // Prepare transaction
    const tx = await program.methods
      .createOfferAndSendTokensToVault(
        offerId,
        inputMint,
        outputMint,
        new anchor.BN(500000000), // 500 tokens
        new anchor.BN(1000000000), // Expected 1000 tokens
        new anchor.BN(Math.floor(Date.now() / 1000) + 86400), // 24h from now
        new anchor.BN(10), // 10% fee
        provider.wallet.publicKey // Fee wallet
      )
      .accounts({
        maker: maker.publicKey,
        offer: offerPDA,
        whitelist: whitelistPDA,
        offerFeeConfig: feeConfigPDA,
        inputTokenMint: inputMint,
        outputTokenMint: outputMint,
        makerTokenAccount: makerTokenAccount,
        vaultTokenAccount: vaultTokenAccount,
        vaultAuthority: vaultAuthority,
        tokenProgram: token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([maker])
      .rpc();

    // Verify vault token account balance
    const vaultBalance = await token.getAccount(
      provider.connection, 
      vaultTokenAccount
    );

    expect(vaultBalance.amount).to.equal(BigInt(500000000));
  });

  it("should fail creating offer with insufficient tokens", async () => {
    const offerId = new anchor.BN(Date.now());

    try {
      await program.methods
        .createOfferAndSendTokensToVault(
          offerId,
          inputMint,
          outputMint,
          new anchor.BN(2000000000), // More than account balance
          new anchor.BN(1000000000),
          new anchor.BN(Math.floor(Date.now() / 1000) + 86400),
          new anchor.BN(10),
          provider.wallet.publicKey
        )
        .accounts({
          maker: maker.publicKey,
          makerTokenAccount: makerTokenAccount,
          vaultTokenAccount: vaultTokenAccount,
          inputTokenMint: inputMint,
          outputTokenMint: outputMint
        })
        .signers([maker])
        .rpc();
      
      expect.fail("Should have thrown an error");
    } catch (err) {
      expect(err.message).to.include("insufficient funds");
    }
  });
});