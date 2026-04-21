use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;


const MAIN_APP_WEBHOOK: &str = "http://localhost:5555/webhook_and_api";
const ACTION: &str = "set_website_fund_xmr";
const TOKEN: &str = "123";

#[derive(Serialize)]
struct RPCRequest<'a> {
    jsonrpc: &'a str,
    id: &'a str,
    method: &'a str,
}

#[derive(Deserialize)]
struct RPCResponse {
    result: RPCResult,
}

#[derive(Deserialize)]
struct RPCResult {
    _balance: u64,
    unlocked_balance: u64,
}

#[tokio::main]
async fn main() {
    let client = Client::new();

    loop {
        match get_unlocked_balance(&client).await {
            Ok(amount) => {
                println!("Unlocked balance: {}", amount);

                if let Err(e) = send_to_main_app_webhook(&client, amount).await {
                    eprintln!("Webhook error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("RPC error: {}", e);
            }
        }


        tokio::time::sleep(Duration::from_mins(10)).await;
    }
}

async fn get_unlocked_balance(client: &Client) -> Result<u64, reqwest::Error> {
    let request = RPCRequest {
        jsonrpc: "2.0",
        id: "0",
        method: "get_balance", 
    };

    let response: RPCResponse = client
        .post("http://127.0.0.1:38089/json_rpc")
        .json(&request)
        .send()
        .await?
        .json()
        .await?;
    Ok(response.result.unlocked_balance)
}

async fn send_to_main_app_webhook(client: &Client, amount: u64) -> Result<(), reqwest::Error> {
    client
        .post(MAIN_APP_WEBHOOK)
        .form(&[
            ("token", TOKEN),
            ("action", ACTION),
            ("amount", &amount.to_string()),
        ])
        .send()
        .await?;

    Ok(())
}
