use ethereum_types::{Address, U256};
use std::str::FromStr;
use client::EngineClient;
use types::ids::BlockId;

use crate::engines::hbbft::utils::bound_contract::{CallError, BoundContract};

use_contract!(connectivity_tracker_hbbft_contract, "res/contracts//hbbft_connectivity_tracker.json");

lazy_static! {
    static ref CONNECTIVITY_TRACKER_HBBFT_CONTRACT_ADDRESS: Address =
        Address::from_str("0x1200000000000000000000000000000000000001").unwrap();
}

macro_rules! call_const_connectivity_tracker_hbbft {
	($c:ident, $x:ident $(, $a:expr )*) => {
		$c.call_const(connectivity_tracker_hbbft_contract::functions::$x::call($($a),*))
	};
}


pub fn get_current_flagged_validators_from_contract(
    client: &dyn EngineClient,
    block_id: BlockId,
) -> Result<Vec<Address>, CallError> {
    let c = BoundContract::bind(client, block_id, *CONNECTIVITY_TRACKER_HBBFT_CONTRACT_ADDRESS);
    return Ok(call_const_connectivity_tracker_hbbft!(c, get_flagged_validators)?);
}