use ethereum_types::{Address, U256};
use std::str::FromStr;
use client::EngineClient;
use types::ids::BlockId;

use crate::{engines::hbbft::utils::bound_contract::{CallError, BoundContract}, client::{traits::TransactionRequest, BlockChainClient}};

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


pub fn report_missing_connectivity(
    client: &dyn EngineClient,
    full_client: &dyn BlockChainClient,
    missing_validator: &Address,
    signing_address: &Address,
) -> bool  {

    let (block_number, block_hash) =
    if let Some(block_number) = client.block_number(BlockId::Latest) {
        if let Some(header) = client.block_header(BlockId::Number(block_number)) {
            let hash = header.hash();
            (block_number, hash)
        } else {
            warn!(target:"engine", "could not get block number for block: {block_number}");
            return false;
        }
    } else {
        warn!(target:"engine", "report_missing_connectivity: could not get latest block.");
        return false;
    };

    let send_data = connectivity_tracker_hbbft_contract::functions::report_missing_connectivity::call(
        *missing_validator,
        block_number,
        block_hash
    );

    let nonce = full_client.next_nonce(signing_address);

    let transaction = TransactionRequest::call(*CONNECTIVITY_TRACKER_HBBFT_CONTRACT_ADDRESS, send_data.0)
        .gas(U256::from(200_000))
        .nonce(nonce);

    info!(target:"consensus", "sending report_missing_connectivity for with nonce: {nonce}, missing: {:?} ", missing_validator);
    if let Err(e) = full_client.transact_silently(transaction) {
        warn!(target:"consensus", "could not report_missing_connectivity {e:?}");
        return false;
    }
    return true;
    
}