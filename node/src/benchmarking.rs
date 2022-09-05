// This file is part of Substrate.

// Copyright (C) 2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Contains code to setup the command invocations in [`super::command`] which would
//! otherwise bloat that module.

use std::{sync::Arc, time::Duration};

use codec::Encode;
// Substrate
use sc_client_api::UsageProvider;
use sp_core::{ecdsa, Pair, H256};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_runtime::{
    generic::{Era, SignedPayload},
    traits::{Dispatchable, Extrinsic, SignedExtension},
    OpaqueExtrinsic,
};
// Local
use primitives_core::{AccountId, Balance};

use crate::{
    chain_spec::key_helper::baltathar_pair,
    client::{Client, EvaExecutor, WallEExecutor},
    service::FullClient,
};

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct RemarkBuilder {
    client: Arc<Client>,
}

impl RemarkBuilder {
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

impl frame_benchmarking_cli::ExtrinsicBuilder for RemarkBuilder {
    fn pallet(&self) -> &str {
        "system"
    }

    fn extrinsic(&self) -> &str {
        "remark"
    }

    fn build(&self, nonce: u32) -> Result<OpaqueExtrinsic, &'static str> {
        with_client! {
            self.client.as_ref(), client, {
                use runtime::{Call, SystemCall, BlockHashCount};

                let call = Call::System(SystemCall::remark { remark: vec![] });
                let signer = baltathar_pair();

                let period = BlockHashCount::get()
                    .checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
                let genesis = client.usage_info().chain.best_hash;

                client.sign_call(call, nonce, 0, period, genesis, signer)
                    .ok_or("Create signed extrinsic failed")
            }
        }
    }
}

/// Generates `Balances::TransferKeepAlive` extrinsics for the benchmarks.
///
/// Note: Should only be used for benchmarking.
pub struct TransferKeepAliveBuilder {
    client: Arc<Client>,
    dest: AccountId,
    value: Balance,
}

impl TransferKeepAliveBuilder {
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<Client>, dest: AccountId, value: Balance) -> Self {
        Self { client, dest, value }
    }
}

impl frame_benchmarking_cli::ExtrinsicBuilder for TransferKeepAliveBuilder {
    fn pallet(&self) -> &str {
        "balances"
    }

    fn extrinsic(&self) -> &str {
        "transfer_keep_alive"
    }

    fn build(&self, nonce: u32) -> Result<OpaqueExtrinsic, &'static str> {
        with_client! {
            self.client.as_ref(), client, {
                use runtime::{Call, BalancesCall, BlockHashCount};

                let call = Call::Balances(BalancesCall::transfer_keep_alive {
                    dest: self.dest,
                    value: self.value,
                 });
                let signer = baltathar_pair();

                let period = BlockHashCount::get()
                    .checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
                let genesis = client.usage_info().chain.best_hash;

                client.sign_call(call, nonce, 0, period, genesis, signer)
                .ok_or("Create signed extrinsic failed")
            }
        }
    }
}

/// Provides the existential deposit that is only needed for benchmarking.
pub trait ExistentialDepositProvider {
    /// Returns the existential deposit.
    fn existential_deposit(&self) -> Balance;
}

impl ExistentialDepositProvider for Client {
    fn existential_deposit(&self) -> Balance {
        with_client! {
            self,
            _client,
            runtime::ExistentialDeposit::get()
        }
    }
}

/// Helper trait to implement [`frame_benchmarking_cli::ExtrinsicBuilder`].
///
/// Should only be used for benchmarking since it makes strong assumptions
/// about the chain state that these calls will be valid for.
trait BenchmarkCallSigner<Call: Encode + Clone, Signer: Pair> {
    /// Signs a call together with the signed extensions of the specific runtime.
    ///
    /// Only works if the current block is the genesis block since the
    /// `CheckMortality` check is mocked by using the genesis block.
    fn sign_call(
        &self,
        call: Call,
        nonce: u32,
        current_block: u64,
        period: u64,
        genesis: H256,
        acc: Signer,
    ) -> Option<OpaqueExtrinsic>;
}

impl BenchmarkCallSigner<eva_runtime::Call, ecdsa::Pair>
    for FullClient<eva_runtime::RuntimeApi, EvaExecutor>
{
    fn sign_call(
        &self,
        call: eva_runtime::Call,
        nonce: u32,
        current_block: u64,
        period: u64,
        genesis: H256,
        acc: ecdsa::Pair,
    ) -> Option<OpaqueExtrinsic> {
        use eva_runtime::{self as runtime, Runtime};

        create_benchmark_extrinsic::<
            runtime::Address,
            runtime::Call,
            runtime::Signature,
            runtime::SignedExtra,
            <runtime::SignedExtra as SignedExtension>::AdditionalSigned,
            runtime::UncheckedExtrinsic,
        >(
            acc,
            call,
            (
                frame_system::CheckNonZeroSender::<Runtime>::new(),
                frame_system::CheckSpecVersion::<Runtime>::new(),
                frame_system::CheckTxVersion::<Runtime>::new(),
                frame_system::CheckGenesis::<Runtime>::new(),
                frame_system::CheckMortality::<Runtime>::from(Era::mortal(period, current_block)),
                frame_system::CheckNonce::<Runtime>::from(nonce),
                frame_system::CheckWeight::<Runtime>::new(),
                pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
            ),
            (
                (),
                runtime::VERSION.spec_version,
                runtime::VERSION.transaction_version,
                genesis,
                genesis,
                (),
                (),
                (),
            ),
        )
    }
}

impl BenchmarkCallSigner<wall_e_runtime::Call, ecdsa::Pair>
    for FullClient<wall_e_runtime::RuntimeApi, WallEExecutor>
{
    fn sign_call(
        &self,
        call: wall_e_runtime::Call,
        nonce: u32,
        current_block: u64,
        period: u64,
        genesis: H256,
        acc: ecdsa::Pair,
    ) -> Option<OpaqueExtrinsic> {
        use wall_e_runtime::{self as runtime, Runtime};

        create_benchmark_extrinsic::<
            runtime::Address,
            runtime::Call,
            runtime::Signature,
            runtime::SignedExtra,
            <runtime::SignedExtra as SignedExtension>::AdditionalSigned,
            runtime::UncheckedExtrinsic,
        >(
            acc,
            call,
            (
                frame_system::CheckNonZeroSender::<Runtime>::new(),
                frame_system::CheckSpecVersion::<Runtime>::new(),
                frame_system::CheckTxVersion::<Runtime>::new(),
                frame_system::CheckGenesis::<Runtime>::new(),
                frame_system::CheckMortality::<Runtime>::from(Era::mortal(period, current_block)),
                frame_system::CheckNonce::<Runtime>::from(nonce),
                frame_system::CheckWeight::<Runtime>::new(),
                pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(0),
            ),
            (
                (),
                runtime::VERSION.spec_version,
                runtime::VERSION.transaction_version,
                genesis,
                genesis,
                (),
                (),
                (),
            ),
        )
    }
}

/// Create a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
pub fn create_benchmark_extrinsic<
    Address,
    Call,
    Signature,
    SignedExtra,
    AdditionalSigned,
    UncheckedExtrinsic,
>(
    acc: ecdsa::Pair,
    call: Call,
    extra: SignedExtra,
    additional_signed: AdditionalSigned,
) -> Option<OpaqueExtrinsic>
where
    Address: From<ecdsa::Public>,
    Call: Clone + Encode + Dispatchable,
    Signature: From<ecdsa::Signature>,
    SignedExtra: SignedExtension<Call = Call, AdditionalSigned = AdditionalSigned>,
    UncheckedExtrinsic: Extrinsic<Call = Call, SignaturePayload = (Address, Signature, SignedExtra)>
        + Into<OpaqueExtrinsic>,
{
    let raw_payload = SignedPayload::<Call, SignedExtra>::from_raw(
        call.clone(),
        extra.clone(),
        additional_signed,
    );
    let signature = raw_payload.using_encoded(|p| {
        // must use `keccak_256` hash for ethereum-liked system.
        // the default hash for ecdsa in substrate is `blake2_256`
        let msg = sp_core::hashing::keccak_256(p);
        acc.sign_prehashed(&msg)
    });

    let sign_payload = (Address::from(acc.public()), Signature::from(signature), extra);
    UncheckedExtrinsic::new(call, Some(sign_payload)).map(Into::into)
}

/// Generates inherent data for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub fn inherent_benchmark_data() -> sc_cli::Result<InherentData> {
    let mut inherent_data = InherentData::new();
    let d = Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

    timestamp
        .provide_inherent_data(&mut inherent_data)
        .map_err(|e| format!("creating inherent data: {:?}", e))?;
    Ok(inherent_data)
}
