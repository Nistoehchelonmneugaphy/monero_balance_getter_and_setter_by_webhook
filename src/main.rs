use serde_json::Value;
use std::time::Duration;

// const RPC_URL: &str = "http://127.0.0.1:38089/json_rpc";
// const WEBHOOK_URL: &str = "http://localhost:5555/webhook_and_api";

const VERSION: &str = "0.0.0.0.2";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("version: {}", VERSION);
    dotenvy::dotenv().ok();
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");

    let webhook_url = std::env::var("WEBHOOK_URL").expect("WEBHOOK_URL must be set");

    let webhook_action = std::env::var("WEBHOOK_ACTION").expect("WEBHOOK_ACTION must be set");
    let webhook_token = std::env::var("WEBHOOK_TOKEN").expect("WEBHOOK_TOKEN must be set");
    let webhook_send_interval_seconds = std::env::var("WEBHOOK_SEND_INTERVAL_SECONDS")
        .expect("WEBHOOK_SEND_INTERVAL_SECONDS must be set")
        .parse::<u64>()
        .expect("Expected WEBHOOK_SEND_INTERVAL_SECONDS to be parsable into u64");

    let cf_bypass_header_name =
        std::env::var("CF_BYPASS_HEADER_NAME").expect("CF_BYPASS_HEADER_NAME must be set");

    let cf_bypass_token = std::env::var("CF_BYPASS_TOKEN").expect("CF_BYPASS_TOKEN must be set");

    println!(
        "Env vars: RPC_URL={}\n WEBHOOK_URL={}\n WEBHOOK_TOKEN={}\n",
        rpc_url, webhook_url, webhook_token
    );

    let client = reqwest::Client::new();
    let mut last_unlocked_balance: u64 = 999999999999;
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
            println!(
                "RTC sent same value as last, not sending it to webhook so as not to send duplicates"
            )
        } else {
            last_unlocked_balance = unlocked;
            // -------------------------
            // 2. Send to webhook
            // -------------------------
            let form = [
                ("token", webhook_token.as_str()),
                ("action", webhook_action.as_str()),
                ("amount", &unlocked.to_string()),
            ];

            let resp = client
                .post(webhook_url.clone())
                .header(cf_bypass_header_name.clone(), cf_bypass_token.clone())
                .form(&form)
                .send()
                .await?;

            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();

            println!("Webhook response: {status} - {body}");
        }

        // -------------------------
        // 3. Wait 10 minutes
        // -------------------------
        tokio::time::sleep(Duration::from_secs(webhook_send_interval_seconds)).await;
    }
}
