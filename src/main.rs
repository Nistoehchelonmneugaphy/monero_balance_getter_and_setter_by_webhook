use serde_json::Value;
use std::{path::PathBuf, time::Duration};

// const RPC_URL: &str = "http://127.0.0.1:38089/json_rpc";
// const WEBHOOK_URL: &str = "http://localhost:5555/webhook_and_api";

const VERSION: &str = git_version::git_version!(args = ["--tags", "--abbrev=0"]);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("version: {}", VERSION);
    dotenvy::dotenv().ok();
    let app_config = read_config(PathBuf::from(".env"));
    let AppConfig {
        rpc_url,
        webhook_target_domain,
        webhook_secret_path_prefix,
        webhook_path,
        webhook_action,
        webhook_token,
        webhook_send_interval_seconds,
        cf_bypass_header_name,
        cf_bypass_token,
    } = app_config;
    let webhook_full_url = format!("{webhook_target_domain}/{webhook_secret_path_prefix}/{webhook_path}/{webhook_action}");
    println!(
        "RPC_URL={rpc_url}\nWEBHOOK_FULL_URL={webhook_full_url}\n",
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
                .post(webhook_full_url.clone())
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

#[derive(Debug, Clone, serde::Deserialize)]
struct AppConfig {
    rpc_url: String,
    webhook_target_domain: String,
    webhook_secret_path_prefix: String,
    webhook_path: String,
    webhook_action: String,
    webhook_token: String,
    webhook_send_interval_seconds: u64,
    cf_bypass_header_name: String,
    cf_bypass_token: String,
}

fn read_config(config_file_path: PathBuf) -> AppConfig {
    dotenvy::from_path(config_file_path).ok();

    config::Config::builder()
        .add_source(
            config::Environment::default(), //.separator("__")  // default is ".". I guess that's fine?
        )
        .build()
        .expect("Failed to build config")
        .try_deserialize::<AppConfig>()
        .expect("Invalid or missing config values")
}
