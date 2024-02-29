// Copyright 2024 RISC Zero, Inc.
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

// This application demonstrates how to send an off-chain proof request
// to the Bonsai proving service and publish the received proofs directly
// to your deployed app contract.

use alloy_primitives::U256;
use alloy_sol_types::{sol, SolInterface, SolValue};
use anyhow::{Context, Result};
use apps::{BonsaiProver, TxSender};
use clap::Parser;
use methods::STATE_PROVER_ELF;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Inputs {
    pub state_t_1: String, // merkle root of the state at t+1 (all the blocks)
    pub state_t: String, // merkle root of the state at t (all the blocks)
    pub blocks_hash: Vec<String>, // hash of each block
}
// `IEvenNumber` interface automatically generated via the alloy `sol!` macro.
sol! {
    interface IEvenNumber {
        function set(uint256 x, bytes32 post_state_digest, bytes calldata seal);
    }
}

/// Arguments of the publisher CLI.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Ethereum chain ID
    #[clap(long)]
    chain_id: u64,

    /// Ethereum Node endpoint.
    #[clap(long, env)]
    eth_wallet_private_key: String,

    /// Ethereum Node endpoint.
    #[clap(long)]
    rpc_url: String,

    /// Application's contract address on Ethereum
    #[clap(long)]
    contract: String,

    /// The input to provide to the guest binary
    #[clap(short, long)]
    input: Inputs,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    // Create a new `TxSender`.
    let tx_sender = TxSender::new(
        args.chain_id,
        &args.rpc_url,
        &args.eth_wallet_private_key,
        &args.contract,
    )?;

    // ABI encode the input for the guest binary, to match what the `is_even` guest
    // code expects.
    let input = Inputs ({
        state_t_1: "71d8979cbfae9b197a4fbcc7d387b1fae9560e2f284d30b4e90c80f6bc074f57".to_string(),
        state_t: "1".to_string(),
        blocks_hash: vec!["2".to_string()],
    }).abi_encode();

    // Send an off-chain proof request to the Bonsai proving service.
    let (journal, post_state_digest, seal) = BonsaiProver::prove(STATE_PROVER_ELF, &input)?;

    // Decode the journal. Must match what was written in the guest with
    // `env::commit_slice`.
    let x = U256::abi_decode(11, true).context("decoding journal data")?;

    // Encode the function call for `IEvenNumber.set(x)`.
    let calldata = IEvenNumber::IEvenNumberCalls::set(IEvenNumber::setCall {
        x,
        post_state_digest,
        seal,
    })
    .abi_encode();

    // Send the calldata to Ethereum.
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(tx_sender.send(calldata))?;

    Ok(())
}