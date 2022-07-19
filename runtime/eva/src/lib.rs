#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::or_fun_call)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Substrate
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{
        BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable, IdentityLookup, NumberFor,
        OpaqueKeys, PostDispatchInfoOf, UniqueSaturatedInto,
    },
    transaction_validity::{TransactionSource, TransactionValidity, TransactionValidityError},
    ApplyExtrinsicResult, Permill,
};
use sp_std::prelude::*;
use sp_version::RuntimeVersion;
// Substrate FRAME
use frame_support::{
    construct_runtime, parameter_types,
    traits::{ConstU16, ConstU32, EitherOfDiverse, KeyOwnerProofSystem},
    weights::{constants::RocksDbWeight, ConstantMultiplier},
};
use frame_system::EnsureRoot;
use pallet_evm::{
    EVMCurrencyAdapter, EnsureAddressNever, EnsureAddressRoot, FeeCalculator, Runner,
};
use pallet_grandpa::{fg_primitives, AuthorityList as GrandpaAuthorityList};
use pallet_transaction_payment::CurrencyAdapter;
// re-exports
// A few exports that help ease life for downstream crates.
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_ethereum::{Call as EthereumCall, Transaction as EthereumTransaction};
pub use pallet_timestamp::Call as TimestampCall;
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_finality_grandpa::AuthorityId as GrandpaId;
// Local
use eva_runtime_constants::{currency, evm, fee, system, time};
use primitives_core::{
    AccountId, Address, Balance, Block as NodeBlock, BlockNumber, Hash, Header, Index, Moment,
    Signature,
};
use runtime_common::{
    evm_config,
    pallets::{authorities as pallet_authorities, privilege as pallet_privilege},
    precompiles::FrontierPrecompiles,
    CoinbaseAuthor, ToAuthor,
};

// To learn more about runtime versioning and what each of the following value means:
//   https://docs.substrate.io/v3/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("armonia-eva"),
    impl_name: create_runtime_str!("armonia-eva"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
    sp_version::NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

// Configure FRAME pallets to include in runtime.

// ################################################################################################
// System && Utility.
// ################################################################################################

parameter_types! {
    pub const Version: RuntimeVersion = VERSION;
    pub const BlockHashCount: BlockNumber = 2400;
    pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
        ::with_sensible_defaults(system::MAXIMUM_BLOCK_WEIGHT, system::NORMAL_DISPATCH_RATIO);
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(system::MAXIMUM_BLOCK_LENGTH, system::NORMAL_DISPATCH_RATIO);
    pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
    /// The basic call filter to use in dispatchable.
    type BaseCallFilter = frame_support::traits::Everything;
    /// Block & extrinsics weights: base values and limits.
    type BlockWeights = BlockWeights;
    /// The maximum length of a block (in bytes).
    type BlockLength = BlockLength;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = Index;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = BlakeTwo256;
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = IdentityLookup<AccountId>;
    /// The header type.
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// The ubiquitous event type.
    type Event = Event;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// The weight of database operations that the runtime can invoke.
    type DbWeight = RocksDbWeight;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type PalletInfo = PalletInfo;
    /// The data to be stored in an account.
    type AccountData = pallet_balances::AccountData<Balance>;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = ();
    /// Weight information for the extrinsics of this pallet.
    type SystemWeightInfo = ();
    /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
    type SS58Prefix = SS58Prefix;
    /// The set code logic, just the default since we're not a parachain.
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const MinimumPeriod: Moment = time::SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = Moment;
    #[cfg(feature = "aura")]
    type OnTimestampSet = Aura;
    #[cfg(feature = "manual-seal")]
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = currency::deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = currency::deposit(0, 32);
}

impl pallet_multisig::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = ConstU16<100>;
    type WeightInfo = ();
}

impl pallet_sudo::Config for Runtime {
    type Event = Event;
    type Call = Call;
}

// ################################################################################################
// Monetary.
// ################################################################################################

parameter_types! {
    // TODO. need to check this value.
    pub const ExistentialDeposit: Balance = 0;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
}

parameter_types! {
    // TODO. need to check this value.
    pub const TransactionByteFee: Balance = 1;
    // TODO. need to check this value.
    pub const OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
    type OnChargeTransaction = CurrencyAdapter<Balances, ToAuthor<Runtime, Balances>>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type WeightToFee = fee::WeightToFee;
    type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
    type FeeMultiplierUpdate = ();
}

// ################################################################################################
// Consensus.
// ################################################################################################

parameter_types! {
    pub const MaxAuthorities: u32 = 21;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = AuraId;
    type MaxAuthorities = MaxAuthorities;
    type DisabledValidators = ();
}

impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;

    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

    type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
        KeyTypeId,
        GrandpaId,
    )>>::IdentificationTuple;

    type KeyOwnerProofSystem = ();

    type HandleEquivocation = ();

    type WeightInfo = ();
    type MaxAuthorities = MaxAuthorities;
}

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        pub grandpa: Grandpa,
    }
}

parameter_types! {
    pub const Period: BlockNumber = 4 * time::HOURS;
}

impl pallet_session::Config for Runtime {
    type Event = Event;
    type ValidatorId = <Self as frame_system::Config>::AccountId;
    type ValidatorIdOf = Authorities;
    type ShouldEndSession = pallet_session::PeriodicSessions<Period, Period>;
    type NextSessionRotation = pallet_session::PeriodicSessions<Period, Period>;
    type SessionManager = Authorities;
    type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
    type Keys = SessionKeys;
    type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
    pub const UncleGenerations: BlockNumber = 0;
}

impl pallet_authorship::Config for Runtime {
    type FindAuthor = CoinbaseAuthor<Runtime, Aura>;
    type UncleGenerations = UncleGenerations;
    type FilterUncle = ();
    type EventHandler = ();
}

impl pallet_authorities::Config for Runtime {
    type Event = Event;
    type MaxAuthorities = MaxAuthorities;
}

// ################################################################################################
// Governance.
// ################################################################################################

impl pallet_privilege::Config for Runtime {
    type Event = Event;
    type PrivilegeOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    type Call = Call;
}

parameter_types! {
    /// The maximum amount of time (in blocks) for technical committee members to vote on motions.
    /// Motions may end in fewer blocks if enough votes are cast to determine the result.
    pub const MotionDuration: BlockNumber = 7 * time::DAYS;
    /// The maximum number of technical committee members.
    pub const MaxProposals: u32 = 32;
    /// The maximum number of technical committee members.
    pub const TechnicalMaxMembers: u32 = 30;
}

type TechnicalCollective = pallet_collective::Instance1;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
    type Origin = Origin;
    type Proposal = Call;
    type Event = Event;
    /// The maximum amount of time (in blocks) for technical committee members to vote on motions.
    /// Motions may end in fewer blocks if enough votes are cast to determine the result.
    type MotionDuration = MotionDuration;
    /// The maximum number of Proposals that can be open in the technical committee at once.
    type MaxProposals = MaxProposals;
    type MaxMembers = TechnicalMaxMembers;
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type WeightInfo = ();
}

/// 2/3 vote right for Technical members.
type EnsureRootOrTwoThirdsTechnicalCommittee = EitherOfDiverse<
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>,
>;

type TechnicalMembership = pallet_membership::Instance1;
impl pallet_membership::Config<TechnicalMembership> for Runtime {
    type Event = Event;
    type AddOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    type RemoveOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    type SwapOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    type ResetOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    type PrimeOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    type MembershipInitialized = TechnicalCommittee;
    type MembershipChanged = TechnicalCommittee;
    type MaxMembers = TechnicalMaxMembers;
    type WeightInfo = ();
}

// ################################################################################################
// EVM compatibility.
// ################################################################################################

parameter_types! {
    pub const ChainId: u64 = 160;
    pub BlockGasLimit: U256 = U256::from(system::NORMAL_DISPATCH_RATIO * system::MAXIMUM_BLOCK_WEIGHT / evm::WEIGHT_PER_GAS);
    pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::new();
}

impl pallet_evm::Config for Runtime {
    type FeeCalculator = evm_config::FixedGasPrice;
    type GasWeightMapping = evm_config::GasWeightMapping;
    type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
    type CallOrigin = EnsureAddressRoot<AccountId>;
    type WithdrawOrigin = EnsureAddressNever<AccountId>;
    type AddressMapping = evm_config::IntoAddressMapping;
    type Currency = Balances;
    type Event = Event;
    type PrecompilesType = FrontierPrecompiles<Self>;
    type PrecompilesValue = PrecompilesValue;
    type ChainId = ChainId;
    type BlockGasLimit = BlockGasLimit;
    type Runner = pallet_evm::runner::stack::Runner<Self>;
    type OnChargeTransaction = EVMCurrencyAdapter<Balances, ToAuthor<Runtime, Balances>>;
    type FindAuthor = CoinbaseAuthor<Runtime, Aura>;
}

impl pallet_ethereum::Config for Runtime {
    type Event = Event;
    type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

parameter_types! {
    pub IsActive: bool = false;
    pub DefaultBaseFeePerGas: U256 = U256::from(1_000_000_000);
}

impl pallet_base_fee::Config for Runtime {
    type Event = Event;
    type Threshold = evm_config::BaseFeeThreshold;
    type IsActive = IsActive;
    type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = NodeBlock,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        // System && Utility.
        System: frame_system = 0,
        Timestamp: pallet_timestamp = 1,
        Utility: pallet_utility = 2,
        Multisig: pallet_multisig = 3,

        // Monetary.
        Balances: pallet_balances = 10,
        TransactionPayment: pallet_transaction_payment = 11,

        // Consensus.
        Aura: pallet_aura = 20,
        Grandpa: pallet_grandpa = 21,
        Session: pallet_session = 22,
        Authorship: pallet_authorship = 23,
        Authorities: pallet_authorities = 24,

        // Governance.
        Privilege: pallet_privilege = 30,
        TechnicalCommittee: pallet_collective::<Instance1> = 31,
        TechnicalCommitteeMembership: pallet_membership::<Instance1> = 32,

        // Evm compatibility.
        EVM: pallet_evm = 100,
        Ethereum: pallet_ethereum = 101,
        BaseFee: pallet_base_fee = 102,

        // Sudo (temporary).
        Sudo: pallet_sudo = 255,
    }
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
    fp_self_contained::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = fp_self_contained::CheckedExtrinsic<AccountId, Call, SignedExtra, H160>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

pub type TransactionConverter =
    runtime_common::ethereum::EthTransactionConverter<UncheckedExtrinsic, Runtime>;
// frontier interface for runtime-api
impl fp_self_contained::SelfContainedCall for Call {
    type SignedInfo = H160;

    fn is_self_contained(&self) -> bool {
        match self {
            Call::Ethereum(call) => call.is_self_contained(),
            _ => false,
        }
    }

    fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.check_self_contained(),
            _ => None,
        }
    }

    fn validate_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<Call>,
        len: usize,
    ) -> Option<TransactionValidity> {
        match self {
            Call::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
            _ => None,
        }
    }

    fn pre_dispatch_self_contained(
        &self,
        info: &Self::SignedInfo,
        dispatch_info: &DispatchInfoOf<Call>,
        len: usize,
    ) -> Option<Result<(), TransactionValidityError>> {
        match self {
            Call::Ethereum(call) => call.pre_dispatch_self_contained(info, dispatch_info, len),
            _ => None,
        }
    }

    fn apply_self_contained(
        self,
        info: Self::SignedInfo,
    ) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
        match self {
            call @ Call::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(
                call.dispatch(Origin::from(pallet_ethereum::RawOrigin::EthereumTransaction(info))),
            ),
            _ => None,
        }
    }
}

/// if the call is the eth tx, return it, else None.
fn try_get_eth_trx(call: Call) -> Option<EthereumTransaction> {
    match call {
        Call::Ethereum(EthereumCall::transact { transaction }) => Some(transaction),
        _ => None,
    }
}

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    define_benchmarks!(
        [frame_benchmarking, BaselineBench::<Runtime>]
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_timestamp, Timestamp]
    );
}

impl_runtime_apis! {
    // ############################################################################################
    // Substrate
    // ############################################################################################

    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }

        fn authorities() -> Vec<AuraId> {
            Aura::authorities().into_inner()
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            SessionKeys::generate(seed)
        }

        fn decode_session_keys(
            encoded: Vec<u8>,
        ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            SessionKeys::decode_into_raw_public_keys(&encoded)
        }
    }

    impl fg_primitives::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: fg_primitives::SetId,
            _authority_id: GrandpaId,
        ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
            // NOTE: this is the only implementation possible since we've
            // defined our key owner proof type as a bottom type (i.e. a type
            // with no values).
            None
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            let mut list = Vec::<BenchmarkList>::new();
            list_benchmarks!(list, extra);

            let storage_info = AllPalletsWithSystem::storage_info();

            (list, storage_info)
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, TrackedStorageKey};

            use frame_system_benchmarking::Pallet as SystemBench;
            use baseline::Pallet as BaselineBench;

            impl frame_system_benchmarking::Config for Runtime {}
            impl baseline::Config for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                // Block Number
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
                // Total Issuance
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
                // Execution Phase
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
                // Event Count
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
                // System Events
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);
            add_benchmarks!(params, batches);

            Ok(batches)
        }
    }

    #[cfg(feature = "try-runtime")]
    impl frame_try_runtime::TryRuntime<Block> for Runtime {
        fn on_runtime_upgrade() -> (
            frame_support::weights::Weight,
            frame_support::weights::Weight,
        ) {
            // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
            // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
            // right here and right now.
            let weight = Executive::try_runtime_upgrade().unwrap();
            (weight, BlockWeights::get().max_block)
        }

        fn execute_block_no_check(block: Block) -> frame_support::weights::Weight {
            Executive::execute_block_no_check(block)
        }
    }

    // ############################################################################################
    // Frontier
    // ############################################################################################

    impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
        fn chain_id() -> u64 {
            <Runtime as pallet_evm::Config>::ChainId::get()
        }

        fn account_basic(address: H160) -> pallet_evm::Account {
            let (account, _) = EVM::account_basic(&address);
            account
        }

        fn gas_price() -> U256 {
            let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
            gas_price
        }

        fn account_code_at(address: H160) -> Vec<u8> {
            EVM::account_codes(address)
        }

        fn author() -> H160 {
            <pallet_evm::Pallet<Runtime>>::find_author()
        }

        fn storage_at(address: H160, index: U256) -> H256 {
            let mut tmp = [0u8; 32];
            index.to_big_endian(&mut tmp);
            EVM::account_storages(address, H256::from_slice(&tmp[..]))
        }

        fn call(
            from: H160,
            to: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

            let is_transactional = false;
            let validate = true;
            let evm_config = config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config());
            <Runtime as pallet_evm::Config>::Runner::call(
                from,
                to,
                data,
                value,
                gas_limit.unique_saturated_into(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list.unwrap_or_default(),
                is_transactional,
                validate,
                evm_config,
            ).map_err(|err| err.error.into())
        }

        fn create(
            from: H160,
            data: Vec<u8>,
            value: U256,
            gas_limit: U256,
            max_fee_per_gas: Option<U256>,
            max_priority_fee_per_gas: Option<U256>,
            nonce: Option<U256>,
            estimate: bool,
            access_list: Option<Vec<(H160, Vec<H256>)>>,
        ) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
            let config = if estimate {
                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                Some(config)
            } else {
                None
            };

            let is_transactional = false;
            let validate = true;
            let evm_config = config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config());
            <Runtime as pallet_evm::Config>::Runner::create(
                from,
                data,
                value,
                gas_limit.unique_saturated_into(),
                max_fee_per_gas,
                max_priority_fee_per_gas,
                nonce,
                access_list.unwrap_or_default(),
                is_transactional,
                validate,
                evm_config,
            ).map_err(|err| err.error.into())
        }

        fn current_transaction_statuses() -> Option<Vec<fp_rpc::TransactionStatus>> {
            Ethereum::current_transaction_statuses()
        }

        fn current_block() -> Option<pallet_ethereum::Block> {
            Ethereum::current_block()
        }

        fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
            Ethereum::current_receipts()
        }

        fn current_all() -> (
            Option<pallet_ethereum::Block>,
            Option<Vec<pallet_ethereum::Receipt>>,
            Option<Vec<fp_rpc::TransactionStatus>>,
        ) {
            (
                Ethereum::current_block(),
                Ethereum::current_receipts(),
                Ethereum::current_transaction_statuses(),
            )
        }

        fn extrinsic_filter(
            xts: Vec<<Block as BlockT>::Extrinsic>,
        ) -> Vec<EthereumTransaction> {
            xts.into_iter().filter_map(|xt| match xt.0.function {
                Call::Ethereum(EthereumCall::transact { transaction }) => Some(transaction),
                _ => None
            }).collect::<Vec<EthereumTransaction>>()
        }

        fn elasticity() -> Option<Permill> {
            Some(BaseFee::elasticity())
        }
    }

    impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
        fn convert_transaction(
            transaction: pallet_ethereum::Transaction
        ) -> <Block as BlockT>::Extrinsic {
            UncheckedExtrinsic::new_unsigned(
                pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
            )
        }
    }

    // ############################################################################################
    // Tracing RPC
    // ############################################################################################

    impl primitives_rpc::debug::DebugRuntimeApi<Block> for Runtime {
        #[allow(unused)]
        fn trace_transaction(
            extrinsics: Vec<<Block as BlockT>::Extrinsic>,
            traced_transaction: &EthereumTransaction,
        ) -> Result<(), sp_runtime::DispatchError> {
            #[cfg(feature = "evm-tracing")]
            {
                use primitives_evm_tracer::EvmTracer;
                // Apply the a subset of extrinsics: all the substrate-specific or ethereum
                // transactions that preceded the requested transaction.
                for ext in extrinsics.into_iter() {
                    let _ = match &ext.0.function {
                        Call::Ethereum(EthereumCall::transact { transaction }) => {
                            if transaction == traced_transaction {
                                EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
                                return Ok(());
                            } else {
                                Executive::apply_extrinsic(ext)
                            }
                        },
                        _ => Executive::apply_extrinsic(ext),
                    };
                }
                Err(sp_runtime::DispatchError::Other(
                    "Failed to find Ethereum transaction among the extrinsics.",
                ))
            }

            #[cfg(not(feature = "evm-tracing"))]
            {
                Err(sp_runtime::DispatchError::Other("Missing `evm-tracing` feature flag."))
            }
        }

        #[allow(unused)]
        fn trace_block(
            extrinsics: Vec<<Block as BlockT>::Extrinsic>,
            known_transactions: Vec<H256>,
        ) -> Result<(), sp_runtime::DispatchError> {
            #[cfg(feature = "evm-tracing")]
            {
                use primitives_evm_tracer::EvmTracer;

                let mut config = <Runtime as pallet_evm::Config>::config().clone();
                config.estimate = true;
                // Apply all extrinsics. Ethereum extrinsics are traced.
                for ext in extrinsics.into_iter() {
                    match &ext.0.function {
                        Call::Ethereum(EthereumCall::transact { transaction }) => {
                            if known_transactions.contains(&transaction.hash()) {
                                // Each known extrinsic is a new call stack.
                                EvmTracer::emit_new();
                                EvmTracer::new().trace(|| Executive::apply_extrinsic(ext));
                            } else {
                                let _ = Executive::apply_extrinsic(ext);
                            }
                        }
                        _ => {
                            let _ = Executive::apply_extrinsic(ext);
                        }
                    };
                }
                Ok(())
            }

            #[cfg(not(feature = "evm-tracing"))]
            {
                Err(sp_runtime::DispatchError::Other("Missing `evm-tracing` feature flag."))
            }
        }
    }

    impl primitives_rpc::txpool::TxPoolRuntimeApi<Block> for Runtime {
        fn extrinsic_filter(
            xts_ready: Vec<<Block as BlockT>::Extrinsic>,
            xts_future: Vec<<Block as BlockT>::Extrinsic>,
        ) -> primitives_rpc::txpool::TxPoolResponse {
            primitives_rpc::txpool::TxPoolResponse {
                ready: xts_ready
                    .into_iter()
                    .filter_map(|xt| try_get_eth_trx(xt.0.function))
                    .collect(),
                future: xts_future
                    .into_iter()
                    .filter_map(|xt| try_get_eth_trx(xt.0.function))
                    .collect(),
            }
        }
    }
}
