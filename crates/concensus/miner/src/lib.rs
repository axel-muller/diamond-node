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

#![warn(missing_docs)]

//! Miner module
//! Keeps track of transactions and mined block.

extern crate ansi_term;
extern crate common_types as types;
extern crate ethabi;
extern crate ethabi_derive;
extern crate ethcore_call_contract as call_contract;
extern crate ethereum_types;
extern crate futures;
extern crate keccak_hash as hash;
extern crate linked_hash_map;
extern crate parity_crypto as crypto;
extern crate parity_runtime;
extern crate parity_util_mem;
extern crate parking_lot;
#[cfg(feature = "price-info")]
extern crate price_info;
extern crate rlp;
extern crate rustc_hex;
extern crate txpool;

#[macro_use]
extern crate ethabi_contract;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate trace_time;

#[cfg(test)]
extern crate env_logger;
#[cfg(test)]
extern crate ethkey;

pub mod external;
#[cfg(feature = "price-info")]
pub mod gas_price_calibrator;
pub mod gas_pricer;
pub mod local_accounts;
pub mod pool;
pub mod service_transaction_checker;
#[cfg(feature = "work-notify")]
pub mod work_notify;
