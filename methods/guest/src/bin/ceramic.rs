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

fn fibonacci(n: U256) -> U256 {
    let (mut prev, mut curr) = (U256::one(), U256::one());
    for _ in 2..=n.as_u32() {
        (prev, curr) = (curr, prev + curr);
    }
    curr
}

fn main() {
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    let decoded_input = ethabi::decode_whole(&[ParamType::Bytes, ParamType::String], &input_bytes).unwrap();
    let request_data = decoded_input[0].clone().into_bytes();
    let validation_data = decoded_input[0].clone().into_string();

    let decoded_request_data: Vec<Token> = ethabi::decode(
        &[ParamType::FixedBytes(16), ParamType::String],
        &request_data,
    )
    .unwrap();
    let dapp_id = decoded_request_data[0].clone().into_fixed_bytes().unwrap();
    let file_id = decoded_request_data[1].clone().into_string().unwrap();

    // TODO: validate

    let result: Vec<u8> = vec![1];

    // // Commit the journal that will be received by the application contract.
    // // Encoded types should match the args expected by the application callback.
    env::commit_slice(&ethabi::encode(&[Token::FixedBytes(dapp_id), Token::String(file_id), Token::Bytes(result)]));
}
