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

#![no_main]

use std::io::Read;

use ethabi::{ethereum_types::U256, ParamType, Token};
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

fn main() {
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    let decoded_input =
        ethabi::decode_whole(&[ParamType::FixedBytes, ParamType::Bytes, ParamType::String], &input_bytes).unwrap();
    let query_id = decoded_input[0].clone().into_fixed_bytes();
    let query_data = decoded_input[1].clone().into_bytes();
    let validation_data = decoded_input[2].clone().into_string();

    let decoded_query_data: Vec<Token> =
        ethabi::decode(&[ParamType::FixedBytes, ParamType::String, ParamType::String], &query_data).unwrap();
    let owner = decoded_query_data[0].clone().into_fixed_bytes().unwrap();
    let file_id = decoded_query_data[1].clone().into_string().unwrap();
    let commit_id = decoded_query_data[2].clone().into_string().unwrap();

    let events: Vec<dataverse_ceramic::Event> = serde_json.from_str(validation_data)?;
    let state = dataverse_ceramic::StreamState::make(3, events).await;
    let result = match state {
        Ok(_) => true,
        Err(_) => false
    };

    // // Commit the journal that will be received by the application contract.
    // // Encoded types should match the args expected by the application callback.
    env::commit_slice(&ethabi::encode(&[
        Token::Address(owner),
        Token::String(file_id),
        Token::String(commit_id),
        Token::Bool(result),
    ]));
}
