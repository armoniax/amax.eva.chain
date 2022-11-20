use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};

use precompile_utils::precompile_set::{
    AddressU64, AllowDelegateCall, ForbidRecursion, PrecompileAt, PrecompileSetBuilder,
    PrecompilesInRangeInclusive,
};

pub type WallEPrecompiles<R> = PrecompileSetBuilder<
    R,
    (
        // Skip precompiles if out of range.
        PrecompilesInRangeInclusive<
            (AddressU64<1>, AddressU64<4095>),
            (
                // Ethereum precompiles:
                // We allow DELEGATECALL to stay compliant with Ethereum behavior.
                PrecompileAt<AddressU64<1>, ECRecover, ForbidRecursion, AllowDelegateCall>,
                PrecompileAt<AddressU64<2>, Sha256, ForbidRecursion, AllowDelegateCall>,
                PrecompileAt<AddressU64<3>, Ripemd160, ForbidRecursion, AllowDelegateCall>,
                PrecompileAt<AddressU64<4>, Identity, ForbidRecursion, AllowDelegateCall>,
                PrecompileAt<AddressU64<5>, Modexp, ForbidRecursion, AllowDelegateCall>,
                PrecompileAt<AddressU64<6>, Bn128Add, ForbidRecursion, AllowDelegateCall>,
                PrecompileAt<AddressU64<7>, Bn128Mul, ForbidRecursion, AllowDelegateCall>,
                PrecompileAt<AddressU64<8>, Bn128Pairing, ForbidRecursion, AllowDelegateCall>,
                PrecompileAt<AddressU64<9>, Blake2F, ForbidRecursion, AllowDelegateCall>,
                // Non-wall-e specific nor Ethereum precompiles :
                PrecompileAt<AddressU64<1024>, Sha3FIPS256>,
                // PrecompileAt<AddressU64<1025>, Dispatch<R>>,
                PrecompileAt<AddressU64<1026>, ECRecoverPublicKey>,
                // Wall-e specific precompiles:
            ),
        >,
    ),
>;
