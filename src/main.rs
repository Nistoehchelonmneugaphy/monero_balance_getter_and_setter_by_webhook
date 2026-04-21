use serde_json::Value;
use std::time::Duration;

const RPC_URL: &str = "http://127.0.0.1:38089/json_rpc";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    loop {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": "get_balance"
        });

        let res: Value = client
            .post(RPC_URL)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        let result = &res["result"];

        let balance = result["balance"].as_u64().unwrap_or(0);
        let unlocked = result["unlocked_balance"].as_u64().unwrap_or(0);

        println!(
            "Balance: {} | Unlocked balance: {}",
            balance, unlocked
        );

        tokio::time::sleep(Duration::from_secs(600)).await;
    }
}