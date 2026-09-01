use crate::{
    block::ExecutedBlock,
    client::EngineClient,
    engines::hbbft::contracts::diamond_dao::{ProposalState, get_proposal_state},
    error::Error,
    state::CleanupMode,
    types::ids::BlockId,
};
use ethereum_types::Address;
use ethjson::spec::hbbft::HbbftDaoHardforkTransfer;
use std::collections::HashSet;

pub fn validate_config(forks: &[HbbftDaoHardforkTransfer]) -> Result<(), String> {
    let mut codenames = HashSet::new();
    let mut proposal_ids = HashSet::new();

    for fork in forks {
        let codename = fork.codename.trim();
        if codename.is_empty() {
            return Err("DAO hardfork codename cannot be empty".into());
        }

        if !codenames.insert(codename.to_owned()) {
            return Err(format!("duplicate DAO hardfork codename '{}'", codename));
        }

        if fork.proposal_id.0.is_zero() {
            return Err(format!(
                "DAO fork '{}': proposalId cannot be zero",
                fork.codename
            ));
        }

        if !proposal_ids.insert(fork.proposal_id.0) {
            return Err(format!(
                "DAO fork '{}': duplicate proposalId",
                fork.codename
            ));
        }

        if fork.execution_timestamp == 0 {
            return Err(format!(
                "DAO fork '{}': executionTimestamp cannot be zero",
                fork.codename
            ));
        }

        if fork.hardfork_accounts.is_empty() {
            return Err(format!(
                "DAO fork '{}': hardforkAccounts cannot be empty",
                fork.codename
            ));
        }

        if fork.beneficiary == Address::zero() {
            return Err(format!(
                "DAO fork '{}': beneficiary cannot be zero",
                fork.codename
            ));
        }

        let mut local_sources = HashSet::new();

        for source in &fork.hardfork_accounts {
            if *source == Address::zero() {
                return Err(format!(
                    "DAO fork '{}': source cannot be zero",
                    fork.codename
                ));
            }

            if *source == fork.beneficiary {
                return Err(format!(
                    "DAO fork '{}': beneficiary cannot be a source",
                    fork.codename
                ));
            }

            if !local_sources.insert(*source) {
                return Err(format!(
                    "DAO fork '{}': duplicate source {:?}",
                    fork.codename, source
                ));
            }
        }
    }

    Ok(())
}

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

    let proposal_state = get_proposal_state(client, BlockId::Hash(parent_hash), fork.proposal_id.0)
        .map_err(|err| {
            Error::from(format!(
                "DAO fork '{}': state read failed: {:?}",
                fork.codename, err
            ))
        })?;

    match proposal_state {
        ProposalState::Accepted | ProposalState::Executed => {
            for account in fork.hardfork_accounts.iter() {
                let block_number = block.header.number();
                let st = block.state_mut();
                let balance = st.balance(&account)?;

                info!(target: "engine",
                "DAO fork '{}': proposal state {:?}, transferring {} wei from {:?} to {:?} in block {}",
                fork.codename, proposal_state, balance, account, fork.beneficiary,
                block_number);

                st.transfer_balance(&account, &fork.beneficiary, &balance, CleanupMode::NoEmpty)?;
            }
        }
        state => {
            info!(target: "engine",
                "DAO fork '{}': NOT executing, proposal state is {:?}. This trigger is now permanently passed.",
                fork.codename, state);
        }
    }
    Ok(())
}

fn is_trigger_block(parent_ts: u64, block_ts: u64, execution_ts: u64) -> bool {
    parent_ts < execution_ts && execution_ts <= block_ts
}

#[cfg(test)]
mod tests {
    use super::validate_config;
    use ethereum_types::{Address, U256};
    use ethjson::{spec::hbbft::HbbftDaoHardforkTransfer, uint::Uint};

    fn address(value: u64) -> Address {
        Address::from_low_u64_be(value)
    }

    fn dao_hardfork(
        codename: &str,
        proposal_id: u64,
        execution_timestamp: u64,
        sources: Vec<Address>,
        beneficiary: Address,
    ) -> HbbftDaoHardforkTransfer {
        HbbftDaoHardforkTransfer {
            codename: codename.to_owned(),
            proposal_id: Uint(U256::from(proposal_id)),
            execution_timestamp,
            hardfork_accounts: sources,
            beneficiary,
        }
    }

    #[test]
    fn accepts_valid_config() {
        let forks = vec![
            dao_hardfork("Scintilla", 1, 100, vec![address(1)], address(10)),
            dao_hardfork("Second", 2, 100, vec![address(2)], address(10)),
            dao_hardfork("Third", 3, 200, vec![address(1)], address(20)),
        ];

        assert!(validate_config(&forks).is_ok());
    }

    #[test]
    fn rejects_invalid_single_fork_fields() {
        let valid = dao_hardfork("Scintilla", 1, 100, vec![address(1)], address(10));
        let invalid = vec![
            dao_hardfork(" ", 1, 100, vec![address(1)], address(10)),
            dao_hardfork("Scintilla", 0, 100, vec![address(1)], address(10)),
            dao_hardfork("Scintilla", 1, 0, vec![address(1)], address(10)),
            dao_hardfork("Scintilla", 1, 100, vec![], address(10)),
            dao_hardfork("Scintilla", 1, 100, vec![address(1)], Address::zero()),
            dao_hardfork("Scintilla", 1, 100, vec![Address::zero()], address(10)),
            dao_hardfork("Scintilla", 1, 100, vec![address(10)], address(10)),
            dao_hardfork(
                "Scintilla",
                1,
                100,
                vec![address(1), address(1)],
                address(10),
            ),
        ];

        assert!(validate_config(&[valid]).is_ok());
        for config in invalid {
            assert!(validate_config(&[config]).is_err());
        }
    }

    #[test]
    fn rejects_duplicate_identifiers() {
        let duplicate_codename = vec![
            dao_hardfork("Scintilla", 1, 100, vec![address(1)], address(10)),
            dao_hardfork("Scintilla", 2, 200, vec![address(2)], address(20)),
        ];
        let duplicate_proposal = vec![
            dao_hardfork("Scintilla", 1, 100, vec![address(1)], address(10)),
            dao_hardfork("Second", 1, 200, vec![address(2)], address(20)),
        ];

        assert!(validate_config(&duplicate_codename).is_err());
        assert!(validate_config(&duplicate_proposal).is_err());
    }
}
