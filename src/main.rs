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

    return;
}