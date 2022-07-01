// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Hbbft parameter deserialization.

use ethereum_types::Address;

/// Skip block reward parameter.
/// Defines one (potential open) range about skips
/// for reward calls in the hbbft engine.
/// https://github.com/DMDcoin/openethereum-3.x/issues/49
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct HbbftParamsSkipBlockReward {
    /// No block reward calls get executed by the hbbft engine with beginning with this block (inclusive).
    pub from_block: u64,
    /// No block reward calls get executed up to this block (inclusive).
    pub to_block: Option<u64>,
}

/// Hbbft parameters.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct HbbftParams {
    /// The minimum time duration between blocks, in seconds.
    pub minimum_block_time: u64,
    /// The maximum time duration between blocks, in seconds.
    pub maximum_block_time: u64,
    /// The length of the transaction queue at which block creation should be triggered.
    pub transaction_queue_size_trigger: usize,
    /// Should be true when running unit tests to avoid starting timers.
    pub is_unit_test: Option<bool>,
    /// Block reward contract address.
    pub block_reward_contract_address: Option<Address>,
    /// Block reward skips at different blocks.
    pub block_reward_skips: Option<Vec<HbbftParamsSkipBlockReward>>,
    /// Number of consensus messages to store on the disk. 0 means zero blocks get stored.
    pub blocks_to_keep_on_disk: Option<u64>,
    /// Directory where to store the Hbbft Messages.
    /// Usually only the latest HBBFT messages are interesting for Debug, Analytics or Evidence.
    pub blocks_to_keep_directory: Option<String>,
}

/// Hbbft engine config.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hbbft {
    /// Hbbft parameters.
    pub params: HbbftParams,
}

impl HbbftParams {
    /// Should the reward call get executed.
    /// Returns false if a skip section is defined for this block number.
    pub fn should_do_block_reward_contract_call(&self, block_number: u64) -> bool {
        if let Some(skips) = &self.block_reward_skips {
            for skip in skips {
                if block_number >= skip.from_block {
                    if let Some(end) = skip.to_block {
                        if block_number <= end {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::Hbbft;
    use ethereum_types::Address;
    use std::str::FromStr;

    #[test]
    fn hbbft_deserialization() {
        let s = r#"{
			"params": {
				"minimumBlockTime": 0,
				"maximumBlockTime": 600,
				"transactionQueueSizeTrigger": 1,
				"isUnitTest": true,
				"blockRewardContractAddress": "0x2000000000000000000000000000000000000002"
			}
		}"#;

        let deserialized: Hbbft = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized.params.minimum_block_time, 0);
        assert_eq!(deserialized.params.maximum_block_time, 600);
        assert_eq!(deserialized.params.transaction_queue_size_trigger, 1);
        assert_eq!(deserialized.params.is_unit_test, Some(true));
        assert_eq!(
            deserialized.params.block_reward_contract_address,
            Address::from_str("2000000000000000000000000000000000000002").ok()
        );
    }

    #[test]
    fn hbbft_deserialization_reward_skips() {
        let s = r#"{
			"params": {
				"minimumBlockTime": 0,
				"maximumBlockTime": 600,
				"transactionQueueSizeTrigger": 1,
				"isUnitTest" : true,
				"blockRewardContractAddress": "0x2000000000000000000000000000000000000002",
				"blockRewardSkips" : [
					{ "fromBlock": 1000, "toBlock": 2000 },
					{ "fromBlock": 3000 }
				]
			}
		}"#;

        let deserialized: Hbbft = serde_json::from_str(s).unwrap();
        assert!(deserialized.params.block_reward_skips.is_some());
        let skips = deserialized.params.block_reward_skips.as_ref().unwrap();
        assert_eq!(skips.len(), 2);
        assert_eq!(
            deserialized.params.should_do_block_reward_contract_call(0),
            true
        );
        assert_eq!(
            deserialized
                .params
                .should_do_block_reward_contract_call(1000),
            false
        );
        assert_eq!(
            deserialized
                .params
                .should_do_block_reward_contract_call(1500),
            false
        );
        assert_eq!(
            deserialized
                .params
                .should_do_block_reward_contract_call(2000),
            false
        );
        assert_eq!(
            deserialized
                .params
                .should_do_block_reward_contract_call(2001),
            true
        );
        assert_eq!(
            deserialized
                .params
                .should_do_block_reward_contract_call(3001),
            false
        );
        assert_eq!(
            deserialized
                .params
                .should_do_block_reward_contract_call(100_000),
            false
        );
    }
}
