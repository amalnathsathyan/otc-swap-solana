/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/swap.json`.
 */
export type Swap = {
  "address": "7dgTkUkHLBEZEv3VfpvkGfmmmNXZy4vdxxLz5XLyELhC",
  "metadata": {
    "name": "swap",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "addMintsToWhitelist",
      "discriminator": [
        116,
        92,
        68,
        136,
        148,
        70,
        0,
        95
      ],
      "accounts": [
        {
          "name": "admin",
          "docs": [
            "Admin signer"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "adminConfig",
          "docs": [
            "PDA verifying admin authority"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  109,
                  105,
                  110,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "mintWhitelist",
          "docs": [
            "Mint whitelist to modify"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  105,
                  110,
                  116,
                  95,
                  119,
                  104,
                  105,
                  116,
                  101,
                  108,
                  105,
                  115,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "newMints",
          "type": {
            "vec": "pubkey"
          }
        }
      ]
    },
    {
      "name": "cancelOffer",
      "docs": [
        "if incase, it's completed shouldn't be able to call this"
      ],
      "discriminator": [
        92,
        203,
        223,
        40,
        92,
        89,
        53,
        119
      ],
      "accounts": [
        {
          "name": "maker",
          "docs": [
            "Original offer maker who will:",
            "- Sign the cancellation transaction",
            "- Receive returned tokens",
            "- Receive rent from closed accounts",
            "Must match the maker stored in the offer"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "offer",
          "docs": [
            "The offer PDA that serves dual purpose as:",
            "1. Storage for offer details",
            "2. Authority over the vault token account",
            "",
            "Constraints:",
            "- Must be signed by original maker",
            "- Must be in Ongoing status",
            "- Will be closed with rent returned to maker",
            "",
            "Seeds: [\"offer\", maker_pubkey, offer_id]"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "maker"
              },
              {
                "kind": "account",
                "path": "offer.offer_id",
                "account": "offer"
              }
            ]
          }
        },
        {
          "name": "whitelist",
          "docs": [
            "The whitelist PDA storing allowed takers",
            "Will be closed and rent returned to maker",
            "",
            "Seeds: [\"whitelist\", maker_pubkey, offer_id]"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  119,
                  104,
                  105,
                  116,
                  101,
                  108,
                  105,
                  115,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "maker"
              },
              {
                "kind": "account",
                "path": "offer.offer_id",
                "account": "offer"
              }
            ]
          }
        },
        {
          "name": "makerTokenAccount",
          "docs": [
            "Token account owned by maker that will receive",
            "returned tokens from the vault",
            "",
            "Constraints:",
            "- Must be owned by maker",
            "- Must match input token mint from offer"
          ],
          "writable": true
        },
        {
          "name": "vaultTokenAccount",
          "docs": [
            "The vault token account holding the offered tokens",
            "Created as an Associated Token Account owned by offer PDA",
            "Will be closed after returning tokens",
            "",
            "Constraints:",
            "- Must be an ATA",
            "- Must have offer PDA as authority",
            "- Must match input token mint"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "offer"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "inputTokenMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "adminConfig",
          "docs": [
            "Admin configuration PDA for updating protocol statistics",
            "Tracks active/cancelled offer counts",
            "",
            "Seeds: [\"admin_config\"]"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  109,
                  105,
                  110,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "inputTokenMint",
          "docs": [
            "The mint of the token being returned",
            "Used for transfer_checked validation"
          ]
        },
        {
          "name": "tokenProgram",
          "docs": [
            "Token interface program for Token-2022 support"
          ]
        },
        {
          "name": "associatedTokenProgram",
          "docs": [
            "Required for ATA validation"
          ],
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "createOfferAndSendTokensToVault",
      "discriminator": [
        205,
        44,
        113,
        254,
        16,
        213,
        68,
        130
      ],
      "accounts": [
        {
          "name": "maker",
          "docs": [
            "The offer creator who will:",
            "- Pay for account initialization",
            "- Provide tokens for the trade",
            "- Control offer parameters"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "offer",
          "docs": [
            "The offer PDA storing all trade details",
            "Space breakdown:",
            "- 8 bytes discriminator",
            "- 8 bytes offer id",
            "- 32 bytes maker pubkey",
            "- 32 bytes input token mint",
            "- 8 bytes token amount",
            "- 32 bytes output token mint",
            "- 8 bytes expected amount",
            "- 8 bytes for token amount remaining",
            "- 8 bytes for expected fulfilled amount",
            "- 8 bytes deadline",
            "- 1 byte offer status",
            "- 8 bytes fee percentage",
            "- 32 bytes fee wallet"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  111,
                  102,
                  102,
                  101,
                  114
                ]
              },
              {
                "kind": "account",
                "path": "maker"
              },
              {
                "kind": "arg",
                "path": "offerId"
              }
            ]
          }
        },
        {
          "name": "adminConfig",
          "docs": [
            "Admin configuration for protocol verification and statistics",
            "Must be initialized before any offers can be created"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  109,
                  105,
                  110,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "feeConfig",
          "docs": [
            "Fee configuration providing protocol fee parameters",
            "Must be initialized before any offers can be created"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "makerTokenAccount",
          "docs": [
            "Maker's token account containing tokens to be offered",
            "Must match the input token mint"
          ],
          "writable": true
        },
        {
          "name": "vaultTokenAccount",
          "docs": [
            "Vault token account created as an Associated Token Account",
            "Will hold the offered tokens until trade completion",
            "Authority is the offer PDA"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "offer"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "inputTokenMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "inputTokenMint",
          "docs": [
            "Input token mint (token being offered)"
          ]
        },
        {
          "name": "outputTokenMint",
          "docs": [
            "Output token mint (token being requested)"
          ]
        },
        {
          "name": "tokenProgram",
          "docs": [
            "Token interface program for Token-2022 support"
          ]
        },
        {
          "name": "associatedTokenProgram",
          "docs": [
            "Required for ATA initialization"
          ],
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "offerId",
          "type": "u64"
        },
        {
          "name": "tokenAmount",
          "type": "u64"
        },
        {
          "name": "expectedTotalAmount",
          "type": "u64"
        },
        {
          "name": "deadline",
          "type": "i64"
        }
      ]
    },
    {
      "name": "initializeAdmin",
      "discriminator": [
        35,
        176,
        8,
        143,
        42,
        160,
        61,
        158
      ],
      "accounts": [
        {
          "name": "admin",
          "docs": [
            "The admin account that will control protocol settings"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "adminConfig",
          "docs": [
            "PDA storing admin details and protocol statistics",
            "Space breakdown:",
            "- 8 bytes discriminator",
            "- 32 bytes admin pubkey"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  109,
                  105,
                  110,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "feeConfig",
          "docs": [
            "PDA storing fee configuration",
            "Space breakdown:",
            "- 8 bytes discriminator",
            "- 8 bytes fee percentage",
            "- 32 bytes fee wallet address"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "whitelistConfig",
          "docs": [
            "PDA storing whitelist enforcement configuration",
            "Space breakdown:",
            "- 8 bytes discriminator",
            "- 1 byte boolean flag"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  119,
                  104,
                  105,
                  116,
                  101,
                  108,
                  105,
                  115,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "mintWhitelist",
          "docs": [
            "PDA storing whitelisted token mints",
            "Space breakdown:",
            "- 8 bytes discriminator",
            "- 4 bytes vector length",
            "- 32 * 50 bytes for pubkeys (max 50 mints)"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  105,
                  110,
                  116,
                  95,
                  119,
                  104,
                  105,
                  116,
                  101,
                  108,
                  105,
                  115,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "docs": [
            "Global PDA for tracking maker sequences"
          ],
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "feePercentage",
          "type": "u64"
        },
        {
          "name": "feeWallet",
          "type": "pubkey"
        },
        {
          "name": "requireWhitelist",
          "type": "bool"
        },
        {
          "name": "initialMints",
          "type": {
            "vec": "pubkey"
          }
        }
      ]
    },
    {
      "name": "manageWhitelist",
      "discriminator": [
        252,
        197,
        13,
        121,
        14,
        54,
        39,
        93
      ],
      "accounts": [
        {
          "name": "maker",
          "docs": [
            "Original offer maker, must sign whitelist operations"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "whitelist",
          "docs": [
            "The whitelist PDA storing allowed takers",
            "Created on first use with init_if_needed",
            "Space for up to 50 taker addresses"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  119,
                  104,
                  105,
                  116,
                  101,
                  108,
                  105,
                  115,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "maker"
              },
              {
                "kind": "account",
                "path": "offer.offer_id",
                "account": "offer"
              }
            ]
          }
        },
        {
          "name": "offer",
          "docs": [
            "The offer this whitelist belongs to",
            "Used to verify maker authority"
          ]
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "takers",
          "type": {
            "vec": "pubkey"
          }
        }
      ]
    },
    {
      "name": "removeMintsFromWhitelist",
      "discriminator": [
        214,
        232,
        203,
        229,
        255,
        239,
        201,
        97
      ],
      "accounts": [
        {
          "name": "admin",
          "docs": [
            "Admin signer"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "adminConfig",
          "docs": [
            "PDA verifying admin authority"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  109,
                  105,
                  110,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "mintWhitelist",
          "docs": [
            "Mint whitelist to modify"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  109,
                  105,
                  110,
                  116,
                  95,
                  119,
                  104,
                  105,
                  116,
                  101,
                  108,
                  105,
                  115,
                  116
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "removeMints",
          "type": {
            "vec": "pubkey"
          }
        }
      ]
    },
    {
      "name": "takeOffer",
      "discriminator": [
        128,
        156,
        242,
        207,
        237,
        192,
        103,
        240
      ],
      "accounts": [
        {
          "name": "core",
          "accounts": [
            {
              "name": "taker",
              "docs": [
                "Transaction signer who will take the offer.",
                "Responsible for:",
                "- Paying for account initialization",
                "- Sending payment tokens",
                "- Paying protocol fees",
                "- Receiving offered tokens"
              ],
              "writable": true,
              "signer": true
            },
            {
              "name": "adminConfig",
              "docs": [
                "Protocol admin configuration.",
                "Tracks global settings and statistics.",
                "PDA with seeds: [\"admin_config\"]"
              ],
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      97,
                      100,
                      109,
                      105,
                      110,
                      95,
                      99,
                      111,
                      110,
                      102,
                      105,
                      103
                    ]
                  }
                ]
              }
            },
            {
              "name": "offer",
              "docs": [
                "The offer being taken.",
                "PDA with seeds: [\"offer\", maker_pubkey, offer_id]",
                "Constraints:",
                "- Must be in Ongoing status",
                "- Closed on completion with rent returned to maker"
              ],
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      111,
                      102,
                      102,
                      101,
                      114
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "offer.maker",
                    "account": "offer"
                  },
                  {
                    "kind": "account",
                    "path": "offer.offer_id",
                    "account": "offer"
                  }
                ]
              }
            },
            {
              "name": "maker",
              "writable": true
            },
            {
              "name": "whitelist",
              "docs": [
                "Whitelist of authorized takers.",
                "PDA with seeds: [\"whitelist\", maker_pubkey, offer_id]",
                "Constraints:",
                "- Must include taker's public key",
                "- Closed on offer completion"
              ],
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "const",
                    "value": [
                      119,
                      104,
                      105,
                      116,
                      101,
                      108,
                      105,
                      115,
                      116
                    ]
                  },
                  {
                    "kind": "account",
                    "path": "offer.maker",
                    "account": "offer"
                  },
                  {
                    "kind": "account",
                    "path": "offer.offer_id",
                    "account": "offer"
                  }
                ]
              }
            }
          ]
        },
        {
          "name": "token",
          "accounts": [
            {
              "name": "makerReceiveTokenAccount",
              "docs": [
                "Maker's token account for receiving payment.",
                "Automatically created as an ATA if it doesn't exist."
              ],
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "account",
                    "path": "maker"
                  },
                  {
                    "kind": "account",
                    "path": "tokenProgram"
                  },
                  {
                    "kind": "account",
                    "path": "outputTokenMint"
                  }
                ],
                "program": {
                  "kind": "const",
                  "value": [
                    140,
                    151,
                    37,
                    143,
                    78,
                    36,
                    137,
                    241,
                    187,
                    61,
                    16,
                    41,
                    20,
                    142,
                    13,
                    131,
                    11,
                    90,
                    19,
                    153,
                    218,
                    255,
                    16,
                    132,
                    4,
                    142,
                    123,
                    216,
                    219,
                    233,
                    248,
                    89
                  ]
                }
              }
            },
            {
              "name": "takerPaymentTokenAccount",
              "docs": [
                "Taker's token account for sending payment.",
                "Automatically created as an ATA if it doesn't exist."
              ],
              "writable": true
            },
            {
              "name": "takerReceiveTokenAccount",
              "docs": [
                "Taker's token account for receiving offered tokens.",
                "Automatically created as an ATA if it doesn't exist."
              ],
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "account",
                    "path": "taker"
                  },
                  {
                    "kind": "account",
                    "path": "tokenProgram"
                  },
                  {
                    "kind": "account",
                    "path": "inputTokenMint"
                  }
                ],
                "program": {
                  "kind": "const",
                  "value": [
                    140,
                    151,
                    37,
                    143,
                    78,
                    36,
                    137,
                    241,
                    187,
                    61,
                    16,
                    41,
                    20,
                    142,
                    13,
                    131,
                    11,
                    90,
                    19,
                    153,
                    218,
                    255,
                    16,
                    132,
                    4,
                    142,
                    123,
                    216,
                    219,
                    233,
                    248,
                    89
                  ]
                }
              }
            },
            {
              "name": "feeTokenAccount",
              "docs": [
                "Protocol fee receiving account.",
                "Constraints:",
                "- Must match configured fee wallet",
                "- Must use correct token mint"
              ],
              "writable": true,
              "pda": {
                "seeds": [
                  {
                    "kind": "account",
                    "path": "feeWallet"
                  },
                  {
                    "kind": "account",
                    "path": "tokenProgram"
                  },
                  {
                    "kind": "account",
                    "path": "outputTokenMint"
                  }
                ],
                "program": {
                  "kind": "const",
                  "value": [
                    140,
                    151,
                    37,
                    143,
                    78,
                    36,
                    137,
                    241,
                    187,
                    61,
                    16,
                    41,
                    20,
                    142,
                    13,
                    131,
                    11,
                    90,
                    19,
                    153,
                    218,
                    255,
                    16,
                    132,
                    4,
                    142,
                    123,
                    216,
                    219,
                    233,
                    248,
                    89
                  ]
                }
              }
            },
            {
              "name": "vaultTokenAccount",
              "docs": [
                "Vault holding the offered tokens.",
                "Constraints:",
                "- Must be owned by offer PDA",
                "- Must match input token mint"
              ],
              "writable": true
            },
            {
              "name": "inputTokenMint",
              "docs": [
                "Mint of the token being offered"
              ]
            },
            {
              "name": "outputTokenMint",
              "docs": [
                "Mint of the token being requested"
              ]
            },
            {
              "name": "tokenProgram",
              "docs": [
                "Required program interfaces"
              ]
            },
            {
              "name": "associatedTokenProgram",
              "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
            },
            {
              "name": "systemProgram",
              "address": "11111111111111111111111111111111"
            },
            {
              "name": "feeWallet"
            },
            {
              "name": "taker",
              "writable": true
            },
            {
              "name": "maker"
            },
            {
              "name": "offer"
            }
          ]
        }
      ],
      "args": [
        {
          "name": "inputTokenAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "toggleRequireWhitelist",
      "discriminator": [
        234,
        5,
        149,
        172,
        124,
        21,
        127,
        98
      ],
      "accounts": [
        {
          "name": "admin",
          "docs": [
            "Admin signer"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "adminConfig",
          "docs": [
            "PDA verifying admin authority"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  109,
                  105,
                  110,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "whitelistConfig",
          "docs": [
            "Whitelist configuration to toggle"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  119,
                  104,
                  105,
                  116,
                  101,
                  108,
                  105,
                  115,
                  116,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "updateFeeAddress",
      "discriminator": [
        110,
        156,
        143,
        17,
        70,
        205,
        116,
        191
      ],
      "accounts": [
        {
          "name": "admin",
          "docs": [
            "Admin signer"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "adminConfig",
          "docs": [
            "PDA verifying admin authority"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  109,
                  105,
                  110,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "feeConfig",
          "docs": [
            "Fee configuration to update"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "newAddress",
          "type": "pubkey"
        }
      ]
    },
    {
      "name": "updateFeePercentage",
      "discriminator": [
        102,
        119,
        197,
        160,
        139,
        102,
        182,
        0
      ],
      "accounts": [
        {
          "name": "admin",
          "docs": [
            "Admin signer"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "adminConfig",
          "docs": [
            "PDA verifying admin authority"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  100,
                  109,
                  105,
                  110,
                  95,
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "feeConfig",
          "docs": [
            "Fee configuration to update"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  101,
                  101
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "newFee",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "adminConfig",
      "discriminator": [
        156,
        10,
        79,
        161,
        71,
        9,
        62,
        77
      ]
    },
    {
      "name": "feeConfig",
      "discriminator": [
        143,
        52,
        146,
        187,
        219,
        123,
        76,
        155
      ]
    },
    {
      "name": "mintWhitelist",
      "discriminator": [
        21,
        127,
        24,
        53,
        117,
        168,
        234,
        153
      ]
    },
    {
      "name": "offer",
      "discriminator": [
        215,
        88,
        60,
        71,
        170,
        162,
        73,
        229
      ]
    },
    {
      "name": "whitelist",
      "discriminator": [
        204,
        176,
        52,
        79,
        146,
        121,
        54,
        247
      ]
    },
    {
      "name": "whitelistConfig",
      "discriminator": [
        58,
        51,
        12,
        166,
        38,
        109,
        18,
        255
      ]
    }
  ],
  "events": [
    {
      "name": "adminInitialized",
      "discriminator": [
        237,
        223,
        71,
        11,
        140,
        218,
        196,
        171
      ]
    },
    {
      "name": "feeUpdated",
      "discriminator": [
        228,
        75,
        43,
        103,
        9,
        196,
        182,
        4
      ]
    },
    {
      "name": "feeWalletUpdated",
      "discriminator": [
        239,
        21,
        163,
        102,
        93,
        63,
        3,
        248
      ]
    },
    {
      "name": "mintsAddedToWhitelist",
      "discriminator": [
        217,
        228,
        138,
        116,
        179,
        91,
        53,
        53
      ]
    },
    {
      "name": "mintsRemovedFromWhitelist",
      "discriminator": [
        103,
        176,
        231,
        135,
        82,
        141,
        69,
        19
      ]
    },
    {
      "name": "offerCancelled",
      "discriminator": [
        45,
        42,
        175,
        214,
        51,
        192,
        154,
        9
      ]
    },
    {
      "name": "offerCreated",
      "discriminator": [
        31,
        236,
        215,
        144,
        75,
        45,
        157,
        87
      ]
    },
    {
      "name": "offerTaken",
      "discriminator": [
        97,
        101,
        174,
        50,
        76,
        209,
        178,
        148
      ]
    },
    {
      "name": "takerUpdated",
      "discriminator": [
        221,
        147,
        234,
        94,
        135,
        205,
        91,
        223
      ]
    },
    {
      "name": "whitelistRequirementToggled",
      "discriminator": [
        167,
        221,
        168,
        215,
        11,
        108,
        229,
        22
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidOfferStatus",
      "msg": "Invalid Offer Status"
    },
    {
      "code": 6001,
      "name": "takerNotWhitelisted",
      "msg": "Taker not whitelisted"
    },
    {
      "code": 6002,
      "name": "invalidMaker",
      "msg": "Invalid Maker"
    },
    {
      "code": 6003,
      "name": "takerAlreadyWhitelisted",
      "msg": "Taker already exists"
    },
    {
      "code": 6004,
      "name": "mintAlreadyWhitelisted",
      "msg": "Token already whitelisted"
    },
    {
      "code": 6005,
      "name": "mintNotWhitelisted",
      "msg": "Token not whitelisted"
    },
    {
      "code": 6006,
      "name": "invalidAdmin",
      "msg": "Invalid Admin"
    },
    {
      "code": 6007,
      "name": "offerExpired",
      "msg": "Offer has expired"
    },
    {
      "code": 6008,
      "name": "insufficientAmount",
      "msg": "Insufficient amount available"
    },
    {
      "code": 6009,
      "name": "cannotCancelOffer",
      "msg": "Cannot Cancel Offer"
    },
    {
      "code": 6010,
      "name": "unauthorizedMaker",
      "msg": "Invalid Maker"
    },
    {
      "code": 6011,
      "name": "unauthorizedAdmin",
      "msg": "Unauthorized admin"
    },
    {
      "code": 6012,
      "name": "offerNotExpired",
      "msg": "Offer has not expired yet"
    },
    {
      "code": 6013,
      "name": "tooManyMints",
      "msg": "Maximum number of whitelisted mints reached"
    },
    {
      "code": 6014,
      "name": "invalidFeePercentage",
      "msg": "Fee percentage cannot exceed 100%"
    },
    {
      "code": 6015,
      "name": "adminNotInitialized",
      "msg": "Admin has not been initialized"
    },
    {
      "code": 6016,
      "name": "feeConfigNotInitialized",
      "msg": "Fee configuration has not been initialized"
    },
    {
      "code": 6017,
      "name": "invalidAddress",
      "msg": "Zero Address not allowed"
    },
    {
      "code": 6018,
      "name": "invalidTokenAccount",
      "msg": "Invalid token account owner"
    },
    {
      "code": 6019,
      "name": "invalidTokenMint",
      "msg": "Token mint mismatch"
    },
    {
      "code": 6020,
      "name": "invalidDeadline",
      "msg": "Deadline must be in the future"
    },
    {
      "code": 6021,
      "name": "whitelistFull",
      "msg": "Whitelist is full"
    },
    {
      "code": 6022,
      "name": "emptyTakersList",
      "msg": "Takers list cannot be empty"
    },
    {
      "code": 6023,
      "name": "invalidAmount",
      "msg": "Token amount must be greater than 0"
    },
    {
      "code": 6024,
      "name": "sequenceOverflow",
      "msg": "Sequence Overflow"
    },
    {
      "code": 6025,
      "name": "calculationError",
      "msg": "Calculation error occurred."
    },
    {
      "code": 6026,
      "name": "invalidVaultOwner",
      "msg": "Invalid vault owner"
    }
  ],
  "types": [
    {
      "name": "adminConfig",
      "docs": [
        "Configuration account for admin operations and offer tracking"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "docs": [
              "Admin's public key for authorization"
            ],
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "adminInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "feePercentage",
            "type": "u64"
          },
          {
            "name": "feeWallet",
            "type": "pubkey"
          },
          {
            "name": "requireWhitelist",
            "type": "bool"
          },
          {
            "name": "initialMints",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "cancellationReason",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "expired"
          },
          {
            "name": "makerCancelled"
          }
        ]
      }
    },
    {
      "name": "feeConfig",
      "docs": [
        "Account structure storing fee configuration for the protocol",
        "Controls both the fee amount and where fees are sent"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "feePercentage",
            "docs": [
              "The fee amount in basis points (1/100th of a percent)",
              "e.g., 100 = 1%, 50 = 0.5%, 25 = 0.25%"
            ],
            "type": "u64"
          },
          {
            "name": "feeAddress",
            "docs": [
              "The public key of the account that receives protocol fees",
              "All fees collected from trades will be sent to this address"
            ],
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "feeUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "oldFee",
            "type": "u64"
          },
          {
            "name": "newFee",
            "type": "u64"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "feeWalletUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "oldWallet",
            "type": "pubkey"
          },
          {
            "name": "newWallet",
            "type": "pubkey"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "mintWhitelist",
      "docs": [
        "Account structure storing the whitelist of permitted token mints",
        "This controls which tokens can be used in the protocol"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mints",
            "docs": [
              "Vector of public keys representing allowed token mints",
              "Only tokens from these mints can be used in the protocol when whitelist is enabled"
            ],
            "type": {
              "vec": "pubkey"
            }
          }
        ]
      }
    },
    {
      "name": "mintsAddedToWhitelist",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "newMints",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "mintsRemovedFromWhitelist",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "removedMints",
            "type": {
              "vec": "pubkey"
            }
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "offer",
      "docs": [
        "Account structure representing a token swap offer",
        "Stores all details about an offer including its current state and parameters"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offerId",
            "docs": [
              "Unique identifier for the offer"
            ],
            "type": "u64"
          },
          {
            "name": "maker",
            "docs": [
              "The public key of the account that created this offer",
              "Controls permissions for offer modification and cancellation"
            ],
            "type": "pubkey"
          },
          {
            "name": "inputTokenMint",
            "docs": [
              "The mint address of the token being offered",
              "Must be a whitelisted token mint if whitelist is enabled"
            ],
            "type": "pubkey"
          },
          {
            "name": "tokenAmount",
            "docs": [
              "The amount of tokens being offered",
              "Decreases as partial fills occur, reaches 0 when fully filled"
            ],
            "type": "u64"
          },
          {
            "name": "outputTokenMint",
            "docs": [
              "The mint address of the token being offered",
              "Must be a whitelisted token mint if whitelist is enabled"
            ],
            "type": "pubkey"
          },
          {
            "name": "expectedTotalAmount",
            "docs": [
              "The total amount of payment tokens expected in return",
              "Used to calculate the exchange rate for partial fills"
            ],
            "type": "u64"
          },
          {
            "name": "tokenAmountRemaining",
            "docs": [
              "token amount_a remaining after each trade"
            ],
            "type": "u64"
          },
          {
            "name": "expectedFulfilledAmount",
            "docs": [
              "token_b fullfilled"
            ],
            "type": "u64"
          },
          {
            "name": "deadline",
            "docs": [
              "Unix timestamp when this offer expires",
              "Offer cannot be taken after this time"
            ],
            "type": "i64"
          },
          {
            "name": "status",
            "docs": [
              "Current status of the offer",
              "Controls what operations are permitted"
            ],
            "type": {
              "defined": {
                "name": "offerStatus"
              }
            }
          },
          {
            "name": "feePercentage",
            "type": "u64"
          },
          {
            "name": "feeWallet",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "offerCancelled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offerId",
            "type": "u64"
          },
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "tokenAmount",
            "type": "u64"
          },
          {
            "name": "tokenMint",
            "type": "pubkey"
          },
          {
            "name": "reason",
            "type": {
              "defined": {
                "name": "cancellationReason"
              }
            }
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "offerCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offerId",
            "type": "u64"
          },
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "inputTokenMint",
            "type": "pubkey"
          },
          {
            "name": "outputTokenMint",
            "type": "pubkey"
          },
          {
            "name": "tokenAmount",
            "type": "u64"
          },
          {
            "name": "expectedAmount",
            "type": "u64"
          },
          {
            "name": "deadline",
            "type": "i64"
          }
        ]
      }
    },
    {
      "name": "offerStatus",
      "docs": [
        "Enum representing the possible states of an offer",
        "Used to track the offer's lifecycle and control permitted operations"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "initialized"
          },
          {
            "name": "vaultInitialized"
          },
          {
            "name": "ongoing"
          },
          {
            "name": "completed"
          },
          {
            "name": "cancelled"
          },
          {
            "name": "expired"
          }
        ]
      }
    },
    {
      "name": "offerTaken",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offerId",
            "type": "u64"
          },
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "taker",
            "type": "pubkey"
          },
          {
            "name": "inputTokenAmount",
            "type": "u64"
          },
          {
            "name": "paymentAmount",
            "type": "u64"
          },
          {
            "name": "feeAmount",
            "type": "u64"
          },
          {
            "name": "remainingAmount",
            "type": "u64"
          },
          {
            "name": "inputTokenMint",
            "type": "pubkey"
          },
          {
            "name": "outputTokenMint",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "takerUpdated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "offerId",
            "type": "u64"
          },
          {
            "name": "maker",
            "type": "pubkey"
          },
          {
            "name": "takers",
            "type": {
              "vec": "pubkey"
            }
          }
        ]
      }
    },
    {
      "name": "whitelist",
      "docs": [
        "Account structure representing a whitelist of allowed takers for a specific offer",
        "Controls which addresses can take (accept) a particular offer"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "maker",
            "docs": [
              "Maker who controls the whitelist"
            ],
            "type": "pubkey"
          },
          {
            "name": "offer",
            "docs": [
              "The public key of the offer this whitelist is associated with",
              "Links the whitelist to its specific offer"
            ],
            "type": "pubkey"
          },
          {
            "name": "takers",
            "docs": [
              "Vector of public keys representing addresses allowed to take the offer",
              "Only these addresses can execute trades against the offer when whitelist is enabled",
              "Empty vector means no takers are currently whitelisted"
            ],
            "type": {
              "vec": "pubkey"
            }
          }
        ]
      }
    },
    {
      "name": "whitelistConfig",
      "docs": [
        "Account structure controlling whether token mint whitelist is enforced",
        "Provides global toggle for whitelist functionality"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "requireWhitelist",
            "docs": [
              "Boolean flag indicating if whitelist checking is required",
              "true = only whitelisted token mints can be used",
              "false = any token mint can be used"
            ],
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "whitelistRequirementToggled",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "newStatus",
            "type": "bool"
          },
          {
            "name": "timestamp",
            "type": "i64"
          }
        ]
      }
    }
  ]
};
