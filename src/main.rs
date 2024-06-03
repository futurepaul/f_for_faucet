use reqwest::Client;
use serde_json::json;
use std::env;

const HELP_MESSAGE: &str = r#"
You need at least one argument. So like a number (to get an invoice for that amount), an invoice string (to be paid), or an address + number (to be paid).

Special commands:
  - "help" to see this message
  - "a" to get an address
  - "ln" to get the faucet lightning address
  - "f" to get the federation invite code
"#;

async fn get_invoice(client: &Client, amount: u64) -> Result<String, reqwest::Error> {
    let res = client
        .post("https://faucet.mutinynet.com/api/bolt11")
        .json(&json!({ "amount_sats": amount }))
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    Ok(json["bolt11"].as_str().unwrap_or("").to_string())
}

async fn pay_invoice(client: &Client, invoice: &str) -> Result<(), reqwest::Error> {
    let res = client
        .post("https://faucet.mutinynet.com/api/lightning")
        .json(&json!({ "bolt11": invoice }))
        .send()
        .await?;

    println!("Status: {}", res.status());
    Ok(())
}

async fn pay_address(client: &Client, address: &str, amount: u64) -> Result<(), reqwest::Error> {
    let res = client
        .post("https://faucet.mutinynet.com/api/onchain")
        .json(&json!({ "address": address, "sats": amount }))
        .send()
        .await?;

    println!("Status: {}", res.status());
    Ok(())
}

fn handle_special_commands(command: &str) -> bool {
    match command {
        "a" => {
            println!("tb1qd28npep0s8frcm3y7dxqajkcy2m40eysplyr9v");
            true
        }
        "f" => {
            println!("fed11qgqzc2nhwden5te0vejkg6tdd9h8gepwvejkg6tdd9h8garhduhx6at5d9h8jmn9wshxxmmd9uqqzgxg6s3evnr6m9zdxr6hxkdkukexpcs3mn7mj3g5pc5dfh63l4tj6g9zk4er");
            true
        }
        "ln" => {
            println!("refund@lnurl-staging.mutinywallet.com");
            true
        }
        "help" => {
            println!("{}", HELP_MESSAGE);
            true
        }
        _ => false,
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let client = Client::new();

    match args.len() {
        // Single argument like a number or an invoice
        2 => {
            let command = &args[1];
            let handled = handle_special_commands(command);
            if handled {
                return;
            }
            if let Ok(amount) = command.parse::<u64>() {
                match get_invoice(&client, amount).await {
                    Ok(bolt11) => println!("{}", bolt11),
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else {
                if let Err(e) = pay_invoice(&client, command).await {
                    eprintln!("Error: {}", e);
                }
            }
        }
        // Dual argument, like an address + an amount
        3 => {
            let arg1 = &args[1];
            let arg2 = &args[2];
            if let (Ok(amount), Err(_)) = (arg1.parse::<u64>(), arg2.parse::<u64>()) {
                if let Err(e) = pay_address(&client, arg2, amount).await {
                    eprintln!("Error: {}", e);
                }
            } else if let (Err(_), Ok(amount)) = (arg1.parse::<u64>(), arg2.parse::<u64>()) {
                if let Err(e) = pay_address(&client, arg1, amount).await {
                    eprintln!("Error: {}", e);
                }
            }
        }
        _ => eprintln!("{}", HELP_MESSAGE),
    }
}
