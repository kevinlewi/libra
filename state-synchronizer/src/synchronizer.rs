// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    coordinator::{CoordinatorMessage, SyncCoordinator},
    executor_proxy::{ExecutorProxy, ExecutorProxyTrait},
};
use config::config::{NodeConfig, StateSyncConfig};
use executor::Executor;
use failure::prelude::*;
use futures::{
    channel::{mpsc, oneshot},
    future::Future,
    SinkExt,
};
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use network::validator_network::{StateSynchronizerEvents, StateSynchronizerSender};
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};
use vm_runtime::MoveVM;

pub struct StateSynchronizer {
    _runtime: Runtime,
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSynchronizer {
    /// Setup state synchronizer. spawns coordinator and downloader routines on executor
    pub fn bootstrap(
        network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>,
        executor: Arc<Executor<MoveVM>>,
        config: &NodeConfig,
    ) -> Self {
        let executor_proxy = ExecutorProxy::new(executor, config);
        Self::bootstrap_with_executor_proxy(network, &config.state_sync, executor_proxy)
    }

    pub fn bootstrap_with_executor_proxy<E: ExecutorProxyTrait + 'static>(
        network: Vec<(StateSynchronizerSender, StateSynchronizerEvents)>,
        state_sync_config: &StateSyncConfig,
        executor_proxy: E,
    ) -> Self {
        let runtime = Builder::new()
            .name_prefix("state-sync-")
            .build()
            .expect("[state synchronizer] failed to create runtime");
        let executor = runtime.executor();

        let (coordinator_sender, coordinator_receiver) = mpsc::unbounded();

        let coordinator = SyncCoordinator::new(
            coordinator_receiver,
            state_sync_config.clone(),
            executor_proxy,
        );
        executor.spawn(coordinator.start(network));

        Self {
            _runtime: runtime,
            coordinator_sender,
        }
    }

    pub fn create_client(&self) -> Arc<StateSyncClient> {
        Arc::new(StateSyncClient {
            coordinator_sender: self.coordinator_sender.clone(),
        })
    }
}

pub struct StateSyncClient {
    coordinator_sender: mpsc::UnboundedSender<CoordinatorMessage>,
}

impl StateSyncClient {
    /// Sync validator's state up to given `version`
    pub fn sync_to(&self, target: LedgerInfoWithSignatures) -> impl Future<Output = Result<bool>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        async move {
            sender
                .send(CoordinatorMessage::Requested(target, cb_sender))
                .await?;
            let sync_status = cb_receiver.await?;
            Ok(sync_status)
        }
    }

    /// Notifies state synchronizer about new version
    pub fn commit(&self, version: u64) -> impl Future<Output = Result<()>> {
        let mut sender = self.coordinator_sender.clone();
        async move {
            sender.send(CoordinatorMessage::Commit(version)).await?;
            Ok(())
        }
    }

    /// Returns information about StateSynchronizer internal state
    pub fn get_state(&self) -> impl Future<Output = Result<u64>> {
        let mut sender = self.coordinator_sender.clone();
        let (cb_sender, cb_receiver) = oneshot::channel();
        async move {
            sender.send(CoordinatorMessage::GetState(cb_sender)).await?;
            let info = cb_receiver.await?;
            Ok(info)
        }
    }
}
