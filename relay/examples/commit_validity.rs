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
use ceramic_core::StreamId;
use clap::Parser;
use dataverse_ceramic::network::Network;
use dataverse_ceramic::{EventsLoader, Ceramic};
use ethabi::{ParamType, Token};
use ethers::prelude::*;
use ethers::{providers::Provider, types::Address};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    #[arg(
        long,
        env,
        // default_value = "wss://polygon-mumbai.g.alchemy.com/v2/xVWk2q1-WpFBPKJuvCAO1sXLlfCKqfaI"
    )]
    address: Address,

    #[arg(
        long,
        env,
        // default_value = "wss://polygon-mumbai.g.alchemy.com/v2/xVWk2q1-WpFBPKJuvCAO1sXLlfCKqfaI"
    )]
    rpc_url: String,

    /// Bonsai Relay API URL.
    #[arg(long, env, default_value = "http://localhost:8080")]
    bonsai_relay_api_url: String,

    #[arg(long, env)]
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
            event QuerySent(bytes32 queryId, bytes32 imageId, bytes queryData, address callbackAddr, bytes4 callbackFunc, uint64 gasLimit)
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
        let query_id: [u8; 32] = evt.query_id;
        let image_id: [u8; 32] = evt.image_id;
        let query_data: Bytes = evt.query_data;
        let callback_addr: Address = evt.callback_addr;
        let callback_func: [u8; 4] = evt.callback_func;
        let gas_limit: u64 = evt.gas_limit;

        let decoded_data: Vec<Token> = ethabi::decode(
            &[ParamType::Address, ParamType::String, ParamType::String],
            &query_data,
        )
        .unwrap();
        // let owner = decoded_data[0].clone().into_string().unwrap();
        let file_id = decoded_data[1].clone().into_string().unwrap();
        let commit_id = decoded_data[2].clone().into_string().unwrap();

        let payload  = get_payload(StreamId::from_str(file_id.as_str())?, commit_id).await.unwrap();
        let payload_data = serde_json::to_string(&payload)?;
        println!("payload: {}", payload_data);

        let input_params = vec![Token::FixedBytes(query_id.to_vec()), Token::Bytes(query_data.to_vec()), Token::String(payload_data)];
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

pub async fn get_payload(
    file_id: StreamId,
    _commit_id: String,
) -> anyhow::Result<Vec<dataverse_ceramic::event::Event>> {
    let client = dataverse_ceramic::http::Client::new();
    let ceramic = Ceramic{ endpoint: "https://dataverseceramicdaemon.com".into(), network: Network::Mainnet };
    let events =client.load_events(&ceramic, &file_id, None).await?;

    return Ok(events);
}
