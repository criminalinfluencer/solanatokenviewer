use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_account_decoder::{UiAccountEncoding, parse_token_account_data};
use solana_sdk::account::ReadableAccount;
use std::str::FromStr;
use std::fs::OpenOptions;
use std::io::prelude::*;
use eframe::{egui, epi};
use reqwest::blocking::Client;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let app = TokenApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

struct TokenApp {
    tokens: Vec<TokenInfo>,
}

impl Default for TokenApp {
    fn default() -> Self {
        let tokens = fetch_tokens();
        TokenApp { tokens }
    }
}

struct TokenInfo {
    pubkey: String,
    mint: String,
    owner: String,
    amount: String,
    price: f64,
    market_cap: f64,
    volume_24h: f64,
}

impl epi::App for TokenApp {
    fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tokens on Solana");
            for token in &self.tokens {
                ui.label(format!("Token Account Pubkey: {}", token.pubkey));
                ui.label(format!("Mint: {}", token.mint));
                ui.label(format!("Owner: {}", token.owner));
                ui.label(format!("Amount: {}", token.amount));
                ui.label(format!("Price: ${:.2}", token.price));
                ui.label(format!("Market Cap: ${:.2}", token.market_cap));
                ui.label(format!("24h Volume: ${:.2}", token.volume_24h));
                ui.separator();
            }
        });
    }

    fn name(&self) -> &str {
        "Solana Token Viewer"
    }
}

#[derive(Deserialize)]
struct CoinGeckoResponse {
    market_data: MarketData,
}

#[derive(Deserialize)]
struct MarketData {
    current_price: std::collections::HashMap<String, f64>,
    market_cap: std::collections::HashMap<String, f64>,
    total_volume: std::collections::HashMap<String, f64>,
}

fn fetch_tokens() -> Vec<TokenInfo> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let client = RpcClient::new(rpc_url.to_string());

    let token_program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let accounts = client.get_program_accounts(&token_program_id).expect("Falha ao obter contas do programa");

    let mut tokens = Vec::new();
    let mut log_data = String::new();
    let http_client = Client::new();

    for (pubkey, account) in accounts {
        if let Ok(parsed_account) = parse_token_account_data(account.data()) {
            let token_id = parsed_account.mint.to_string();
            let url = format!("https://api.coingecko.com/api/v3/coins/{}", token_id);

            if let Ok(response) = http_client.get(&url).send() {
                if let Ok(coingecko_data) = response.json::<CoinGeckoResponse>() {
                    let price = coingecko_data.market_data.current_price.get("usd").cloned().unwrap_or(0.0);
                    let market_cap = coingecko_data.market_data.market_cap.get("usd").cloned().unwrap_or(0.0);
                    let volume_24h = coingecko_data.market_data.total_volume.get("usd").cloned().unwrap_or(0.0);

                    let token_info = TokenInfo {
                        pubkey: pubkey.to_string(),
                        mint: parsed_account.mint.to_string(),
                        owner: parsed_account.owner.to_string(),
                        amount: parsed_account.amount.to_string(),
                        price,
                        market_cap,
                        volume_24h,
                    };

                    log_data.push_str(&format!(
                        "Token Account Pubkey: {}\nMint: {}\nOwner: {}\nAmount: {}\nPrice: ${:.2}\nMarket Cap: ${:.2}\n24h Volume: ${:.2}\n====================\n",
                        token_info.pubkey, token_info.mint, token_info.owner, token_info.amount, token_info.price, token_info.market_cap, token_info.volume_24h
                    ));

                    tokens.push(token_info);
                }
            }
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("token_log.txt")
        .expect("Não foi possível abrir o arquivo de log");
    file.write_all(log_data.as_bytes())
        .expect("Não foi possível escrever no arquivo de log");

    tokens
}
