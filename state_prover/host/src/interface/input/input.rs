use crate::interface::{UTXO,PendingTransaction};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Inputs {
    pub state_t_1: String, //merkle root of the utxo set at time t+1
    pub state_t: Vec<UTXO>, // list of All the utxo
    pub txs: Vec<PendingTransaction>, //list of all the txs
} 