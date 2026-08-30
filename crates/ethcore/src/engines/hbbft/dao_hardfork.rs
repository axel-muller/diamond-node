use crate::block::ExecutedBlock;
use crate::engines::hbbft::contracts::diamond_dao::ProposalState;
use crate::state::CleanupMode;
use crate::{
    client::EngineClient, engines::hbbft::contracts::diamond_dao::get_proposal_state, error::Error,
    types::ids::BlockId,
};
use ethjson::spec::hbbft::HbbftDaoHardforkTransfer;

/// Applies DAO voting baced hardfork at this block.
pub fn apply_dao_hardfork(
    forks: &[HbbftDaoHardforkTransfer],
    client: &dyn EngineClient,
    block: &mut ExecutedBlock,
) -> Result<(), Error> {
    for fork in forks {
        apply_single(fork, client, block)?;
    }
    Ok(())
}

fn apply_single(
    fork: &HbbftDaoHardforkTransfer,
    client: &dyn EngineClient,
    block: &mut ExecutedBlock,
) -> Result<(), Error> {
    if block.header.timestamp() < fork.execution_timestamp {
        return Ok(());
    }

    let parent_hash = *block.header.parent_hash();
    let parent = match client.block_header(BlockId::Hash(parent_hash)) {
        Some(header) => header,
        None => {
            return Err(Error::from(format!(
                "dao fork {}: parent header {:?} unavailable",
                fork.codename, parent_hash
            )));
        }
    };

    if !is_trigger_block(
        parent.timestamp(),
        block.header.timestamp(),
        fork.execution_timestamp,
    ) {
        return Ok(());
    }

    match get_proposal_state(client, BlockId::Hash(parent_hash), fork.proposal_id.0) {
        Ok(state) if matches!(state, ProposalState::Accepted | ProposalState::Executed) => {
            for account in fork.hardfork_accounts.iter() {
                let block_number = block.header.number();
                let st = block.state_mut();
                let balance = st.balance(&account)?;

                info!(target: "engine",
                "DAO fork '{}': proposal state {:?}, transferring {} wei from {:?} to {:?} in block {}",
                fork.codename, state, balance, account, fork.beneficiary,
                block_number);

                st.transfer_balance(&account, &fork.beneficiary, &balance, CleanupMode::NoEmpty)?;
            }
        }
        Ok(state) => {
            info!(target: "engine",
                "DAO fork '{}': NOT executing, proposal state is {:?}. This trigger is now permanently passed.",
                fork.codename, state);
        }
        Err(err) => {
            error!(target: "engine",
                "DAO fork '{}': DAO state read failed at trigger block, fork skipped: {:?}",
                fork.codename, err);
        }
    }
    Ok(())
}

fn is_trigger_block(parent_ts: u64, block_ts: u64, execution_ts: u64) -> bool {
    parent_ts < execution_ts && execution_ts <= block_ts
}
