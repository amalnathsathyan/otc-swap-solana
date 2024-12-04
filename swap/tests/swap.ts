import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Swap } from "../target/types/swap";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import { expect } from "chai";

describe("swap program - create offer", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;

  const program = anchor.workspace.Swap as Program<Swap>;
  const maker = Keypair.generate();
  const admin = Keypair.generate()
  const taker = Keypair.generate()
  const mint_a = Keypair.generate();
  const mint_b = Keypair.generate();
  
  before(async () => {
    try {

      console.log("maker:", maker.publicKey.toBase58())
      // Airdrop SOL to maker and taker
      const airdropAmount = anchor.web3.LAMPORTS_PER_SOL * 5;
      const makerAirdropTx = await connection.requestAirdrop(maker.publicKey, airdropAmount);
      const takerAirdropTx = await connection.requestAirdrop(taker.publicKey, airdropAmount);
      const adminAirdropTx = await connection.requestAirdrop(taker.publicKey, airdropAmount);

      const latestBlockhash = await connection.getLatestBlockhash();

      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: makerAirdropTx,
      });

      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: takerAirdropTx,
      });

      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: adminAirdropTx,
      });

      console.log("Airdrop Successful with hash", makerAirdropTx)
      console.log("Airdrop Successful with hash", takerAirdropTx)
      console.log("Airdrop Successful with hash", adminAirdropTx)

      const inputMint = await createMint(
        connection,
        maker,
        admin.publicKey,
        admin.publicKey,
        9,
        mint_a,
        {commitment:'confirmed'},
        TOKEN_PROGRAM_ID
      )
      console.log("InputMint:", inputMint.toBase58())

      //creating ATA
      const inputMintMakerATA = await getOrCreateAssociatedTokenAccount(
        connection,
        maker,
        inputMint,
        maker.publicKey,
        false,
        'confirmed',
        {commitment:'confirmed'},
        TOKEN_PROGRAM_ID
      )

      console.log("inputMint ATA for Maker Created:", inputMintMakerATA.address.toBase58())

      //minting tokens mint_a to Maker

      const mintToMakerTx = await mintTo(
        connection,
        maker,
        inputMint,
        inputMintMakerATA.address,
        admin,
        1000_000000000,
      )

      console.log("Token minted to maker ATA:", mintToMakerTx)

      const outputMint = await createMint(
        connection,
        taker,
        admin.publicKey,
        admin.publicKey,
        9,
      )
      console.log("OutputMint:", outputMint.toBase58())

      const outputMintTakerATA = await getOrCreateAssociatedTokenAccount(
        connection,
        taker,
        inputMint,
        taker.publicKey,
        false,
        'confirmed',
        {commitment:'confirmed'},
        TOKEN_PROGRAM_ID
      )

      console.log("outputMint ATA for Taker Created:", outputMintTakerATA.address.toBase58())

      //minting tokens mint_a to Maker
      const mintToTakerTx = await mintTo(
        connection,
        maker,
        inputMint,
        inputMintMakerATA.address,
        admin,
        2000_000000000,
      )
      console.log("Token minted to maker ATA:", mintToTakerTx)


    } catch (error) {
      console.error("Setup error:", error);
      throw error;
    }
  });

  it("initialize admin", async () => {
      program.methods.initializeAdmin(
        new anchor.BN('200'),

      )
  });
  it("takes offer")
});
