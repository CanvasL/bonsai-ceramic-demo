// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::Context;
use bonsai_ethereum_relay::sdk::client::{CallbackRequest, Client};
use clap::Parser;
use ethabi::{ParamType, Token};
use ethers::prelude::*;
use ethers::{providers::Provider, types::Address};
use std::sync::Arc;
use tokio::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    #[arg(
        long,
        env,
        default_value = "0x9a9964D9E39D8891a95490C1e1159C4D6eC77980"
    )]
    address: Address,

    #[arg(
        long,
        env,
        default_value = "wss://polygon-mumbai.g.alchemy.com/v2/xVWk2q1-WpFBPKJuvCAO1sXLlfCKqfaI"
    )]
    rpc_url: String,

    /// Bonsai Relay API URL.
    #[arg(long, env, default_value = "https://api.bonsai.xyz/")]
    bonsai_relay_api_url: String,

    #[arg(long, env, default_value = "6Y7lEBg5sI7NF1ih0CtpY6zpOzn5TsB09ToWpr4t")]
    bonsai_api_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    println!("address: {}", args.address);
    println!("rpc_url: {}", args.rpc_url);
    println!("bonsai_relay_api_url: {}", args.bonsai_relay_api_url);
    println!("bonsai_api_key: {}", args.bonsai_api_key);
    // initialize a relay client
    let relay_client = Client::from_parts(
        args.bonsai_relay_api_url.clone(),
        args.bonsai_api_key.clone(),
    )
    .context("Failed to initialize the relay client")?;

    abigen!(
        Consumer,
        r#"[
            event QuerySent(bytes32 imageId, bytes requestData, address callbackAddr, bytes4 callbackFunc, uint64 gasLimit)
        ]"#
    );

    let provider = Provider::<Ws>::connect(args.rpc_url.as_str()).await?;
    let eth_client = Arc::new(provider);

    let current_block_number = eth_client.get_block_number().await?;

    let contract = Consumer::new(args.address, eth_client);
    let events = contract
        .event::<QuerySentFilter>()
        .from_block(current_block_number);
    let mut stream = events.subscribe().await?;

    while let Some(Ok(evt)) = stream.next().await {
        println!("QuerySent event: {evt:?}");
        let image_id: [u8; 32] = evt.image_id;
        let request_data: Bytes = evt.request_data;
        let callback_addr: Address = evt.callback_addr;
        let callback_func: [u8; 4] = evt.callback_func;
        let gas_limit: u64 = evt.gas_limit;

        let decoded_data: Vec<Token> = ethabi::decode(
            &[ParamType::FixedBytes(16), ParamType::String],
            &request_data,
        )
        .unwrap();
        let dapp_id = decoded_data[0].clone().into_fixed_bytes().unwrap();
        let file_id = decoded_data[1].clone().into_string().unwrap();

        let validation_data  = fetch_ceramic_validation_data(dapp_id, file_id).await.unwrap();

        let input_params = vec![Token::Bytes(request_data.to_vec()), Token::String(validation_data)];
        let input = ethabi::encode(&input_params);

        let request = CallbackRequest {
            callback_contract: callback_addr,
            function_selector: callback_func,
            gas_limit,
            image_id,
            input,
        };

        relay_client
        .callback_request(request)
        .await
        .context("Callback request failed")?;
    };

    Ok(())
}

async fn fetch_ceramic_validation_data(
    dapp_id: Vec<u8>,
    file_id: String,
) -> Result<String, Box<dyn std::error::Error + 'static>> {
    // TODO: network-crates is unavailable now
    println!("dapp_id:{:?}\nfile_id:{}", dapp_id, file_id);
    let raw_content = fs::read("relay/examples/validation.json").await?;
    let content = String::from_utf8(raw_content)?;
    Ok(content)
}
