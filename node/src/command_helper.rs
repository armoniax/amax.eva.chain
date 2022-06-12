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

use sc_client_api::UsageProvider;
use sp_core::{crypto::DEV_PHRASE, ecdsa, Encode, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_runtime::OpaqueExtrinsic;

use crate::{
    chain_spec::key_helper::{derive_bip44_pairs_from_mnemonic, get_account_id_from_pair},
    client::Client,
};

macro_rules! signed_payload {
    (
	$extra:ident, $raw_payload:ident,
	(
		$period:expr,
		$current_block:expr,
		$nonce:expr,
		$tip:expr,
		$call:expr,
		$genesis:expr
	)
	) => {
        let $extra: runtime::SignedExtra = (
            frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
            frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
            frame_system::CheckTxVersion::<runtime::Runtime>::new(),
            frame_system::CheckGenesis::<runtime::Runtime>::new(),
            frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
                $period,
                $current_block,
            )),
            frame_system::CheckNonce::<runtime::Runtime>::from($nonce),
            frame_system::CheckWeight::<runtime::Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from($tip),
        );

        let $raw_payload = runtime::SignedPayload::from_raw(
            $call.clone(),
            $extra.clone(),
            (
                (),
                runtime::VERSION.spec_version,
                runtime::VERSION.transaction_version,
                $genesis.clone(),
                $genesis,
                (),
                (),
                (),
            ),
        );
    };
}

macro_rules! with_signed_payload {
	{
		$multi_client:ident,
		{
			$extra:ident,
			$client:ident,
			$raw_payload:ident
		},
		{
			$( $setup:tt )*
		},
		(
			$period:expr,
			$current_block:expr,
			$nonce:expr,
			$tip:expr,
			$call:expr,
			$genesis:expr
		),
		{
			$( $usage:tt )*
		}
	} => {
		match $multi_client {
			Client::Eva($client) => {
				use eva_runtime as runtime;

				$( $setup )*

				signed_payload!($extra, $raw_payload,
					($period, $current_block, $nonce, $tip, $call, $genesis));

				$( $usage )*
			},
			Client::WallE($client) => {
				use wall_e_runtime as runtime;

				$( $setup )*

				signed_payload!($extra, $raw_payload,
					($period, $current_block, $nonce, $tip, $call, $genesis));

				$( $usage )*
			},
		}
	}
}

/// Generates extrinsics for the `benchmark overhead` command.
///
/// Note: Should only be used for benchmarking.
pub struct BenchmarkExtrinsicBuilder {
    client: Arc<Client>,
}

impl BenchmarkExtrinsicBuilder {
    /// Creates a new [`Self`] from the given client.
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

impl frame_benchmarking_cli::ExtrinsicBuilder for BenchmarkExtrinsicBuilder {
    fn remark(&self, nonce: u32) -> Result<OpaqueExtrinsic, &'static str> {
        let acc = derive_bip44_pairs_from_mnemonic::<ecdsa::Public>(DEV_PHRASE, 2);
        let sender = acc[1].clone();
        let multi_client = self.client.as_ref();
        with_signed_payload! {
            multi_client,
            {extra, client, raw_payload},
            {
                // First the setup code to init all the variables that are needed
                // to build the signed extras.
                use runtime::{Call, SystemCall};

                let call = Call::System(SystemCall::remark { remark: vec![] });
                let period = runtime_common::constants::system::BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;

                let current_block = 0;
                let tip = 0;
                let genesis = client.usage_info().chain.best_hash;
            },
            (period, current_block, nonce, tip, call, genesis),
            {
                // Use the payload to generate a signature.
                let signature = raw_payload.using_encoded(|e| sender.sign(e));
                let signed = get_account_id_from_pair(sender).expect("must can generate account_id");
                let ext = runtime::UncheckedExtrinsic::new_signed(
                    call,
                    signed,
                    primitives_core::Signature::from(signature),
                    extra,
                );
                return Ok(ext.into());
            }
        }

        // let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
        //     self.client.as_ref(),
        //     acc[1].clone(),
        //     runtime::SystemCall::remark { remark: vec![] }.into(),
        //     nonce,
        // )
        // .into();

        // Ok(extrinsic)
    }
}
/*
/// Create a transaction using the given `call`.
///
/// Note: Should only be used for benchmarking.
pub fn create_benchmark_extrinsic(
    client: &Client,
    sender: ecdsa::Pair,
    call: runtime::Call,
    nonce: u32,
) -> runtime::UncheckedExtrinsic {
    let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
    let best_hash = client.chain_info().best_hash;
    let best_block = client.chain_info().best_number;

    let period = runtime::BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2) as u64;
    let extra: runtime::SignedExtra = (
        frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
            period,
            best_block.saturated_into(),
        )),
        frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
        frame_system::CheckWeight::<runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
    );

    let raw_payload = runtime::SignedPayload::from_raw(
        call.clone(),
        extra.clone(),
        (
            (),
            runtime::VERSION.spec_version,
            runtime::VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    let signed = get_account_id_from_pair(sender).expect("must can generate account_id");
    runtime::UncheckedExtrinsic::new_signed(
        call,
        signed,
        runtime::Signature::from(signature),
        extra,
    )
}
*/
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
