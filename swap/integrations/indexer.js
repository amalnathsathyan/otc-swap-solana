const { Connection, PublicKey } = require("@solana/web3.js");
const anchor = require("@coral-xyz/anchor");

const programId = new anchor.web3.PublicKey("7dgTkUkHLBEZEv3VfpvkGfmmmNXZy4vdxxLz5XLyELhC"); // Replace with your program ID
const idl = require("./idl.json");
const program = new anchor.Program(idl, programId);

async function getTransactionLogs(txHash) {
    // Connect to a Solana cluster
    const RPC_URL = "https://api.devnet.solana.com";
    const connection = new Connection(RPC_URL, "confirmed");

    const transactionDetails = await connection.getParsedTransaction(txHash, {
        commitment: "confirmed",
        maxSupportedTransactionVersion: 0,
    });

    if (!transactionDetails) {
        console.log("Transaction not found!");
        return [];
    }

    const logs = transactionDetails.meta.logMessages || [];

    console.log(" logs ", logs)
    return logs;
}

async function decodeEvents(txHash) {
    const logs = await getTransactionLogs(txHash);

    // Filter relevant logs
    const filteredLogs = logs.filter((log) => log.includes("Program data"));

    console.log("filteredLogs ", filteredLogs);
    // Decode logs
    filteredLogs.forEach((log) => {
        try {
            const decodedEvent = program.coder.events.decode(log.replace("Program data: ", ""));
            if (decodedEvent) {
                console.log("Decoded Event:", decodedEvent);
            }
        } catch (error) {
            // Skip logs that aren't events
        }
    });
}

const txHash = "4HJ5WKiS7BgRxtFQsE19HnWnxMw1nnE1zeofJSoULuufHew8dUnaJa7Uz7RtY1FVM8Zrbzn9wvMzPGbmrNgSB8pb"
decodeEvents(txHash);
