use client::EngineClient;
use types::transaction::SignedTransaction;
use engines::hbbft::utils::bound_contract::{BoundContract};
use ethereum_types::{Address, U256};
use std::str::FromStr;
use types::ids::BlockId;
use crate::engines::EngineError;
use crate::error::Error;

use crate::client::{BlockChainClient, traits::TransactionRequest};

use_contract!(random_hbbft_contract, "res/contracts/random_hbbft.json");

lazy_static! {
    static ref RANDOM_HBBFT_CONTRACT_ADDRESS: Address =
        Address::from_str("3000000000000000000000000000000000000001").unwrap();
}

macro_rules! call_const_staking {
    ($c:ident, $x:ident $(, $a:expr )*) => {
        $c.call_const(random_hbbft_contract::functions::$x::call($($a),*))
    };
}

pub fn set_current_seed_tx(client: &dyn EngineClient, random_value: &U256) -> Result<SignedTransaction, Error> {
    
    warn!("set_current_seed_tx");
    //call_const_staking!(c, staking_epoch)
    let decoder = random_hbbft_contract::functions::set_current_seed::call(random_value);
    warn!("call data: {:?}", decoder.0);
    
    match client.as_full_client() {
        Some(full_client) => {
            let tx_request = TransactionRequest::call(
                *RANDOM_HBBFT_CONTRACT_ADDRESS,
                decoder.0,
            );
            match full_client.create_transaction(tx_request) {
                Ok(tx) => Ok(tx),
                Err(e) => {
                    error!(target: "consensus", "Error creating random_value transaction: {:?}", e);
                    return Err(e.into());
                }
            }
        },
        None => {
            warn!(target: "consensus", "Error creating rng seed transaction: full client unavailable");
            return Err(EngineError::Custom("Error creating rng seed transaction: full client unavailable".to_string()).into());
        }
    }
}



// A client for the block reward contract.
// #[derive(PartialEq, Debug)]
// pub struct RandomHbbftContract {
//     kind: SystemOrCodeCallKind,
// }



// impl RandomHbbftContract {
//   /// Create a new block reward contract client targeting the system call kind.
//   pub fn new(kind: SystemOrCodeCallKind) -> RandomHbbftContract {
//     RandomHbbftContract { kind }
//   }

//   /// Create a new block reward contract client targeting the contract address.
//   pub fn new_from_address(address: Address) -> BlockRewardContract {
//       Self::new(SystemOrCodeCallKind::Address(address))
//   }
// }