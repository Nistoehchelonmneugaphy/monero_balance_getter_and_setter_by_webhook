use serde_json::Value;
use std::{time::Duration};

// const RPC_URL: &str = "http://127.0.0.1:38089/json_rpc";
// const WEBHOOK_URL: &str = "http://localhost:5555/webhook_and_api";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
       let rpc_url = std::env::var("RPC_URL")
        .expect("RPC_URL must be set");

    let webhook_url = std::env::var("WEBHOOK_URL")
        .expect("WEBHOOK_URL must be set");
    let webhook_token = std::env::var("WEBHOOK_TOKEN")
        .expect("WEBHOOK_TOKEN must be set");

    println!("Env vars: RPC_URL={} WEBHOOK_URL={} WEBHOOK_TOKEN={}", rpc_url, webhook_url, webhook_token);
    
    let client = reqwest::Client::new();
    let mut last_unlocked_balance: u64 = 0;
    loop {
        // -------------------------
        // 1. Get wallet balance
        // -------------------------
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_balance"
        });

        let res: Value = client
            .post(rpc_url.clone())
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        let result = &res["result"];

        let balance = result["balance"].as_u64().unwrap_or(0);
       //let unlocked = result["unlocked_balance"].as_u64().unwrap_or(0);
        let unlocked = result["balance"].as_u64().unwrap_or(0);

        println!("Balance: {balance} | Unlocked: {unlocked}");

        if last_unlocked_balance == unlocked {
            println!("RTC sent same value as last, not sending it to webhook so as not to send duplicates")
        } else {
            last_unlocked_balance = unlocked;
            // -------------------------
            // 2. Send to webhook
            // -------------------------
            let form = [
                ("token", webhook_token.as_str()),
                ("action", "set_website_fund_xmr"),
                ("amount", &unlocked.to_string()),
            ];

            let resp = client.post(webhook_url.clone()).form(&form).send().await?;

            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();

            println!("Webhook response: {status} - {body}");
        }

        // -------------------------
        // 3. Wait 10 minutes
        // -------------------------
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
