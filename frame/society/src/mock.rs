// This file is part of Substrate.

// Copyright (C) 2020-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Test utilities

use super::*;
use crate as pallet_society;

use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{ConstU32, ConstU64},
};
use frame_support_test::TestRandomness;
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Society: pallet_society::{Pallet, Call, Storage, Event<T>, Config<T>},
	}
);

parameter_types! {
	pub const SocietyPalletId: PalletId = PalletId(*b"py/socie");
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

ord_parameter_types! {
	pub const FounderSetAccount: u128 = 1;
	pub const SuspensionJudgementSetAccount: u128 = 2;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = u128;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type AccountData = pallet_balances::AccountData<u64>;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

impl Config for Test {
	type Event = Event;
	type PalletId = SocietyPalletId;
	type Currency = pallet_balances::Pallet<Self>;
	type Randomness = TestRandomness<Self>;
	type GraceStrikes = ConstU32<1>;
	type PeriodSpend = ConstU64<1000>;
	type VotingPeriod = ConstU64<3>;
	type ClaimPeriod = ConstU64<1>;
	type MaxLockDuration = ConstU64<100>;
	type FounderSetOrigin = EnsureSignedBy<FounderSetAccount, u128>;
	type ChallengePeriod = ConstU64<8>;
	type MaxPayouts = ConstU32<10>;
	type MaxBids = ConstU32<10>;
}

pub struct EnvBuilder {
	balance: u64,
	balances: Vec<(u128, u64)>,
	pot: u64,
	founded: bool,
}

impl EnvBuilder {
	pub fn new() -> Self {
		Self {
			balance: 10_000,
			balances: vec![
				(10, 50),
				(20, 50),
				(30, 50),
				(40, 50),
				(50, 50),
				(60, 50),
				(70, 50),
				(80, 50),
				(90, 50),
			],
			pot: 0,
			founded: true,
		}
	}

	pub fn execute<R, F: FnOnce() -> R>(mut self, f: F) -> R {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		self.balances.push((Society::account_id(), self.balance.max(self.pot)));
		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();
		pallet_society::GenesisConfig::<Test> {
			pot: self.pot,
		}
		.assimilate_storage(&mut t)
		.unwrap();
		let mut ext: sp_io::TestExternalities = t.into();
		ext.execute_with(|| {
			if self.founded {
				let r = b"be cool".to_vec();
				assert!(Society::found_society(Origin::signed(1), 10, 10, 8, 2, 25, r).is_ok());
			}
			f()
		})
	}
	pub fn founded(mut self, f: bool) -> Self {
		self.founded = f;
		self
	}
	pub fn with_pot(mut self, p: u64) -> Self {
		self.pot = p;
		self
	}
}

/// Run until a particular block.
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Society::on_initialize(System::block_number());
	}
}

/// Creates a bid struct using input parameters.
pub fn bid<AccountId, Balance>(
	who: AccountId,
	kind: BidKind<AccountId, Balance>,
	value: Balance,
) -> Bid<AccountId, Balance> {
	Bid { who, kind, value }
}

/// Creates a candidate struct using input parameters.
pub fn candidacy<AccountId, Balance>(
	round: RoundIndex,
	bid: Balance,
	kind: BidKind<AccountId, Balance>,
	approvals: VoteCount,
	rejections: VoteCount,
) -> Candidacy<AccountId, Balance> {
	Candidacy { round, kind, bid, tally: Tally { approvals, rejections }, skeptic_struck: false }
}
