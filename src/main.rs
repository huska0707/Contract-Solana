use {
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        instruction::Instruction,
        pubkey::Pubkey,
        signer::{
            keypair::{read_keypair_file, write_keypair_file, Keypair},
            Signer,
        },
        transaction::Transaction,
        {borsh::try_from_slice_unchecked, program_pack::Pack},
    },
    spl_token::state::{Account, Mint},
    spl_token_metadata::{
        instruction::{create_master_edition, create_metadata_accounts},
        state::{Metadata, EDITION, PREFIX},
    },
    std::{io, io::Write, thread, time},
};

const CLIENT_URL: &'static str = "https://api.devnet.solana.com";
const WALLET_FILE_PATH: &'static str = "wallet.keypair";

fn get_wallet() -> Keypair {
    let wallet_keypair: Keypair = if let Ok(keypair) = read_keypair_file(WALLET_FILE_PATH) {
        keypair
    } else {
        let new_keypair = Keypair::new();
        write_keypair_file(&new_keypair, WALLET_FILE_PATH).unwrap();
        new_keypair
    };

    return wallet_keypair;
}

fn mint_nft(
    wallet_keypair: &Keypair,
    mint_account_pubkey: &Pubkey,
    token_account_pubkey: &Pubkey,
    client: &RpcClient,
) {
    let wallet_pubkey = wallet_keypair.pubkey();

    let mint_to_instruction: Instruction = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint_account_pubkey,
        &token_account_pubkey,
        &wallet_pubkey,
        &[&wallet_pubkey],
        1,
    )
    .unwrap();
    let (recent_blockhash, _fee_calculator) = client.get_recent_blockhash().unwrap();
    let transaction: Transaction = Transaction::new_signed_with_payer(
        &vec![mint_to_instruction],
        Some(&wallet_pubkey),
        &[wallet_keypair],
        recent_blockhash,
    );
    let result = client.send_and_confirm_transaction_with_spinner(&transaction);
    if result.is_ok() {
        println!("Successfully Minted NFT to : {:?}", wallet_pubkey);

        upgrade_to_master_edition(
            &wallet_keypair,
            &create_metadata_account(&wallet_keypair, &mint_account_pubkey, &client),
            &mint_account_pubkey,
            &client,
        );
    };
}

fn main() {
    // Get our Wallet KeyPair
    let wallet_keypair = get_wallet();
    let wallet_pubkey: Pubkey = wallet_keypair.pubkey();

    let program_key = spl_token_metadata::id();
    println!("{:?}", program_key);

    // Connect to the Solana Client and pull our wallet balance
    let client = RpcClient::new(CLIENT_URL.to_string());
    let wallet_balance = client.get_balance(&wallet_pubkey).unwrap();

    println!("Wallet Pubkey: {}", wallet_pubkey);
    println!("Wallet Balance: {}", wallet_balance);

    // Airdrop funds if our wallet is empty
    if wallet_balance == 0 {
        let result = client.request_airdrop(&wallet_keypair.pubkey(), 10_000_000_000);

        if result.is_ok() {
            print!("Airdropping funds to {:?}", wallet_pubkey);
            io::stdout().flush().unwrap();
            while client.get_balance(&wallet_pubkey).unwrap() == 0 {
                print!(".");
                io::stdout().flush().unwrap();
                let one_second = time::Duration::from_millis(1000);
                thread::sleep(one_second);
            }
            println!("");
        } else {
            println!("Failed to Airdrop funds. Try again later.");
            return;
        }
    }

    // Create the required prelim accounts
    let mint_account_pubkey = create_mint_account(&wallet_keypair, &client);
    let token_account_pubkey = create_token_account(&wallet_keypair, &mint_account_pubkey, &client);
    // Create the NFT, including the Metadata associated with it
    mint_nft(
        &wallet_keypair,
        &mint_account_pubkey,
        &token_account_pubkey,
        &client,
    );
    return;
}