use crate::{client::EngineClient, types::ids::BlockId};
use ethereum_types::{Address, U256};
use std::str::FromStr;

use crate::engines::hbbft::utils::bound_contract::{BoundContract, CallError};

use_contract!(diamond_dao_contract, "res/contracts/diamond_dao.json");

lazy_static! {
    static ref DIAMOND_DAO_CONTRACT_ADDRESS: Address =
        Address::from_str("DA0da0da0Da0Da0Da0DA00DA0da0da0DA0DA0dA0").unwrap();
}

macro_rules! call_const_diamond_dao {
	($c:ident, $x:ident $(, $a:expr )*) => {
		$c.call_const(diamond_dao_contract::functions::$x::call($($a),*))
	};
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ProposalState {
    Created,
    Canceled,
    Active,
    VotingFinished,
    Accepted,
    Declined,
    Executed,
    Unknown = 255,
}

impl ProposalState {
    pub fn from_u64(value: u64) -> Self {
        match value {
            0 => Self::Created,
            1 => Self::Canceled,
            2 => Self::Active,
            3 => Self::VotingFinished,
            4 => Self::Accepted,
            5 => Self::Declined,
            6 => Self::Executed,
            _ => Self::Unknown,
        }
    }
}

pub fn get_proposal_state(
    client: &dyn EngineClient,
    block_id: BlockId,
    proposal_id: U256,
) -> Result<ProposalState, CallError> {
    let c = BoundContract::bind(client, block_id, *DIAMOND_DAO_CONTRACT_ADDRESS);
    let (_, _, state, ..) = call_const_diamond_dao!(c, proposals, proposal_id)?;

    Ok(ProposalState::from_u64(state.low_u64()))
}
