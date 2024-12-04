import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Swap } from "../target/types/swap";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { Account, createMint, getAssociatedTokenAddress, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

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
  const mint_c = Keypair.generate();

  let adminConfig: PublicKey;
  let feeConfig: PublicKey;
  let whitelistConfig: PublicKey;
  let mintWhitelist: PublicKey;
  let offer:PublicKey;
  let makerTokenAccount:Account;
  let vaultTokenAccount:Account;
  let vaultAuthority:PublicKey;
  let whitelist:PublicKey;
  let makerRecieveTokenAccount:PublicKey;
  let takerPaymentTokenAccount:PublicKey;
  let takerReceiveTokenAccount:PublicKey;
  let feeTokenAccount:PublicKey;

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
        { commitment: 'confirmed' },
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
        { commitment: 'confirmed' },
        TOKEN_PROGRAM_ID
      )

      makerTokenAccount = inputMintMakerATA;

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
        mint_b,
        { commitment: 'confirmed' },
        TOKEN_PROGRAM_ID
      )
      console.log("OutputMint:", outputMint.toBase58())

      const outputMintTakerATA = await getOrCreateAssociatedTokenAccount(
        connection,
        taker,
        outputMint,
        taker.publicKey,
        false,
        'confirmed',
        { commitment: 'confirmed' },
        TOKEN_PROGRAM_ID
      )

      console.log("outputMint ATA for Taker Created:", outputMintTakerATA.address.toBase58())


      //minting tokens mint_b to Taker

      const mintToTakerTx = await mintTo(
        connection,
        taker,
        outputMint,
        outputMintTakerATA.address,
        admin,
        2000_000000000,
      )
      console.log("Token minted to Taker ATA:", mintToTakerTx)


    } catch (error) {
      console.error("Setup error:", error);
      throw error;
    }
  });

  it("initialize admin", async () => {

    adminConfig = PublicKey.findProgramAddressSync(
      [Buffer.from('admin_config')],
      program.programId
    )[0];

    feeConfig = PublicKey.findProgramAddressSync(
      [Buffer.from('fee')],
      program.programId
    )[0];

    whitelistConfig = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist_config')],
      program.programId
    )[0];

    mintWhitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('mint_whitelist')],
      program.programId
    )[0];

    offer = PublicKey.findProgramAddressSync(
      [Buffer.from('offer'),maker.publicKey.toBuffer()],
      program.programId
    )[0];


    program.methods.initializeAdmin(
      new anchor.BN('200'),
      new PublicKey('2vBAnVajtqmP4RBm8Vw5gzYEy3XCT9Mf1NBeQ2TPkiVF'),
      false,
      [mint_a.publicKey, mint_b.publicKey]
    ).accounts({
      admin: admin.publicKey,
      adminConfig: adminConfig,
      feeConfig: feeConfig,
      whitelistConfig: whitelistConfig,
      mintWhitelist: mintWhitelist,
      systemProgram: SystemProgram.programId
    }
    ).signers(
      [admin]
    ).rpc()
  });

  it("add token-mints to whitelist", async () => {
     program.methods.addMintsToWhitelist(
      [mint_a.publicKey,mint_b.publicKey,mint_c.publicKey]
     ).accounts({
      admin: admin.publicKey,
      adminConfig: adminConfig,
      mintWhitelist: mintWhitelist,
      systemProgram: SystemProgram.programId
     }).signers(
      [admin]
     ).rpc()
  })

  it("removes token-mints from whitelist", async ()=>{
    
    program.methods.removeMintsFromWhitelist(
      [mint_c.publicKey]
     ).accounts({
      admin: admin.publicKey,
      adminConfig: adminConfig,
      mintWhitelist: mintWhitelist,
      systemProgram: SystemProgram.programId
     }).signers(
      [admin]
     ).rpc()
  })

  it("updates fee percentage", async () => {
    program.methods.updateFeePercentage(
      new anchor.BN('300')
    ).accounts({
      admin: admin.publicKey,
      adminConfig: adminConfig,
      feeConfig: feeConfig,
      systemProgram: SystemProgram.programId
    }).signers(
      [admin]
    ).rpc()
  })
  it('updates fee address', async () => {
    program.methods.updateFeeAddress(
      new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv')
    ).accounts({
      admin: admin.publicKey,
      adminConfig: adminConfig,
      feeConfig: feeConfig,
      systemProgram: SystemProgram.programId
    }).signers(
      [admin]
    ).rpc()
  })

  it("Toggles check for token whitelist", async()=> {
      program.methods.toggleRequireWhitelist(
      ).accounts({
        admin: admin.publicKey,
        adminConfig: adminConfig,
        whitelistConfig: whitelistConfig,
        systemProgram: SystemProgram.programId,
      }).signers(
        [admin]
      ).rpc()
  })

  it("Check for offers with no-takers, and update state to Expired after deadline", async()=>{
    program.methods.expireOffer()
    .accounts({
      admin: admin.publicKey,
      adminConfig: adminConfig,
      offer: offer,
      systemProgram: SystemProgram.programId
    }).signers(
      [admin]
    ).rpc()
  })
  it("creates offer and send tokens to vault", async()=>{
    
    vaultTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      maker,
      mint_a.publicKey,
      offer,
      true,
      'confirmed',
      { commitment: 'confirmed' },
      TOKEN_PROGRAM_ID
    )

    program.methods.createOfferAndSendTokensToVault(
      new anchor.BN('500'), //amout of token_a for sale
      new anchor.BN('250'), //total amount of token_b expected
      new anchor.BN('1733515556') //deadline as unix timestamp 
    ).accounts({
      maker: maker.publicKey,
      offer: offer,
      adminConfig: adminConfig,
      feeConfig: feeConfig,
      makerTokenAccount:makerTokenAccount.address,
      vaultTokenAccount:vaultTokenAccount.address,
      inputTokenMint:mint_a.publicKey, // Defining input token mint here
      outputTokenMint:mint_b.publicKey, // Defining output token mint here
      tokenProgram: TOKEN_PROGRAM_ID, // It can accept both TOKEN_PROGRAM & TOKEN_22 PROGRAM, proper input expected from fron-end
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram:SystemProgram.programId
    }).signers(
      [maker]
    ).rpc()
  })
  it("add taker addresses to whitelist", async() =>{

    whitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), maker.publicKey.toBuffer()],
      program.programId
    )[0];

    program.methods.addTakerWhitelist(
      [taker.publicKey,new PublicKey('4sijvjMmXG8sdzwsLJevQ1dSXpY6a9T7fPRkVLeqr6Wm'), new PublicKey('7ZK3Y3izGJDGEd2CAvXpotCZinKuYk71YABoHxBEdo4G')]
    ).accounts({
      maker:maker.publicKey,
      whitelist:whitelist,
      offer:offer,
      systemProgram: SystemProgram.programId
    }).signers(
      [maker]
    ).rpc()
  })
  it("remove takers from whitelist", async () => {
    program.methods.removeTakerWhitelist(
      [new PublicKey('7ZK3Y3izGJDGEd2CAvXpotCZinKuYk71YABoHxBEdo4G')]
    ).accounts({
      maker:maker.publicKey,
      whitelist:whitelist,
      offer:offer,
      systemProgram: SystemProgram.programId
    }).signers(
      [maker]
    ).rpc()
  })
  it("maker cancels the offer and close vault&whitelist PDAs", async() =>{
    vaultAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from('vault'),mint_a.publicKey.toBuffer()],
      program.programId
    )[0];

    program.methods.cancelOffer()
    .accounts({
      maker:maker.publicKey,
      offer:offer,
      whitelist:whitelist,
      feeConfig: feeConfig,
      makerTokenAccount: makerTokenAccount.address,
      vaultTokenAccount:vaultTokenAccount.address,
      vaultAuthority:vaultAuthority,
      tokenProgram:TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    }).signers(
      [maker]
    ).rpc()
  })
  it("taker partially accepts an offer and swap tokens", async () => {

    makerRecieveTokenAccount = await getAssociatedTokenAddress(mint_b.publicKey,maker.publicKey);
    takerPaymentTokenAccount = await getAssociatedTokenAddress(mint_b.publicKey,taker.publicKey);
    takerReceiveTokenAccount = await getAssociatedTokenAddress(mint_a.publicKey,taker.publicKey);
    feeTokenAccount = await getAssociatedTokenAddress(mint_b.publicKey,new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv'));


    program.methods.takeOffer(
      new anchor.BN('100')
    ).accounts({
      taker:taker.publicKey,
      offer:offer,
      maker:maker.publicKey,
      whitelist: whitelist,
      makerRecieveTokenAccount: makerRecieveTokenAccount,
      takerPaymentTokenAccount: takerPaymentTokenAccount,
      takerReceiveTokenAccount:takerReceiveTokenAccount,
      feeTokenAccount:feeTokenAccount,
      vaultTokenAccount:vaultTokenAccount.address,
      inputTokenMint: mint_a.publicKey,
      outputTokenMint: mint_b.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers(
      [taker]
    ).rpc()
  })
  it("taker full accepts an offer and takes remaining tokens in vault", async () => {
    program.methods.takeOffer(
      new anchor.BN('150')
    ).accounts({
      taker:taker.publicKey,
      offer:offer,
      maker:maker.publicKey,
      whitelist: whitelist,
      makerRecieveTokenAccount: makerRecieveTokenAccount,
      takerPaymentTokenAccount: takerPaymentTokenAccount,
      takerReceiveTokenAccount:takerReceiveTokenAccount,
      feeTokenAccount:feeTokenAccount,
      vaultTokenAccount:vaultTokenAccount.address,
      inputTokenMint: mint_a.publicKey,
      outputTokenMint: mint_b.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).signers(
      [taker]
    ).rpc()
  })
});
