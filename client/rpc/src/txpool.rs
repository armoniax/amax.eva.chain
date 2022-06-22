use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use ethereum_types::{H160, H256, U256};
use jsonrpsee::core::RpcResult;
use serde::Serialize;

// Substrate
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::InPoolTransaction;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_core::hashing::keccak_256;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

// Frontier
use fc_rpc::{internal_err, public_key};

// Local
pub use amax_eva_rpc_core::{
    Content, Get as GetT, Inspect, TransactionMap, TxPoolApiServer, TxPoolResult,
};
pub use primitives_rpc::txpool::{TxPoolResponse, TxPoolRuntimeApi};

/// Geth `txpool` API implementation.
pub struct TxPool<B: BlockT, C, A: ChainApi> {
    client: Arc<C>,
    graph: Arc<Pool<A>>,
    _marker: PhantomData<B>,
}

impl<B: BlockT, C, A: ChainApi> TxPool<B, C, A> {
    pub fn new(client: Arc<C>, graph: Arc<Pool<A>>) -> Self {
        Self { client, graph, _marker: PhantomData }
    }
}

impl<B, C, A> TxPool<B, C, A>
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    C: ProvideRuntimeApi<B>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B> + 'static,
    C::Api: TxPoolRuntimeApi<B>,
    A: ChainApi<Block = B> + 'static,
{
    /// Use the transaction graph interface to get the extrinsics currently in the ready and future
    /// queues.
    fn map_build<T>(&self) -> RpcResult<TxPoolResult<TransactionMap<T>>>
    where
        T: GetT + Serialize,
    {
        // Collect transactions in the ready validated pool.
        let txs_ready = self
            .graph
            .validated_pool()
            .ready()
            .map(|in_pool_tx| in_pool_tx.data().clone())
            .collect::<Vec<_>>();

        // Collect transactions in the future validated pool.
        let txs_future = self
            .graph
            .validated_pool()
            .futures()
            .iter()
            .map(|(_hash, extrinsic)| extrinsic.clone())
            .collect::<Vec<_>>();

        // Use the runtime to match the (here) opaque extrinsics against ethereum transactions.
        let best_block: BlockId<B> = BlockId::Hash(self.client.info().best_hash);
        let ethereum_txns: TxPoolResponse = self
            .client
            .runtime_api()
            .extrinsic_filter(&best_block, txs_ready, txs_future)
            .map_err(|err| {
                internal_err(format!("fetch runtime extrinsic filter failed: {:?}", err))
            })?;

        // Build the T response.
        let mut pending = TransactionMap::<T>::new();
        for txn in ethereum_txns.ready.iter() {
            let hash = H256::from(keccak_256(&rlp::encode(txn)));
            let from = match public_key(txn) {
                Ok(pk) => H160::from(H256::from(keccak_256(&pk))),
                Err(_e) => H160::default(),
            };
            pending
                .entry(from)
                .or_insert_with(HashMap::new)
                .insert(T::nonce(txn), T::get(hash, from, txn));
        }

        let mut queued = TransactionMap::<T>::new();
        for txn in ethereum_txns.future.iter() {
            let hash = H256::from(keccak_256(&rlp::encode(txn)));
            let from = match public_key(txn) {
                Ok(pk) => H160::from(H256::from(keccak_256(&pk))),
                Err(_e) => H160::default(),
            };
            queued
                .entry(from)
                .or_insert_with(HashMap::new)
                .insert(T::nonce(txn), T::get(hash, from, txn));
        }

        Ok(TxPoolResult { pending, queued })
    }
}

impl<B, C, A> TxPoolApiServer for TxPool<B, C, A>
where
    B: BlockT<Hash = H256> + Send + Sync + 'static,
    C: ProvideRuntimeApi<B>,
    C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B> + 'static,
    C::Api: TxPoolRuntimeApi<B>,
    A: ChainApi<Block = B> + 'static,
{
    fn content(&self) -> RpcResult<TxPoolResult<TransactionMap<Content>>> {
        self.map_build::<Content>()
    }

    fn inspect(&self) -> RpcResult<TxPoolResult<TransactionMap<Inspect>>> {
        self.map_build::<Inspect>()
    }

    fn status(&self) -> RpcResult<TxPoolResult<U256>> {
        let status = self.graph.validated_pool().status();
        Ok(TxPoolResult { pending: U256::from(status.ready), queued: U256::from(status.future) })
    }
}
