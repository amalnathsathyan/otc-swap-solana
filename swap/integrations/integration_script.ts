import { Program, web3, setProvider, AnchorProvider, Wallet, BN } from "@coral-xyz/anchor";
import { Swap } from "./swap";
import idl from "./idl.json";
import bs58 from "bs58";
import { Keypair, PublicKey, SystemProgram, Connection } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

// Set connection
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

// Set admin wallet and provider
const adminKeypair = Keypair.fromSecretKey(
  Uint8Array.from([])
);
const admin_wallet = new Wallet(adminKeypair);
const admin_provider = new AnchorProvider(connection, admin_wallet, {
  preflightCommitment: "processed",
});
// setProvider(admin_provider);

const MAKER_PVT_KEY = "" // wallet 1
const maker_wallet = new Wallet(Keypair.fromSecretKey(bs58.decode(MAKER_PVT_KEY)));

const maker_provider = new AnchorProvider(connection, maker_wallet, {
  preflightCommitment: "processed",
});

setProvider(maker_provider);
const program = new Program(idl as Swap);
const TAKER_PVT_KEY = "" // Account 3
const taker_wallet = new Wallet(Keypair.fromSecretKey(bs58.decode(TAKER_PVT_KEY)));
const taker_provider = new AnchorProvider(connection, taker_wallet, {
  preflightCommitment: "processed",
});
// Find the required PDA addresses
const [adminConfigPda, adminConfigBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("admin_config")],
  program.programId
);

const [feeConfigPda, feeConfigBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("fee")],
  program.programId
);

const [whitelistConfigPda, whitelistConfigBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("whitelist_config")],
  program.programId
);

const [mintWhitelistPda, mintWhitelistBump] = PublicKey.findProgramAddressSync(
  [Buffer.from("mint_whitelist")],
  program.programId
);

const feePercentage = 200;
const feeWallet = new PublicKey("ApyERY2be2Qzm3M2Qhf2uofsmKjzUYgrbQjcoPaGxqbN"); // Wallet 4 (raghu)
const updatedFeeWallet = new PublicKey("6jkVv9yuKQ8VFwuBaawGGoQrQUmLXasYrQjxpwUfTL1f"); // Account 5 (raghu)
const SOL_MINT = new PublicKey("So11111111111111111111111111111111111111112");
const USDT = new PublicKey("mnt7VJ91SwkwHH2TsKsRVpWAu8xip3LWgcYjVXC62MS"); // own token
const USDC = new PublicKey("mntnVMF6L9CZq8RZgvxh83AKUGxFfwAbVWhUQodHeHw");
const offerId1 = '771189582863134580';


const initialMints = [
  SOL_MINT,
  USDT,
];

// initialize admin
async function initializeAdmin() {
  try {
    const accounts = {
      admin: adminKeypair.publicKey,              // Exact match to "admin"
      admin_config: adminConfigPda,               // Exact match to "admin_config"
      fee_config: feeConfigPda,                   // Exact match to "fee_config"
      whitelist_config: whitelistConfigPda,       // Exact match to "whitelist_config"
      mint_whitelist: mintWhitelistPda,           // Exact match to "mint_whitelist"
      system_program: SystemProgram.programId     // Exact match to "system_program"
    };

    const adminInitialiseIntruction = await program.methods
      .initializeAdmin(new BN(feePercentage), feeWallet, false, initialMints)
      .accounts(accounts)
      .signers([adminKeypair])
      .instruction();


    const instructions: web3.TransactionInstruction[] = [
      adminInitialiseIntruction
    ];
    await createAndSendV0Tx(instructions, admin_provider, admin_provider.wallet.publicKey);
    // console.log("Admin initialized successfully. Transaction signature:", tx);
    const adminConfigFetched = await program.account.adminConfig.fetch(adminConfigPda);
    const feeConfigFetched = await program.account.feeConfig.fetch(feeConfigPda);
    const whitelistConfigFetched = await program.account.whitelistConfig.fetch(whitelistConfigPda);
    console.log("admin ", adminConfigFetched.admin);
    console.log("feeAddress ", feeConfigFetched.feeAddress);
    console.log("feePercentage ", feeConfigFetched.feePercentage);
    console.log("requireWhitelist ", whitelistConfigFetched.requireWhitelist);
  } catch (error) {
    console.error("Error initializing admin:", error);
  }
}

// add token mints
async function addWhiteListTokens() {
  try {
    const accounts = {
      admin: adminKeypair.publicKey,
      admin_config: adminConfigPda,
      mint_whitelist: mintWhitelistPda,
      systemProgram: SystemProgram.programId
    }

    const tokenWhitelistIntruction = await program.methods
      .addMintsToWhitelist([USDC])
      .accounts(accounts)
      .signers([adminKeypair])
      .instruction();


    const instructions: web3.TransactionInstruction[] = [
      tokenWhitelistIntruction
    ];
    await createAndSendV0Tx(instructions, admin_provider, admin_provider.wallet.publicKey);
    const mintwhitelistFetched = await program.account.mintWhitelist.fetch(mintWhitelistPda);
    console.log("Mint Public Keys:");
    mintwhitelistFetched.mints.forEach((mint: web3.PublicKey, index: number) => {
      console.log(`Mint ${index + 1}:`, mint.toBase58());
    });
  } catch (error) {
    console.error("Error initializing admin:", error);
  }
}

// remove token mints
async function removeWhiteListTokens() {
  try {
    const accounts = {
      admin: adminKeypair.publicKey,
      admin_config: adminConfigPda,
      mint_whitelist: mintWhitelistPda,
      systemProgram: SystemProgram.programId
    }

    const tokenWhitelistIntruction = await program.methods
      .removeMintsFromWhitelist([USDC])
      .accounts(accounts)
      .signers([adminKeypair])
      .instruction();


    const instructions: web3.TransactionInstruction[] = [
      tokenWhitelistIntruction
    ];
    await createAndSendV0Tx(instructions, admin_provider, admin_provider.wallet.publicKey);
    const mintwhitelistFetched = await program.account.mintWhitelist.fetch(mintWhitelistPda);
    console.log("Mint Public Keys:");
    mintwhitelistFetched.mints.forEach((mint: web3.PublicKey, index: number) => {
      console.log(`Mint ${index + 1}:`, mint.toBase58());
    });
  } catch (error) {
    console.error("Error initializing admin:", error);
  }
}

// update fee percentage
async function updateFeePercentage() {
  try {
    const accounts = {
      admin: adminKeypair.publicKey,
      admin_config: adminConfigPda,
      fee_config: feeConfigPda,
      systemProgram: SystemProgram.programId
    }

    const updateFeeIntruction = await program.methods
      .updateFeePercentage(new BN('100'))
      .accounts(accounts)
      .signers([adminKeypair])
      .instruction();


    const instructions: web3.TransactionInstruction[] = [
      updateFeeIntruction
    ];
    await createAndSendV0Tx(instructions, admin_provider, admin_provider.wallet.publicKey);
    const feeFetched = await program.account.feeConfig.fetch(feeConfigPda);
    console.log("Fee Percentage: ", feeFetched.feePercentage.toNumber());
  } catch (error) {
    console.error("Error initializing admin:", error);
  }
}

// update fee address
async function updateFeeAddress() {
  try {
    const accounts = {
      admin: adminKeypair.publicKey,
      admin_config: adminConfigPda,
      fee_config: feeConfigPda,
      systemProgram: SystemProgram.programId
    }

    const updateFeeIntruction = await program.methods
      .updateFeeAddress(updatedFeeWallet)
      .accounts(accounts)
      .signers([adminKeypair])
      .instruction();


    const instructions: web3.TransactionInstruction[] = [
      updateFeeIntruction
    ];
    await createAndSendV0Tx(instructions, admin_provider, admin_provider.wallet.publicKey);
    const feeFetched = await program.account.feeConfig.fetch(feeConfigPda);
    console.log("Fee Percentage: ", feeFetched.feeAddress.toBase58());
  } catch (error) {
    console.error("Error initializing admin:", error);
  }
}

// toggle whitelist check
async function toggleWhitelistCheck() {
  try {
    const accounts = {
      admin: adminKeypair.publicKey,
      admin_config: adminConfigPda,
      whitelist_config: whitelistConfigPda,
      systemProgram: SystemProgram.programId
    }

    const toggleWhitelistInstruction = await program.methods
      .toggleRequireWhitelist()
      .accounts(accounts)
      .signers([adminKeypair])
      .instruction();


    const instructions: web3.TransactionInstruction[] = [
      toggleWhitelistInstruction
    ];
    await createAndSendV0Tx(instructions, admin_provider, admin_provider.wallet.publicKey);
    const whitelistConfigFetched = await program.account.whitelistConfig.fetch(whitelistConfigPda);
    console.log("Whitelist Config ", whitelistConfigFetched.requireWhitelist);
  } catch (error) {
    console.error("Error in toggle whitelist:", error);
  }
}

// createOffer and add whitelist
async function createOffer() {
  try {
    console.log("========createOffer============");

    const usdtMintMakerATA = await getAssociatedTokenAddress(
      USDT,
      maker_wallet.publicKey,
      false, // Not a delegate account,
      TOKEN_2022_PROGRAM_ID
    );
    const makerUSDTAccount = usdtMintMakerATA;

    const offer = PublicKey.findProgramAddressSync(
      [Buffer.from('offer'), maker_wallet.publicKey.toBuffer(), Buffer.from(new BN(offerId1).toArray('le', 8))],
      program.programId
    )[0];

    const vaultTokenAccount = await getAssociatedTokenAddressSync(
      USDT,
      offer,
      true,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    )

    const accounts = {
      maker: maker_wallet.publicKey,
      offer,
      adminConfig: adminConfigPda,
      feeConfig: feeConfigPda,
      makerTokenAccount: makerUSDTAccount,
      vaultTokenAccount: vaultTokenAccount,
      inputTokenMint: USDT, // Defining input token mint here
      outputTokenMint: USDC, // Defining output token mint here
      tokenProgram: TOKEN_2022_PROGRAM_ID, // It can accept both TOKEN_PROGRAM & TOKEN_22 PROGRAM, proper input expected from fron-end
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    }
    const createOfferAndSendTokensToVaultInstruction = await program.methods.createOfferAndSendTokensToVault(
      new BN(offerId1), // Offer ID
      new BN('500'), //amout of token_a for sale
      new BN('250'), //total amount of token_b expected
      new BN('1765589361') //deadline as unix timestamp 
    ).accounts(accounts)
    // .signers([maker_wallet.payer])
    .instruction()

    const whitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), maker_wallet.publicKey.toBuffer(), Buffer.from(new BN(offerId1).toArray('le', 8))],
      program.programId
    )[0];

    const whitelistAccounts = {
      maker: maker_wallet.publicKey,
      whitelist: whitelist,
      offer: offer,
      systemProgram: SystemProgram.programId
    }

    const whitelistInstruction = await program.methods.manageWhitelist(
      [taker_wallet.publicKey]
    ).accounts(whitelistAccounts)
    // .signers([maker_wallet.payer])
    .instruction();

    const instructions: web3.TransactionInstruction[] = [
      createOfferAndSendTokensToVaultInstruction,
      whitelistInstruction
    ];
    await createAndSendV0Tx(instructions, maker_provider, maker_wallet.publicKey);
    const takerWhitelistFetched = await program.account.whitelist.fetch(whitelist);
    takerWhitelistFetched.takers.forEach((taker: web3.PublicKey, index: number) => {
      console.log(`Taker ${index + 1}:`, taker.toBase58());
    });
  } catch (error) {
    console.error("Error in creating offer:", error);
  }
}

// cancel offer
async function cancelOffer() {
  try {
    const usdtMintMakerATA = await getOrCreateAssociatedTokenAccount(
      connection,
      maker_wallet.payer,
      USDT,
      maker_wallet.publicKey,
      false,
      'confirmed',
      { commitment: 'confirmed' },
      TOKEN_2022_PROGRAM_ID
    )

    const makerUSDTAccount = usdtMintMakerATA;

    const offer = PublicKey.findProgramAddressSync(
      [Buffer.from('offer'), maker_wallet.publicKey.toBuffer(), Buffer.from(new BN(offerId1).toArray('le', 8))],
      program.programId
    )[0];

    const offerVal = await program.account.offer.fetch(offer); 

    const vaultTokenAccount = await getAssociatedTokenAddressSync(
      USDT,
      offer,
      true,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    )

    const whitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), maker_wallet.publicKey.toBuffer(), Buffer.from(new BN(offerId1).toArray('le', 8))],
      program.programId
    )[0];

    const accounts = {
      maker: maker_wallet.publicKey,
      offer: offer,
      whitelist: whitelist,
      makerTokenAccount: makerUSDTAccount.address,
      vaultTokenAccount: vaultTokenAccount,
      inputTokenMint: USDT,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    }
    const cancelOfferInstruction = await program.methods.cancelOffer()
          .accountsPartial(accounts)
          .signers([maker_wallet.payer])
          .instruction();

    const instructions: web3.TransactionInstruction[] = [
      cancelOfferInstruction
    ];
    await createAndSendV0Tx(instructions, maker_provider, maker_wallet.publicKey);
  } catch (error) {
    console.error("Error in creating offer:", error);
  }
}


// cancel offer
async function manageWhitelistTakers() {
  try {
    let offer = PublicKey.findProgramAddressSync(
      [Buffer.from('offer'), maker_wallet.publicKey.toBuffer(), Buffer.from(new BN(offerId1).toArray('le', 8))],
      program.programId
    )[0];

    const whitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), maker_wallet.publicKey.toBuffer(), Buffer.from(new BN(offerId1).toArray('le', 8))],
      program.programId
    )[0];

    const accounts = {
      maker: maker_wallet.publicKey,
      whitelist: whitelist,
      offer: offer,
      systemProgram: SystemProgram.programId
    }

    const addWhitelistTakersInstruction = await program.methods.manageWhitelist(
      [taker_wallet.publicKey, new PublicKey("ApyERY2be2Qzm3M2Qhf2uofsmKjzUYgrbQjcoPaGxqbN")]
    ).accounts(accounts)
    // .signers([maker_wallet.payer])
    .instruction()

    const instructions: web3.TransactionInstruction[] = [
      addWhitelistTakersInstruction
    ];
    await createAndSendV0Tx(instructions, maker_provider, maker_wallet.publicKey);
    const takerWhitelistFetched = await program.account.whitelist.fetch(whitelist);
    console.log("Whitelisted Public Taker Keys:");
    takerWhitelistFetched.takers.forEach((taker: web3.PublicKey, index: number) => {
      console.log(`Taker ${index + 1}:`, taker.toBase58());
    });
  } catch (error) {
    console.log("Error in adding whitelist takers: ", error);
  }
}


// takeOffer
async function takeOffer() {
  try {
    console.log("========takeOffer============");

    let offer = PublicKey.findProgramAddressSync(
      [Buffer.from('offer'), maker_wallet.publicKey.toBuffer(), Buffer.from(new BN(offerId1).toArray('le', 8))],
      program.programId
    )[0];

    const makerRecieveTokenAccount = await getAssociatedTokenAddress(
      USDC,
      maker_wallet.publicKey,
      false, // Not a delegate account
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_PROGRAM_ID
    );

    const takerPaymentTokenAccount = await getAssociatedTokenAddress(
      USDC,
      taker_wallet.publicKey,
      false, // Not a delegate account
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_PROGRAM_ID
    );

    const takerReceiveTokenAccount = await getAssociatedTokenAddress(
      USDT,
      taker_wallet.publicKey,
      false, // Not a delegate account
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_PROGRAM_ID
    );

    const feeTokenAccount = await getAssociatedTokenAddress(
      USDC,
      feeWallet,
      false, // Not a delegate account
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_PROGRAM_ID
    );

    let vaultTokenAccount = await getAssociatedTokenAddressSync(
      USDT,
      offer,
      true,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    )

    const whitelist = PublicKey.findProgramAddressSync(
      [Buffer.from('whitelist'), maker_wallet.publicKey.toBuffer(), Buffer.from(new BN(offerId1).toArray('le', 8))],
      program.programId
    )[0];

    const accounts = {
      core: {
          taker: taker_wallet.publicKey,
          adminConfig: adminConfigPda,
          offer: offer,
          maker: maker_wallet.publicKey,
          whitelist: whitelist,
      },
      token: {
          makerReceiveTokenAccount: makerRecieveTokenAccount,
          takerPaymentTokenAccount: takerPaymentTokenAccount,
          takerReceiveTokenAccount: takerReceiveTokenAccount,
          feeTokenAccount: feeTokenAccount,
          vaultTokenAccount: vaultTokenAccount,
          inputTokenMint: USDT,
          outputTokenMint: USDC,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          feeWallet: feeWallet,
          taker: taker_wallet.publicKey,
          maker: maker_wallet.publicKey,
          offer: offer,
      }
  }

    const takeOfferInstruction = await program.methods.takeOffer(
      new BN('300')
    ).accounts(accounts).signers(
      [taker_wallet.payer]
    ).instruction()

    const instructions: web3.TransactionInstruction[] = [
      takeOfferInstruction
    ];
    await setProvider(taker_provider);
    await createAndSendV0Tx(instructions, taker_provider, taker_wallet.publicKey);
  } catch (error) {
    console.error("Error in creating offer:", error);
  }
}

async function createAndSendV0Tx(
  txInstructions: web3.TransactionInstruction[],
  provider,
  user
) {
  // Step 1 - Fetch the latest blockhash
  let latestBlockhash = await provider.connection.getLatestBlockhash(
    "confirmed"
  );
  console.log(
    "   ✅ - Fetched latest blockhash. Last Valid Height:",
    latestBlockhash.lastValidBlockHeight
  );

  // Step 2 - Generate Transaction Message
  const messageV0 = new web3.TransactionMessage({
    payerKey: user,
    recentBlockhash: latestBlockhash.blockhash,
    instructions: txInstructions,
  }).compileToV0Message();
  console.log("   ✅ - Compiled Transaction Message");
  const transaction = new web3.VersionedTransaction(messageV0);

  // Step 3 - Sign your transaction with the required `Signers`
  provider.wallet.signTransaction(transaction);
  console.log("   ✅ - Transaction Signed");

  // Step 4 - Send our v0 transaction to the cluster
  const txid = await provider.connection.sendTransaction(transaction, {
    maxRetries: 5,
  });
  console.log("   ✅ - Transaction sent to network ", txid);

  // Step 5 - Confirm Transaction
  const confirmation = await provider.connection.confirmTransaction({
    signature: txid,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
  });
  if (confirmation.value.err) {
    throw new Error(
      `   ❌ - Transaction not confirmed.\nReason: ${confirmation.value.err}`
    );
  }
}

// --------- ADMIN FUNCTIONS ------------
// initializeAdmin();
// addWhiteListTokens();
// removeWhiteListTokens();
// updateFeePercentage();
// updateFeeAddress();
// toggleWhitelistCheck();

// ----------- MAKER FUNCTIONS -------------
// createOffer();
// manageWhitelistTakers();
cancelOffer();

// ----------- TAKER FUNCTIONS -------------
// takeOffer();