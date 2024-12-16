import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Swap } from "../target/types/swap";
import { Keypair, PublicKey, SystemProgram, AccountInfo, sendAndConfirmTransaction } from "@solana/web3.js";
import { Account, ASSOCIATED_TOKEN_PROGRAM_ID, createMint, getAccount, getAssociatedTokenAddress, getAssociatedTokenAddressSync, getMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert, expect } from "chai";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

function isAnchorError(error: any): error is { error: any; errorLogs: string[] } {
  return error && typeof error === "object" && "error" in error && "errorLogs" in error;
}

describe("swap program - create offer", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;

  const program = anchor.workspace.Swap as Program<Swap>;
  const maker = Keypair.generate();
  const maker2 = Keypair.generate();
  const admin = Keypair.generate()
  const taker = Keypair.generate()
  const mint_a = Keypair.generate();
  const mint_b = Keypair.generate();
  const mint_c = Keypair.generate();

  let adminConfig: PublicKey;
  let feeConfig: PublicKey;
  let whitelistConfig: PublicKey;
  let mintWhitelist: PublicKey;
  let offer: PublicKey;
  let makerTokenAccount: Account;
  let maker2TokenAccount: Account;
  let vaultTokenAccount: PublicKey;
  let vaultAuthority: PublicKey;
  let whitelist: PublicKey;
  let makerRecieveTokenAccount: PublicKey;
  let takerPaymentTokenAccount: PublicKey;
  let takerReceiveTokenAccount: PublicKey;
  let feeTokenAccount: PublicKey;
  const offerId = 356756;


  before(async () => {
    try {

      console.log("maker:", maker.publicKey.toBase58())
      console.log("maker2:", maker2.publicKey.toBase58())
      console.log("taker:", taker.publicKey.toBase58())
      console.log("admin:", admin.publicKey.toBase58())

      // Airdrop SOL to maker and taker
      const airdropAmount = anchor.web3.LAMPORTS_PER_SOL * 5;
      const makerAirdropTx = await connection.requestAirdrop(maker.publicKey, airdropAmount);
      const maker2AirdropTx = await connection.requestAirdrop(maker2.publicKey, airdropAmount);
      const takerAirdropTx = await connection.requestAirdrop(taker.publicKey, airdropAmount);
      const adminAirdropTx = await connection.requestAirdrop(admin.publicKey, airdropAmount);

      const latestBlockhash = await connection.getLatestBlockhash();

      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: makerAirdropTx,
      }, 'confirmed');

      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: takerAirdropTx,
      }, 'confirmed');

      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: adminAirdropTx,
      }, 'confirmed');
      await connection.confirmTransaction({
        blockhash: latestBlockhash.blockhash,
        lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
        signature: maker2AirdropTx,
      }, 'confirmed');

      // const tx = await connection.confirmTransaction(adminAirdropTx,'confirmed');

      console.log("Maker Airdrop Successful with hash", makerAirdropTx)
      console.log("Maker2 Airdrop Successful with hash", maker2AirdropTx)
      console.log("Taker Airdrop Successful with hash", takerAirdropTx)
      console.log("Admin Airdrop Successful with hash", adminAirdropTx)

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
        10000_000000000,
      )

      console.log("Token minted to maker ATA:", mintToMakerTx)

      //creating ATA
      const inputMintMaker2ATA = await getOrCreateAssociatedTokenAccount(
        connection,
        maker2,
        inputMint,
        maker2.publicKey,
        false,
        'confirmed',
        { commitment: 'confirmed' },
        TOKEN_PROGRAM_ID
      )

      maker2TokenAccount = inputMintMaker2ATA;

      console.log("inputMint ATA for Maker Created:", inputMintMaker2ATA.address.toBase58())

      //minting tokens mint_a to Maker

      const mintToMaker2Tx = await mintTo(
        connection,
        maker2,
        inputMint,
        inputMintMaker2ATA.address,
        admin,
        10000_000000000,
      )

      console.log("Token minted to maker2 ATA:", mintToMaker2Tx)

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
        20000_000000000,
        undefined,
        { commitment: 'confirmed' }
      )
      console.log("Token minted to Taker ATA:", mintToTakerTx)


    } catch (error) {
      console.error("Setup error:", error);
      throw error;
    }
  });

  it("initialize admin", async () => {
    // Find PDA addresses
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

    // Print addresses for debugging
    console.log('adminConfig:', adminConfig.toBase58());
    console.log('feeConfig:', feeConfig.toBase58())
    console.log('whitelistConfig:', whitelistConfig.toBase58())
    console.log('mintWhitelist:', mintWhitelist.toBase58())

    // Call the initializeAdmin function
    const feePercentage = new anchor.BN('200'); // 2.00%
    const feeWallet = new PublicKey('2vBAnVajtqmP4RBm8Vw5gzYEy3XCT9Mf1NBeQ2TPkiVF');
    const requireWhitelist = true;
    const initialMints = [mint_a.publicKey, mint_b.publicKey];

    await program.methods
      .initializeAdmin(feePercentage, feeWallet, requireWhitelist, initialMints)
      .accounts({
        admin: admin.publicKey,
      })
      .signers([admin])
      .rpc();

    // Fetch and assert adminConfig state
    const adminConfigFetched = await program.account.adminConfig.fetch(adminConfig);
    assert(adminConfigFetched.admin.equals(admin.publicKey), "Admin public key mismatch");

    // Fetch and assert feeConfig state
    const feeConfigFetched = await program.account.feeConfig.fetch(feeConfig);
    assert(feeConfigFetched.feeAddress.equals(feeWallet), "Fee wallet mismatch");
    assert(feeConfigFetched.feePercentage.eq(feePercentage), "Fee percentage mismatch");

    // Fetch and assert whitelistConfig state
    const whitelistConfigFetched = await program.account.whitelistConfig.fetch(whitelistConfig);
    assert(whitelistConfigFetched.requireWhitelist === requireWhitelist, "Whitelist requirement mismatch");

    // Fetch and assert mintWhitelist state
    const mintWhitelistFetched = await program.account.mintWhitelist.fetch(mintWhitelist);
    assert.deepEqual(
      mintWhitelistFetched.mints.map(m => m.toBase58()),
      initialMints.map(m => m.toBase58()),
      "Initial mint whitelist mismatch"
    );

    // Check bumps and seeds
    const [expectedAdminConfig, adminConfigBump] = PublicKey.findProgramAddressSync(
      [Buffer.from('admin_config')],
      program.programId
    );
    const [expectedFeeConfig, feeConfigBump] = PublicKey.findProgramAddressSync(
      [Buffer.from('fee')],
      program.programId
    );
    assert(adminConfig.equals(expectedAdminConfig), "AdminConfig PDA mismatch");
    assert(feeConfig.equals(expectedFeeConfig), "FeeConfig PDA mismatch");

    // Check edge cases
    expect(initialMints.length).lessThanOrEqual(50); // Maximum 50 mints allowed
    expect(feePercentage.toNumber()).lessThanOrEqual(10000); // Fee percentage must not exceed 100%

    console.log("All assertions passed for initializeAdmin");
  })

  it("add token-mints to whitelist", async () => {
    const new_mints = [mint_a.publicKey, mint_b.publicKey, mint_c.publicKey];
    await program.methods.addMintsToWhitelist(
      [mint_c.publicKey]
    ).accounts({
      admin: admin.publicKey,
    }).signers(
      [admin]
    ).rpc()

    // Fetch and assert mintWhitelist state
    const mintWhitelistFetched = await program.account.mintWhitelist.fetch(mintWhitelist);
    assert.deepEqual(
      mintWhitelistFetched.mints.map(m => m.toBase58()),
      new_mints.map(m => m.toBase58()),
      "Add mint whitelist mismatch"
    );
    console.log("All assertions passed for add token-mints");
  })

  it("removes token-mints from whitelist", async () => {
    const mints = [mint_a.publicKey, mint_b.publicKey];

    await program.methods.removeMintsFromWhitelist(
      [mint_c.publicKey]
    ).accounts({
      admin: admin.publicKey,
    }).signers(
      [admin]
    ).rpc()

    // Fetch and assert mintWhitelist state
    const mintWhitelistFetched = await program.account.mintWhitelist.fetch(mintWhitelist);
    assert.deepEqual(
      mintWhitelistFetched.mints.map(m => m.toBase58()),
      mints.map(m => m.toBase58()),
      "Remove mint whitelist mismatch"
    );
    console.log("All assertions passed for remove token-mints");
  })

  it("Fail: initialize admin when it is already initialized", async () => {
    // Find PDA addresses
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

    // Call the initializeAdmin function
    const feePercentage = new anchor.BN('200'); // 2.00%
    const feeWallet = new PublicKey('2vBAnVajtqmP4RBm8Vw5gzYEy3XCT9Mf1NBeQ2TPkiVF');
    const requireWhitelist = true;
    const initialMints = [mint_a.publicKey, mint_b.publicKey];

    try {
      await program.methods
        .initializeAdmin(feePercentage, feeWallet, requireWhitelist, initialMints)
        .accounts({
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      console.log("Error (initialize admin when it is already initialized)");
    }
  })

  it("Fail: add mint called by non-admin user", async () => {
    try {
      await program.methods.addMintsToWhitelist(
        [mint_c.publicKey]
      ).accounts({
        admin: maker.publicKey,
      }).signers(
        [maker]
      ).rpc()
    } catch (err) {
      if (isAnchorError(err)) {
        // Check if the error is the one you expect
        assert.strictEqual(err.error.errorCode.code, 'UnauthorizedAdmin');
        assert.strictEqual(err.error.errorMessage, 'Unauthorized admin');
        console.log("Assertion passed: add mint called by non-admin user");
      } else {
        throw err; // Rethrow if it's not an Anchor error
      }
    }
  })

  it("Fail: remove mint called by non-admin user", async () => {
    try {
      await program.methods.removeMintsFromWhitelist(
        [mint_c.publicKey]
      ).accounts({
        admin: maker.publicKey,
      }).signers(
        [maker]
      ).rpc()
    } catch (err) {
      if (isAnchorError(err)) {
        // Check if the error is the one you expect
        assert.strictEqual(err.error.errorCode.code, 'UnauthorizedAdmin');
        assert.strictEqual(err.error.errorMessage, 'Unauthorized admin');
        console.log("Assertion passed: remove mint called by non-admin user");
      } else {
        throw err; // Rethrow if it's not an Anchor error
      }
    }
  })

  it("updates fee percentage", async () => {
    const feePercentage = new anchor.BN('300'); // 3.00%
    const feeWallet = new PublicKey('2vBAnVajtqmP4RBm8Vw5gzYEy3XCT9Mf1NBeQ2TPkiVF');

    await program.methods.updateFeePercentage(
      new anchor.BN('300')
    ).accounts({
      admin: admin.publicKey,
    }).signers(
      [admin]
    ).rpc()

    // Fetch and assert feeConfig state
    const feeConfigFetched = await program.account.feeConfig.fetch(feeConfig);
    assert(feeConfigFetched.feeAddress.equals(feeWallet), "Fee wallet mismatch");
    assert(feeConfigFetched.feePercentage.eq(feePercentage), "Fee percentage mismatch");
    console.log("All assertions passed for updates fee percentage");
  })

  it("Fail: updated fee percentage is more than 100%", async () => {
    try {
      await program.methods.updateFeePercentage(
        new anchor.BN('10001')
      ).accounts({
        admin: admin.publicKey,
      }).signers(
        [admin]
      ).rpc()
    } catch (err) {
      if (isAnchorError(err)) {
        // Check if the error is the one you expect
        assert.strictEqual(err.error.errorCode.code, 'InvalidFeePercentage');
        assert.strictEqual(err.error.errorMessage, 'Fee percentage cannot exceed 100%');
        console.log("Assertion passed: updated fee percentage is more than 100%");
      } else {
        throw err; // Rethrow if it's not an Anchor error
      }
    }
  })

  it("Fail: updates fee percentage by non-admin user", async () => {
    try {
      await program.methods.updateFeePercentage(
        new anchor.BN('10001')
      ).accounts({
        admin: maker.publicKey,
      }).signers(
        [maker]
      ).rpc()
    } catch (err) {
      if (isAnchorError(err)) {
        // Check if the error is the one you expect
        assert.strictEqual(err.error.errorCode.code, 'UnauthorizedAdmin');
        assert.strictEqual(err.error.errorMessage, 'Unauthorized admin');
        console.log("Assertion passed: updates fee percentage by non-admin user");
      } else {
        throw err; // Rethrow if it's not an Anchor error
      }
    }
  })

  it('updates fee address', async () => {
    const newFeeWallet = new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv');
    await program.methods.updateFeeAddress(
      new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv')
    ).accounts({
      admin: admin.publicKey,
    }).signers(
      [admin]
    ).rpc()

    // Fetch and assert feeConfig state
    const feeConfigFetched = await program.account.feeConfig.fetch(feeConfig);
    assert(feeConfigFetched.feeAddress.equals(newFeeWallet), "Fee wallet mismatch");
    console.log("All assertions passed for update fee address");
  })

  it('Fail: updates fee address by non admin user', async () => {
    try {
      await program.methods.updateFeeAddress(
        new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv')
      ).accounts({
        admin: maker.publicKey,
      }).signers(
        [maker]
      ).rpc()
    } catch (err) {
      if (isAnchorError(err)) {
        // Check if the error is the one you expect
        assert.strictEqual(err.error.errorCode.code, 'UnauthorizedAdmin');
        assert.strictEqual(err.error.errorMessage, 'Unauthorized admin');
        console.log("Assertion passed: updates fee address by non admin user");
      } else {
        throw err; // Rethrow if it's not an Anchor error
      }
    }
  })

  it("Toggles check for token whitelist", async () => {
    const whitelistConfigFetchedPreCall = await program.account.whitelistConfig.fetch(whitelistConfig);
    const requireWhitelist = whitelistConfigFetchedPreCall.requireWhitelist;
    await program.methods.toggleRequireWhitelist(
    ).accounts({
      admin: admin.publicKey,
    }).signers(
      [admin]
    ).rpc()

    // Fetch and assert whitelistConfig state
    const whitelistConfigFetched = await program.account.whitelistConfig.fetch(whitelistConfig);
    assert(whitelistConfigFetched.requireWhitelist === !requireWhitelist, "Whitelist requirement mismatch");
    console.log("All assertions passed for Toggles check for token whitelist");
  })

  it("Fail: Toggles check for token whitelist by non-admin user", async () => {
    try {
      await program.methods.toggleRequireWhitelist(
      ).accounts({
        admin: maker.publicKey,
      }).signers(
        [maker]
      ).rpc()
    } catch (err) {
      if (isAnchorError(err)) {
        // Check if the error is the one you expect
        assert.strictEqual(err.error.errorCode.code, 'UnauthorizedAdmin');
        assert.strictEqual(err.error.errorMessage, 'Unauthorized admin');
        console.log("Assertion passed: Toggles check for token whitelist by non-admin user");
      } else {
        throw err; // Rethrow if it's not an Anchor error
      }
    }
  })

  it("creates an offer and sends tokens to the vault", async () => {
    // Unique identifier for the offer
    const amountTokenAForSale = new anchor.BN('500'); // Amount of Token A for sale
    const totalTokenBExpected = new anchor.BN('250'); // Total amount of Token B expected
    const deadlineUnixTimestamp = new anchor.BN('1734363981'); // Deadline as a Unix timestamp

    try {
      // Derive the PDA for the offer
      offer = PublicKey.findProgramAddressSync(
        [Buffer.from('offer'), maker.publicKey.toBuffer(), Buffer.from(new anchor.BN(offerId).toArray('le', 8))],
        program.programId
      )[0];

      // Create or fetch the associated token account for the vault

      vaultTokenAccount = await getAssociatedTokenAddressSync(
        mint_a.publicKey,
        offer,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      )

      console.log('Vault token account found or created:', vaultTokenAccount.toBase58());

      // Fetch maker's token balance before the transaction
      const makerBalanceBefore = await connection.getTokenAccountBalance(makerTokenAccount.address, 'confirmed');
      console.log('makerBalanceBefore:', Number(makerBalanceBefore.value.amount))
      // const vaultBalanceBefore = await connection.getTokenAccountBalance(vaultTokenAccount,'confirmed');
      // console.log('vaultBalanceBefore:', Number(vaultBalanceBefore.value.amount))

      // Call the program method to create the offer and send tokens to the vault
      await program.methods
        .createOfferAndSendTokensToVault(
          new anchor.BN(offerId), // Offer ID
          amountTokenAForSale, // Amount of Token A for sale
          totalTokenBExpected, // Total amount of Token B expected
          deadlineUnixTimestamp // Offer deadline
        )
        .accounts({
          maker: maker.publicKey, // Offer creator's public key
          makerTokenAccount: makerTokenAccount.address, // Maker's token account
          inputTokenMint: mint_a.publicKey, // Mint of input token (Token A)
          outputTokenMint: mint_b.publicKey, // Mint of output token (Token B)
          tokenProgram: TOKEN_PROGRAM_ID, // Token program ID
        })
        .signers([maker]) // Sign with the maker's key
        .rpc();

      console.log('Offer created and tokens sent to vault successfully!');

      // Fetch balances after the transaction
      const makerBalanceAfter = await connection.getTokenAccountBalance(makerTokenAccount.address);
      console.log('makerBalanceAfter:', Number(makerBalanceAfter.value.amount))
      const vaultBalanceAfter = await connection.getTokenAccountBalance(vaultTokenAccount);
      console.log('vaultBalanceAfter:', Number(vaultBalanceAfter.value.amount))

      // Assert balances
      assert.equal(
        Number(makerBalanceAfter.value.amount),
        Number(makerBalanceBefore.value.amount) - 500,
        "Maker's token balance should decrease by 500."
      );

      assert.equal(
        Number(vaultBalanceAfter.value.amount), 500,
        "Vault's token balance should increase by 500."
      );

      // Fetch offer account and assert fields
      const offerAccount = await program.account.offer.fetch(offer);
      assert.equal(offerAccount.offerId.toString(), offerId.toString(), 'Offer ID should match.');
      assert.equal(
        offerAccount.inputTokenMint.toString(),
        mint_a.publicKey.toString(),
        'Token A amount for sale should match.'
      );
      assert.equal(
        offerAccount.outputTokenMint.toString(),
        mint_b.publicKey.toString(),
        'Total Token B expected should match.'
      );
      assert.equal(
        offerAccount.deadline.toString(),
        deadlineUnixTimestamp.toString(),
        'Deadline should match the provided timestamp.'
      );

      console.log('All assertions passed successfully.');
    } catch (error) {
      console.error('Error creating offer and sending tokens to vault:', error);
    }
  });

  it("same maker creates multiple offers with same tokens", async() =>{
    // Unique identifier for the offer
    const amountTokenAForSaleOffer1 = new anchor.BN('500'); // Amount of Token A for sale
    const amountTokenAForSaleOffer2 = new anchor.BN('800'); // Amount of Token A for sale
    const totalTokenBExpectedOffer1 = new anchor.BN('250'); // Total amount of Token B expected
    const totalTokenBExpectedOffer2 = new anchor.BN('350'); // Total amount of Token B expected
    const deadlineUnixTimestampOffer1 = new anchor.BN('1734280633'); // Deadline as a Unix timestamp
    const deadlineUnixTimestampOffer2 = new anchor.BN('1734280676'); // Deadline as a Unix timestamp
    const offer1Id = '2345'
    const offer2Id = '2346'

    //offer1
    try {
      // Derive the PDA for the offer
      const offer1 = PublicKey.findProgramAddressSync(
        [Buffer.from('offer'), maker2.publicKey.toBuffer(), Buffer.from(new anchor.BN(offer1Id).toArray('le', 8))],
        program.programId
      )[0];

      // Create or fetch the associated token account for the vault

      vaultTokenAccount = await getAssociatedTokenAddressSync(
        mint_a.publicKey,
        offer1,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      )

      console.log('Vault token account found or created:', vaultTokenAccount.toBase58());

      // Fetch maker's token balance before the transaction
      const makerBalanceBefore = await connection.getTokenAccountBalance(maker2TokenAccount.address, 'confirmed');
      console.log('makerBalanceBefore:', Number(makerBalanceBefore.value.amount))
      // const vaultBalanceBefore = await connection.getTokenAccountBalance(vaultTokenAccount,'confirmed');
      // console.log('vaultBalanceBefore:', Number(vaultBalanceBefore.value.amount))

      // Call the program method to create the offer and send tokens to the vault
      const offer1Tx = await program.methods
        .createOfferAndSendTokensToVault(
          new anchor.BN(offer1Id), // Offer ID
          amountTokenAForSaleOffer1, // Amount of Token A for sale
          totalTokenBExpectedOffer1, // Total amount of Token B expected
          deadlineUnixTimestampOffer1 // Offer deadline
        )
        .accounts({
          maker: maker2.publicKey, // Offer creator's public key
          makerTokenAccount: maker2TokenAccount.address, // Maker's token account
          inputTokenMint: mint_a.publicKey, // Mint of input token (Token A)
          outputTokenMint: mint_b.publicKey, // Mint of output token (Token B)
          tokenProgram: TOKEN_PROGRAM_ID, // Token program ID
        }).transaction();
       // Sign with the maker's key
     
      const offer1TxSig = await sendAndConfirmTransaction(connection, offer1Tx,[maker2],{commitment:'confirmed'})

      console.log('Offer1 created and tokens sent to vault successfully!', offer1TxSig);

      // Fetch balances after the transaction
      const makerBalanceAfter = await connection.getTokenAccountBalance(maker2TokenAccount.address);
      console.log('makerBalanceAfter:', Number(makerBalanceAfter.value.amount))
      const vaultBalanceAfter = await connection.getTokenAccountBalance(vaultTokenAccount);
      console.log('vaultBalanceAfter:', Number(vaultBalanceAfter.value.amount))

      // Assert balances
      assert.equal(
        Number(makerBalanceAfter.value.amount),
        Number(makerBalanceBefore.value.amount) - 500,
        "Maker's token balance should decrease by 500."
      );

      assert.equal(
        Number(vaultBalanceAfter.value.amount), 500,
        "Vault's token balance should increase by 500."
      );

      // Fetch offer account and assert fields
      const offerAccount = await program.account.offer.fetch(offer1);
      assert.equal(offerAccount.offerId.toString(), offer1Id.toString(), 'Offer ID should match.');
      assert.equal(
        offerAccount.inputTokenMint.toString(),
        mint_a.publicKey.toString(),
        'Token A amount for sale should match.'
      );
      assert.equal(
        offerAccount.outputTokenMint.toString(),
        mint_b.publicKey.toString(),
        'Total Token B expected should match.'
      );
      assert.equal(
        offerAccount.deadline.toString(),
        deadlineUnixTimestampOffer1.toString(),
        'Deadline should match the provided timestamp.'
      );

      console.log('All assertions for offer1 passed successfully.');
    } catch (error) {
      console.error('Error creating offer1 and sending tokens to vault:', error);
    }
    //offer2
    try {
      // Derive the PDA for the offer
      const offer2 = PublicKey.findProgramAddressSync(
        [Buffer.from('offer'), maker2.publicKey.toBuffer(), Buffer.from(new anchor.BN(offer2Id).toArray('le', 8))],
        program.programId
      )[0];

      // Create or fetch the associated token account for the vault

      vaultTokenAccount = await getAssociatedTokenAddressSync(
        mint_a.publicKey,
        offer2,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ) 

      console.log('Vault token account found or created:', vaultTokenAccount.toBase58());

      // Fetch maker's token balance before the transaction
      const maker2BalanceBefore = await connection.getTokenAccountBalance(maker2TokenAccount.address, 'confirmed');
      console.log('maker2BalanceBefore:', Number(maker2BalanceBefore.value.amount))
      // const vaultBalanceBefore = await connection.getTokenAccountBalance(vaultTokenAccount,'confirmed');
      // console.log('vaultBalanceBefore:', Number(vaultBalanceBefore.value.amount))

      // Call the program method to create the offer and send tokens to the vault
      const offer2Tx = await program.methods
        .createOfferAndSendTokensToVault(
          new anchor.BN(offer2Id), // Offer ID
          amountTokenAForSaleOffer2, // Amount of Token A for sale
          totalTokenBExpectedOffer2, // Total amount of Token B expected
          deadlineUnixTimestampOffer2 // Offer deadline
        )
        .accounts({
          maker: maker2.publicKey, // Offer creator's public key
          makerTokenAccount: maker2TokenAccount.address, // Maker's token account
          inputTokenMint: mint_a.publicKey, // Mint of input token (Token A)
          outputTokenMint: mint_b.publicKey, // Mint of output token (Token B)
          tokenProgram: TOKEN_PROGRAM_ID, // Token program ID
        }).transaction()

      const offer2TxSig = await sendAndConfirmTransaction(connection, offer2Tx,[maker2],{commitment:'confirmed'})
      console.log('Offer2 created and tokens sent to vault successfully!', offer2TxSig);

      // Fetch balances after the transaction
      const maker2BalanceAfter = await connection.getTokenAccountBalance(maker2TokenAccount.address);
      console.log('maker2BalanceAfter:', Number(maker2BalanceAfter.value.amount))
      const vaultBalanceAfter = await connection.getTokenAccountBalance(vaultTokenAccount);
      console.log('vaultBalanceAfter:', Number(vaultBalanceAfter.value.amount))

      // Assert balances
      assert.equal(
        Number(maker2BalanceAfter.value.amount),
        Number(maker2BalanceBefore.value.amount) - 800,
        "Maker's token balance should decrease by 800."
      );

      assert.equal(
        Number(vaultBalanceAfter.value.amount), 800,
        "Vault's token balance should increase by 800."
      );

      // Fetch offer account and assert fields
      const offerAccount = await program.account.offer.fetch(offer2);
      assert.equal(offerAccount.offerId.toString(), offer2Id.toString(), 'Offer ID should match.');
      assert.equal(
        offerAccount.inputTokenMint.toString(),
        mint_a.publicKey.toString(),
        'Token A amount for sale should match.'
      );
      assert.equal(
        offerAccount.outputTokenMint.toString(),
        mint_b.publicKey.toString(),
        'Total Token B expected should match.'
      );
      assert.equal(
        offerAccount.deadline.toString(),
        deadlineUnixTimestampOffer2.toString(),
        'Deadline should match the provided timestamp.'
      );

      console.log('All assertions for offer2 passed successfully.');
    } catch (error) {
      console.error('Error creating offer2 and sending tokens to vault:', error);
    }

  })


  it("add taker addresses to whitelist by maker of the offer", async () => {

    const whitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), maker.publicKey.toBuffer(), Buffer.from(new anchor.BN(offerId).toArray('le', 8))],
      program.programId
    )[0];

    await program.methods.manageWhitelist(
      [taker.publicKey, new PublicKey('4sijvjMmXG8sdzwsLJevQ1dSXpY6a9T7fPRkVLeqr6Wm'), new PublicKey('7ZK3Y3izGJDGEd2CAvXpotCZinKuYk71YABoHxBEdo4G')]
    ).accountsPartial({
      maker: maker.publicKey,
      whitelist: whitelist,
      offer: offer,
    }).signers(
      [maker]
    ).rpc()

    const takerWhitelistFetched = await program.account.whitelist.fetch(whitelist);
    const newWhitelist = [taker.publicKey, new PublicKey('4sijvjMmXG8sdzwsLJevQ1dSXpY6a9T7fPRkVLeqr6Wm'), new PublicKey('7ZK3Y3izGJDGEd2CAvXpotCZinKuYk71YABoHxBEdo4G')]
    assert.deepEqual(
      takerWhitelistFetched.takers.map(m => m.toBase58()),
      newWhitelist.map(m => m.toBase58()),
      "Add takerwhitelist whitelist mismatch"
    );
    console.log("All assertions passed for add-taker-whitelist");
  })

  it("Fail: add taker addresses to whitelist by wrong maker of the offer", async () => {

    const whitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), maker.publicKey.toBuffer(), Buffer.from(new anchor.BN(offerId).toArray('le', 8))],
      program.programId
    )[0];

    try {
      await program.methods.manageWhitelist(
        [taker.publicKey, new PublicKey('4sijvjMmXG8sdzwsLJevQ1dSXpY6a9T7fPRkVLeqr6Wm'), new PublicKey('7ZK3Y3izGJDGEd2CAvXpotCZinKuYk71YABoHxBEdo4G')]
      ).accountsPartial({
        maker: maker2.publicKey,
        whitelist: whitelist,
        offer: offer,
      }).signers(
        [maker2]
      ).rpc()
    } catch (err) {
      if (isAnchorError(err)) {
        // Check if the error is the one you expect
        assert.strictEqual(err.error.errorCode.code, 'ConstraintSeeds');
        console.log("Assertion passed: add mint called by non-maker user");
      } else {
        throw err; // Rethrow if it's not an Anchor error
      }
    }
  })


  it("remove takers from whitelist", async () => {

    const whitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), maker.publicKey.toBuffer(), Buffer.from(new anchor.BN(offerId).toArray('le', 8))],
      program.programId
    )[0];
    
    await program.methods.manageWhitelist(
      [taker.publicKey]
    ).accounts({
      maker: maker.publicKey,
      offer: offer,
    }).signers(
      [maker]
    ).rpc()

    const takerWhitelistFetched = await program.account.whitelist.fetch(whitelist);
    const newWhitelist = [taker.publicKey]
    assert.deepEqual(
      takerWhitelistFetched.takers.map(m => m.toBase58()),
      newWhitelist.map(m => m.toBase58()),
      "Add takerwhitelist whitelist mismatch"
    );
    console.log("All assertions passed for remove-taker-whitelist");
  })

  it("Fail: remove takers from whitelist by wrong maker", async () => {

    try {
      await program.methods.manageWhitelist(
        [new PublicKey('7ZK3Y3izGJDGEd2CAvXpotCZinKuYk71YABoHxBEdo4G')]
      ).accounts({
        maker: maker2.publicKey,
        offer: offer,
      }).signers(
        [maker2]
      ).rpc()
    } catch (err) {
      if (isAnchorError(err)) {
        // Check if the error is the one you expect
        assert.strictEqual(err.error.errorCode.code, 'UnauthorizedMaker');
        console.log("Assertion passed: add mint called by non-maker user");
      } else {
        throw err; // Rethrow if it's not an Anchor error
      }
    }
  })

  it("taker fully accepts an offer and swap tokens", async () => {
    // Derive the offer PDA to verify it
    const [offerPDA, bump] = PublicKey.findProgramAddressSync(
        [
            Buffer.from('offer'),
            maker.publicKey.toBuffer(),
            Buffer.from(new anchor.BN(offerId).toArray('le', 8))
        ],
        program.programId
    );
    console.log("Offer PDA:", offerPDA.toBase58());
    console.log("Actual offer account:", offer.toBase58());
    
    vaultTokenAccount = getAssociatedTokenAddressSync(
        mint_a.publicKey,
        offer,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
    );

    try {
        const vaultInfo = await connection.getAccountInfo(vaultTokenAccount);
        console.log("Vault account exists:", !!vaultInfo);
        if (vaultInfo) {
            console.log({vaultInfo})
            const vaultAccountInfo = await connection.getParsedAccountInfo(vaultTokenAccount);
            const vaultAccount = (vaultAccountInfo.value.data as any).parsed.info;
            console.log("Vault balance:", vaultAccount.tokenAmount.amount);
            console.log("Vault owner:", vaultAccount.owner);
        }

        const offerAccount = await program.account.offer.fetch(offer);
        console.log("Offer status:", offerAccount.status);
        console.log("Offer remaining amount:", offerAccount.tokenAmountRemaining.toString());
    } catch (err) {
        console.error("Error checking accounts:", err);
    }

    // Setup token accounts
    makerRecieveTokenAccount = await getAssociatedTokenAddress(
        mint_b.publicKey, 
        maker.publicKey,
        false
    );

    takerPaymentTokenAccount = await getAssociatedTokenAddress(
        mint_b.publicKey, 
        taker.publicKey,
        false
    );
    
    takerReceiveTokenAccount = await getAssociatedTokenAddress(
        mint_a.publicKey, 
        taker.publicKey,
        false
    );

    // Create taker receive account if it doesn't exist
    try {
        await getOrCreateAssociatedTokenAccount(
            connection,
            taker,
            mint_a.publicKey,
            taker.publicKey,
            false,
            'confirmed',
            { commitment: 'confirmed' },
            TOKEN_PROGRAM_ID
        );
    } catch (err) {
        console.log("Taker receive account may already exist");
    }
    
    feeTokenAccount = await getAssociatedTokenAddress(
        mint_b.publicKey, 
        new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv'),
        false
    );

    // Log initial balances
    const takerBalanceTokenBBefore = await connection.getTokenAccountBalance(
        takerPaymentTokenAccount,
        'confirmed'
    );
    console.log('takerBalanceTokenBBefore:', Number(takerBalanceTokenBBefore.value.amount));
    
    try {
        const takeOfferTx = await program.methods.takeOffer(
            new anchor.BN('500')
        ).accountsPartial({
            core: {
                taker: taker.publicKey,
                adminConfig: adminConfig,
                offer: offer,
                maker: maker.publicKey,
                whitelist: whitelist,
            },
            token: {
                makerReceiveTokenAccount: makerRecieveTokenAccount,
                takerPaymentTokenAccount: takerPaymentTokenAccount,
                takerReceiveTokenAccount: takerReceiveTokenAccount,
                feeTokenAccount: feeTokenAccount,
                vaultTokenAccount: vaultTokenAccount,
                inputTokenMint: mint_a.publicKey,
                outputTokenMint: mint_b.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
                feeWallet: new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv'),
                taker: taker.publicKey,
                maker: maker.publicKey,
                offer: offer,
            }
        })
        .signers([taker])
        .rpc({ skipPreflight: true });  // Skip preflight to get more detailed error

        console.log("Take offer transaction:", takeOfferTx);
    } catch (err) {
        console.error("Error details:", err);
        // Get the transaction logs
        if ('logs' in err) {
            console.log("Transaction logs:", err.logs);
        }
        throw err;
    }
});

it("maker cancels the offer and close vault&whitelist PDAs", async () => {
  try {
      // Check if offer exists and is not completed
      try {
          const offerAccount = await program.account.offer.fetch(offer);
          if (offerAccount.status.completed) {
              console.log("Offer already completed - skipping cancel test");
              return;
          }
      } catch (err) {
          if (err.toString().includes("Account does not exist")) {
              console.log("Offer already closed - skipping cancel test");
              return;
          }
          throw err;
      }

      const offerNow = PublicKey.findProgramAddressSync(
        [Buffer.from('offer'), maker.publicKey.toBuffer(), Buffer.from(new anchor.BN(offerId).toArray('le', 8))],
        program.programId
      )[0];
      // Get initial balances
      const makerATokenBalanceBefore = await connection.getTokenAccountBalance(makerTokenAccount.address, 'confirmed');
      console.log('makerATokenAccount', makerTokenAccount.address);
      console.log('makerATokenBalanceBefore', Number(makerATokenBalanceBefore.value.amount));

      const vaultTokenAccountNow = await getOrCreateAssociatedTokenAccount(connection,maker,mint_a.publicKey,offer,true,'confirmed',{},TOKEN_PROGRAM_ID,ASSOCIATED_PROGRAM_ID)
      console.log('vaultTokenAccount:',vaultTokenAccountNow.address.toBase58());
      
      // Get current offer status
      const offerStatusBefore = await program.account.offer.fetch(offer);
      console.log('offer:', offerNow.toBase58())
      console.log('offerStatusBefore:',offerStatusBefore.status);

           // Execute cancel transaction
      try {
        const cancelTx = await program.methods.cancelOffer()
          .accounts({
              maker: maker.publicKey,
              makerTokenAccount: makerTokenAccount.address,
              inputTokenMint: mint_a.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
          }).transaction()
      
      const cancelTxSig = await sendAndConfirmTransaction(connection,cancelTx,[maker],{commitment:'confirmed'});
      console.log(cancelTxSig);
      } catch (err) {
       
          throw err; 
      }
      // Confirm transaction
      console.log('Cancel transaction successful');

      // Check final balances
      const makerATokenBalanceAfter = await connection.getTokenAccountBalance(
          makerTokenAccount.address,
          'confirmed'
      );
      console.log('makerATokenBalanceAfter:', Number(makerATokenBalanceAfter.value.amount));

      // Verify tokens returned
      const remainingAmount = offerStatusBefore.tokenAmountRemaining;
      assert.equal(
          Number(makerATokenBalanceAfter.value.amount),
          Number(makerATokenBalanceBefore.value.amount) + Number(remainingAmount),
          `${remainingAmount} tokens should be moved back from vault`
      );

      // Verify accounts are closed
      try {
          const vaultStatusAfter = await connection.getAccountInfo(vaultTokenAccountNow.address);
          console.log('vaultstatus',vaultStatusAfter);
          assert.equal(null, vaultStatusAfter, "Vault should be closed");
      } catch (err) {
        throw err
      }

      try {
          const offerStatusAfter = await program.account.offer.fetch(offer);
          console.log("offerStatusAfter:",offerStatusAfter.status)
          assert(offerStatusAfter.status, "Offer should not be closed but cancelled");
      } catch (err) {
          throw err
      }

  } catch (err) {
      console.error("Cancel offer error:", err);
      throw err;
  }
});

it("taker partially accepts an offer and maker cancels remaining", async () => {
  // Test configuration
  const offerIdPartial = 356757;
  const amountTokenAForSale = new anchor.BN('500');
  const totalTokenBExpected = new anchor.BN('250');
  const deadlineUnixTimestamp = new anchor.BN('1734363981');
  const partialAmount = new anchor.BN('200');

  try {
      // 1. SETUP: Create new offer PDA and vault
      offer = PublicKey.findProgramAddressSync(
          [
              Buffer.from('offer'), 
              maker.publicKey.toBuffer(), 
              Buffer.from(new anchor.BN(offerIdPartial).toArray('le', 8))
          ],
          program.programId
      )[0];
      console.log(`offer pda: ${offer}`)

      // Create vault token account as ATA
      const vaultTokenAccount = getAssociatedTokenAddressSync(
        mint_a.publicKey,
        offer,
        true,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );
      console.log('New vault token account:', vaultTokenAccount.toBase58());

      // 2. INITIAL BALANCES: Record maker's starting balance
      const makerBalanceBefore = await connection.getTokenAccountBalance(
          makerTokenAccount.address, 
          'confirmed'
      );
      console.log('Maker balance before:', Number(makerBalanceBefore.value.amount));

      // 3. CREATE OFFER: Initialize the offer and send tokens to vault
      await program.methods
          .createOfferAndSendTokensToVault(
              new anchor.BN(offerIdPartial),
              amountTokenAForSale,
              totalTokenBExpected,
              deadlineUnixTimestamp
          )
          .accountsPartial({
              maker: maker.publicKey,
              offer: offer,
              makerTokenAccount: makerTokenAccount.address,
              inputTokenMint: mint_a.publicKey,
              outputTokenMint: mint_b.publicKey,
              tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([maker])
          .rpc();

      console.log('Offer created successfully');

      // Verify offer creation
      const makerBalanceAfter = await connection.getTokenAccountBalance(makerTokenAccount.address);
      const vaultBalanceAfter = await connection.getTokenAccountBalance(vaultTokenAccount);
      
      console.log('Maker balance after:', Number(makerBalanceAfter.value.amount));
      console.log('Vault balance after:', Number(vaultBalanceAfter.value.amount));

      // 4. WHITELIST: Create and add taker to whitelist
      whitelist = PublicKey.findProgramAddressSync(
          [
              Buffer.from('whitelist'), 
              maker.publicKey.toBuffer(), 
              Buffer.from(new anchor.BN(offerIdPartial).toArray('le', 8))
          ],
          program.programId
      )[0];

      await program.methods.manageWhitelist([taker.publicKey])
          .accounts({
              maker: maker.publicKey,
              offer: offer,
          })
          .signers([maker])
          .rpc();

      // 5. SETUP TOKEN ACCOUNTS: Prepare all necessary token accounts
      makerRecieveTokenAccount = await getAssociatedTokenAddress(
          mint_b.publicKey, 
          maker.publicKey
      );
      
      takerPaymentTokenAccount = await getAssociatedTokenAddress(
          mint_b.publicKey, 
          taker.publicKey
      );
      
      takerReceiveTokenAccount = await getAssociatedTokenAddress(
          mint_a.publicKey, 
          taker.publicKey
      );
      
      feeTokenAccount = await getAssociatedTokenAddress(
          mint_b.publicKey, 
          new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv')
      );

      // 6. PRE-TAKE VERIFICATION: Check initial balances and account state
      const vaultBalanceBefore = await connection.getTokenAccountBalance(vaultTokenAccount);
      const takerTokenBBefore = await connection.getTokenAccountBalance(takerPaymentTokenAccount);
      const takerTokenABefore = await connection.getTokenAccountBalance(takerReceiveTokenAccount);
      
      console.log('\nPre-take state:');
      console.log('Vault balance:', Number(vaultBalanceBefore.value.amount));
      console.log('Taker token B balance:', Number(takerTokenBBefore.value.amount));
      console.log('Initial taker token A balance:', Number(takerTokenABefore.value.amount));
      
      const vaultAccountInfo = await connection.getAccountInfo(vaultTokenAccount);
      console.log('Vault owner:', vaultAccountInfo?.owner.toBase58());
      console.log('Offer PDA:', offer.toBase58());

      // 7. EXECUTE PARTIAL TAKE
      await program.methods.takeOffer(partialAmount)
          .accountsPartial({
              core: {
                  taker: taker.publicKey,
                  offer: offer,
                  maker: maker.publicKey,
                  whitelist: whitelist,
              },
              token: {
                  takerPaymentTokenAccount: takerPaymentTokenAccount,
                  vaultTokenAccount: vaultTokenAccount,
                  inputTokenMint: mint_a.publicKey,
                  outputTokenMint: mint_b.publicKey,
                  tokenProgram: TOKEN_PROGRAM_ID,
                  feeWallet: new PublicKey('B5WFNofBtPcFUS9oR2oAuxTHsSCUVp3C4VjFtejKEUnv'),
                  taker: taker.publicKey,
                  maker: maker.publicKey,
                  offer: offer,
              }
          })
          .signers([taker])
          .rpc();

      // 8. POST-TAKE VERIFICATION: Check balances and state after partial take
      console.log('\nPost-take state:');
      const vaultBalanceAfterTake = await connection.getTokenAccountBalance(vaultTokenAccount);
      const takerTokenAAfterTake = await connection.getTokenAccountBalance(takerReceiveTokenAccount);
      const offerAfterTake = await program.account.offer.fetch(offer);

      console.log('Vault balance after take:', Number(vaultBalanceAfterTake.value.amount));
      console.log('Taker token A balance after take:', Number(takerTokenAAfterTake.value.amount));
      console.log('Remaining amount:', offerAfterTake.tokenAmountRemaining.toString());
      console.log('Offer status:', offerAfterTake.status);

      // Verify partial take was successful
      assert.equal(
          Number(vaultBalanceAfterTake.value.amount),
          Number(vaultBalanceBefore.value.amount) - 200,
          "Vault balance should decrease by 200"
      );

      // Verify taker received exactly 200 more tokens than their initial balance
      assert.equal(
          Number(takerTokenAAfterTake.value.amount) - Number(takerTokenABefore.value.amount),
          200,
          "Taker should receive exactly 200 new tokens"
      );

      // 9. CANCEL REMAINING: Maker cancels remaining amount
      const makerBalanceBeforeCancel = await connection.getTokenAccountBalance(
          makerTokenAccount.address
      );

      console.log(makerBalanceBeforeCancel);

      try {
        await program.methods.cancelOffer()
          .accountsPartial({
            maker: maker.publicKey,
            offer: offer,
            whitelist: whitelist,
            makerTokenAccount: makerTokenAccount.address,
            inputTokenMint: mint_a.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([maker])
          .rpc();
      
        // Wait for a short delay after cancelling the offer
        await new Promise(resolve => setTimeout(resolve, 10000));
        assert.isNull(await connection.getAccountInfo(vaultTokenAccount), "Vault account should be closed");

      } catch (err) {
        console.error("Actual error message:", err.message);
        assert(err.message.includes("Account does not exist"), "Unexpected error occurred");
      }

      try {
        // Try to fetch the offer account - this should succeed
        const offerAccountAfterCancel = await program.account.offer.fetch(offer);
        
        // Verify the offer status is set to Cancelled
        assert.equal(JSON.stringify(offerAccountAfterCancel.status), JSON.stringify({ cancelled: {} }),"Offer status should be Cancelled");
        
        // Verify other offer account fields are preserved
        assert.equal(offerAccountAfterCancel.offerId.toString(),offerIdPartial.toString(),"Offer ID should be preserved");
        assert.equal(offerAccountAfterCancel.maker.toBase58(),maker.publicKey.toBase58(),"Maker public key should be preserved");
        console.log("Offer account is still available with status: Cancelled");
      } catch (err) {
        console.error("Error fetching offer account:", err);
        throw new Error("Offer account should still exist");
      }

  } catch (error) {
      console.error('Error in partial take test:', error);
      throw error;
  }
});
});