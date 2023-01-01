use ethereum_types::{Address, U256};
use std::str::FromStr;


use_contract!(random_hbbft_contract, "res/contracts/random_hbbft.json");

lazy_static! {
    static ref RANDOM_HBBFT_CONTRACT_ADDRESS: Address =
        Address::from_str("3000000000000000000000000000000000000001").unwrap();
}

pub fn set_current_seed_tx_raw(random_value: &U256) -> (Address, Vec<u8>) {
    
    warn!("set_current_seed_tx");
    //call_const_staking!(c, staking_epoch)
    let decoder = random_hbbft_contract::functions::set_current_seed::call(random_value);
    warn!("call data: {:?}", decoder.0);
    return (RANDOM_HBBFT_CONTRACT_ADDRESS.clone(), decoder.0);

}