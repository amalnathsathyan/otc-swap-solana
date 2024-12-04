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
  
  const inputMint = Keypair.generate();
  const outputMint = Keypair.generate();
  let makerTokenAccount: PublicKey;
  let vaultTokenAccount: PublicKey;
  let vaultAuthority: PublicKey;

  before(async () => {
    try {
      const connection = provider.connection;
      
      // Airdrop SOL to maker with retry
      const airdropAmount = anchor.web3.LAMPORTS_PER_SOL * 5;
      const tx = await connection.requestAirdrop(maker.publicKey, airdropAmount);
      
      const latestBlockhash = await connection.getLatestBlockhash();
      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: tx
      });

      // Create test tokens with unique keypairs
      const inputMintTx = await token.createMint(
        provider.connection, 
        maker, 
        maker.publicKey, 
        null, 
        9,
        inputMint,
      );

      const outputMintTx = await token.createMint(
        provider.connection, 
        maker, 
        maker.publicKey, 
        null, 
        9,
        outputMint
      );

      // Create maker's token account and mint tokens
      makerTokenAccount = await token.createAccount(
        provider.connection, 
        maker, 
        inputMint.publicKey, 
        maker.publicKey,
      );

      const mintTx = await token.mintTo(
        provider.connection,
        maker,
        inputMint.publicKey,
        makerTokenAccount,
        maker.publicKey,
        1000000000 // 1000 tokens
      );

      // Find vault authority PDA
      const [vaultAuthorityPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("vault"), inputMint.publicKey.toBuffer()],
        program.programId
      );

      vaultAuthority = vaultAuthorityPDA;

      // Create vault token account with vaultAuthority as signer and owner
      vaultTokenAccount = await token.createAssociatedTokenAccount(
        provider.connection,
        inputMint.publicKey,
        vaultAuthority, // Use vaultAuthority as owner
      );

    } catch (error) {
      console.error("Setup error:", error);
      throw error;
    }
  });

  it("should create an offer successfully", async () => {
    // Generate unique offer ID
    const [offerPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"),
        maker.publicKey.toBuffer(),
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
        inputMint.publicKey,
        outputMint.publicKey,
        new anchor.BN('1000'),
        new anchor.BN('500'),
        new anchor.BN('1733387146'),
        new anchor.BN('200'),
        new PublicKey('2vBAnVajtqmP4RBm8Vw5gzYEy3XCT9Mf1NBeQ2TPkiVF')
      )
      .accounts({
        maker: maker.publicKey,
        inputTokenMint: inputMint.publicKey,
        outputTokenMint: outputMint.publicKey,
        makerTokenAccount: makerTokenAccount,
        vaultTokenAccount: vaultTokenAccount,
      })
      .signers([maker])
      .rpc();

    // Verify vault token account balance
    const vaultBalance = await token.getAccount(
      provider.connection, 
      vaultTokenAccount
    );

    expect(vaultBalance.amount).to.equal(BigInt(500000000)); // Expecting 500 tokens in the vault
  });
});
