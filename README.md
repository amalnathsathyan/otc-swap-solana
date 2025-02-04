# OTC Solana Swap

**OTC Solana Swap** is a decentralized application (dApp) built on the Solana blockchain that facilitates one-to-many Over-the-Counter (OTC) swaps. This platform allows makers to create offers by escrowing their tokens at a stated price, with a pre-determined whitelist of takers (buyers). Takers can purchase the required amount of tokens without needing to escrow their own tokens. Additionally, makers have the flexibility to update or cancel ongoing offers, manage the whitelist of allowed buyers, and more. The platform also includes admin functionalities to manage platform fees, token whitelists, and other configurations.

## Features

- **Maker Functions**:
  - **Create Offers**: Makers can create offers by escrowing their tokens and setting a price.
  - **Manage Whitelist**: Makers can add or remove buyers from the whitelist of allowed takers.
  - **Cancel Offers**: Makers can cancel ongoing offers if necessary.

- **Taker Functions**:
  - **Take Offers**: Takers can purchase tokens from existing offers without escrowing their own tokens.

- **Admin Functions**:
  - **Initialize Admin**: Admin can initialize the platform with fee settings, wallet addresses, and token whitelists.
  - **Manage Token Whitelist**: Admin can add or remove tokens from the platform's whitelist.
  - **Update Fees**: Admin can update the platform fee percentage and fee wallet address.
  - **Toggle Whitelist Requirement**: Admin can enable or disable the requirement for a whitelist.

## Contract Functions

### Admin Functions

- **`initialize_admin`**: Initializes the admin settings including fee percentage, fee wallet, whitelist requirement, and initial token whitelist.
- **`add_mints_to_whitelist`**: Adds new tokens to the platform's whitelist.
- **`remove_mints_from_whitelist`**: Removes tokens from the platform's whitelist.
- **`update_fee_percentage`**: Updates the platform fee percentage.
- **`update_fee_address`**: Updates the fee wallet address.
- **`toggle_require_whitelist`**: Toggles the requirement for a whitelist.

### Maker Functions

- **`create_offer_and_send_tokens_to_vault`**: Creates a new offer by escrowing tokens and setting the price, expected amount, and deadline.
- **`manage_whitelist`**: Manages the whitelist of allowed takers for a specific offer.
- **`cancel_offer`**: Cancels an ongoing offer.

### Taker Functions

- **`take_offer`**: Allows takers to purchase tokens from an existing offer.

## Usage

### Setting Up

1. **Initialize the Admin**:
   - Use the `initialize_admin` function to set up the platform with the desired fee structure, wallet addresses, and initial token whitelist.

2. **Create an Offer**:
   - Makers can create an offer using the `create_offer_and_send_tokens_to_vault` function, specifying the token amount, expected total amount, and deadline.

3. **Manage Whitelist**:
   - Makers can manage the whitelist of allowed takers using the `manage_whitelist` function.

4. **Take an Offer**:
   - Takers can purchase tokens from an offer using the `take_offer` function.

5. **Cancel an Offer**:
   - Makers can cancel an ongoing offer using the `cancel_offer` function.

### Admin Management

- **Update Fees**:
  - Admins can update the platform fee percentage and fee wallet address using the `update_fee_percentage` and `update_fee_address` functions.

- **Manage Token Whitelist**:
  - Admins can add or remove tokens from the whitelist using the `add_mints_to_whitelist` and `remove_mints_from_whitelist` functions.

- **Toggle Whitelist Requirement**:
  - Admins can enable or disable the whitelist requirement using the `toggle_require_whitelist` function.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome!

## Support

For support, please open an issue on the [GitHub repository](https://github.com/your-repo/otc-solana-swap) or contact @amalnathsathyan on Telegram
