use serde_json::Value;
use std::time::Duration;

const RPC_URL: &str = "http://127.0.0.1:38089/json_rpc";
const WEBHOOK_URL: &str = "http://localhost:5555/webhook_and_api";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            .post(RPC_URL)
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        let result = &res["result"];

        let balance = result["balance"].as_u64().unwrap_or(0);
        let unlocked = result["unlocked_balance"].as_u64().unwrap_or(0);

        println!("Balance: {balance} | Unlocked: {unlocked}");

       if last_unlocked_balance != unlocked {
         // -------------------------
        // 2. Send to webhook
        // -------------------------
        let form = [
            ("token", "123"),
            ("action", "set_website_fund_xmr"),
            ("amount", &unlocked.to_string()),
        ];

        let resp = client
            .post(WEBHOOK_URL)
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
        tokio::time::sleep(Duration::from_secs(600)).await;
    }
}