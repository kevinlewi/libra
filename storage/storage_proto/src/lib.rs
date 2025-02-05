// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate provides Protocol Buffers definitions for the services provided by the
//! [`storage_service`](../storage_service/index.html) crate.
//!
//! The protocol is documented in Protocol Buffers sources files in the `.proto` extension and the
//! documentation is not viewable via rustdoc. Refer to the source code to see it.
//!
//! The content provided in this documentation falls to two categories:
//!
//!   1. Those automatically generated by [`grpc-rs`](https://github.com/pingcap/grpc-rs):
//!       * In [`proto::storage`] are structs corresponding to our Protocol Buffers messages.
//!       * In [`proto::storage_grpc`] live the [GRPC](grpc.io) client struct and the service trait
//! which correspond to our Protocol Buffers services.
//!   1. Structs we wrote manually as helpers to ease the manipulation of the above category of
//! structs. By implementing the [`TryFrom`](std::convert::TryFrom) and
//! [`From`](std::convert::From) traits, these structs convert from/to the above category of
//! structs in a single method call and in that process data integrity check can be done. These live
//! right in the root module of this crate (this page).
//!
//! Ihis is provided as a separate crate so that crates that use the storage service via
//! [`storage-client`](../storage-client/index.html) don't need to depending on the entire
//! [`storage_service`](../storage-client/index.html).

pub mod proto;

use crypto::HashValue;
use failure::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    crypto_proxies::LedgerInfoWithSignatures,
    ledger_info::LedgerInfo,
    proof::SparseMerkleProof,
    transaction::{TransactionListWithProof, TransactionToCommit, Version},
};
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use std::convert::{TryFrom, TryInto};

/// Helper to construct and parse [`proto::storage::GetAccountStateWithProofByVersionRequest`]
#[derive(PartialEq, Eq, Clone)]
pub struct GetAccountStateWithProofByVersionRequest {
    /// The access path to query with.
    pub address: AccountAddress,

    /// The version the query is based on.
    pub version: Version,
}

impl GetAccountStateWithProofByVersionRequest {
    /// Constructor.
    pub fn new(address: AccountAddress, version: Version) -> Self {
        Self { address, version }
    }
}

impl TryFrom<crate::proto::storage::GetAccountStateWithProofByVersionRequest>
    for GetAccountStateWithProofByVersionRequest
{
    type Error = Error;

    fn try_from(
        proto: crate::proto::storage::GetAccountStateWithProofByVersionRequest,
    ) -> Result<Self> {
        let address = AccountAddress::try_from(&proto.address[..])?;
        let version = proto.version;

        Ok(Self { address, version })
    }
}

impl From<GetAccountStateWithProofByVersionRequest>
    for crate::proto::storage::GetAccountStateWithProofByVersionRequest
{
    fn from(version: GetAccountStateWithProofByVersionRequest) -> Self {
        Self {
            address: version.address.into(),
            version: version.version,
        }
    }
}

/// Helper to construct and parse [`proto::storage::GetAccountStateWithProofByVersionResponse`]
#[derive(PartialEq, Eq, Clone)]
pub struct GetAccountStateWithProofByVersionResponse {
    /// The account state blob requested.
    pub account_state_blob: Option<AccountStateBlob>,

    /// The state root hash the query is based on.
    pub sparse_merkle_proof: SparseMerkleProof,
}

impl GetAccountStateWithProofByVersionResponse {
    /// Constructor.
    pub fn new(
        account_state_blob: Option<AccountStateBlob>,
        sparse_merkle_proof: SparseMerkleProof,
    ) -> Self {
        Self {
            account_state_blob,
            sparse_merkle_proof,
        }
    }
}

impl TryFrom<crate::proto::storage::GetAccountStateWithProofByVersionResponse>
    for GetAccountStateWithProofByVersionResponse
{
    type Error = Error;

    fn try_from(
        proto: crate::proto::storage::GetAccountStateWithProofByVersionResponse,
    ) -> Result<Self> {
        let account_state_blob = proto
            .account_state_blob
            .map(AccountStateBlob::try_from)
            .transpose()?;
        Ok(Self {
            account_state_blob,
            sparse_merkle_proof: SparseMerkleProof::try_from(
                proto.sparse_merkle_proof.unwrap_or_else(Default::default),
            )?,
        })
    }
}

impl From<GetAccountStateWithProofByVersionResponse>
    for crate::proto::storage::GetAccountStateWithProofByVersionResponse
{
    fn from(response: GetAccountStateWithProofByVersionResponse) -> Self {
        Self {
            account_state_blob: response.account_state_blob.map(Into::into),
            sparse_merkle_proof: Some(response.sparse_merkle_proof.into()),
        }
    }
}

impl Into<(Option<AccountStateBlob>, SparseMerkleProof)>
    for GetAccountStateWithProofByVersionResponse
{
    fn into(self) -> (Option<AccountStateBlob>, SparseMerkleProof) {
        (self.account_state_blob, self.sparse_merkle_proof)
    }
}

/// Helper to construct and parse [`proto::storage::SaveTransactionsRequest`]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct SaveTransactionsRequest {
    pub txns_to_commit: Vec<TransactionToCommit>,
    pub first_version: Version,
    pub ledger_info_with_signatures: Option<LedgerInfoWithSignatures>,
}

impl SaveTransactionsRequest {
    /// Constructor.
    pub fn new(
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_signatures: Option<LedgerInfoWithSignatures>,
    ) -> Self {
        SaveTransactionsRequest {
            txns_to_commit,
            first_version,
            ledger_info_with_signatures,
        }
    }
}

impl TryFrom<crate::proto::storage::SaveTransactionsRequest> for SaveTransactionsRequest {
    type Error = Error;

    fn try_from(proto: crate::proto::storage::SaveTransactionsRequest) -> Result<Self> {
        let txns_to_commit = proto
            .txns_to_commit
            .into_iter()
            .map(TransactionToCommit::try_from)
            .collect::<Result<Vec<_>>>()?;
        let first_version = proto.first_version;
        let ledger_info_with_signatures = proto
            .ledger_info_with_signatures
            .map(LedgerInfoWithSignatures::try_from)
            .transpose()?;

        Ok(Self {
            txns_to_commit,
            first_version,
            ledger_info_with_signatures,
        })
    }
}

impl From<SaveTransactionsRequest> for crate::proto::storage::SaveTransactionsRequest {
    fn from(request: SaveTransactionsRequest) -> Self {
        let txns_to_commit = request.txns_to_commit.into_iter().map(Into::into).collect();
        let first_version = request.first_version;
        let ledger_info_with_signatures = request.ledger_info_with_signatures.map(Into::into);

        Self {
            txns_to_commit,
            first_version,
            ledger_info_with_signatures,
        }
    }
}

/// Helper to construct and parse [`proto::storage::GetTransactionsRequest`]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct GetTransactionsRequest {
    pub start_version: Version,
    pub batch_size: u64,
    pub ledger_version: Version,
    pub fetch_events: bool,
}

impl GetTransactionsRequest {
    /// Constructor.
    pub fn new(
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Self {
        GetTransactionsRequest {
            start_version,
            batch_size,
            ledger_version,
            fetch_events,
        }
    }
}

impl TryFrom<crate::proto::storage::GetTransactionsRequest> for GetTransactionsRequest {
    type Error = Error;

    fn try_from(proto: crate::proto::storage::GetTransactionsRequest) -> Result<Self> {
        Ok(GetTransactionsRequest {
            start_version: proto.start_version,
            batch_size: proto.batch_size,
            ledger_version: proto.ledger_version,
            fetch_events: proto.fetch_events,
        })
    }
}

impl From<GetTransactionsRequest> for crate::proto::storage::GetTransactionsRequest {
    fn from(request: GetTransactionsRequest) -> Self {
        Self {
            start_version: request.start_version,
            batch_size: request.batch_size,
            ledger_version: request.ledger_version,
            fetch_events: request.fetch_events,
        }
    }
}

/// Helper to construct and parse [`proto::storage::GetTransactionsResponse`]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct GetTransactionsResponse {
    pub txn_list_with_proof: TransactionListWithProof,
}

impl GetTransactionsResponse {
    /// Constructor.
    pub fn new(txn_list_with_proof: TransactionListWithProof) -> Self {
        GetTransactionsResponse {
            txn_list_with_proof,
        }
    }
}

impl TryFrom<crate::proto::storage::GetTransactionsResponse> for GetTransactionsResponse {
    type Error = Error;

    fn try_from(proto: crate::proto::storage::GetTransactionsResponse) -> Result<Self> {
        Ok(GetTransactionsResponse {
            txn_list_with_proof: proto
                .txn_list_with_proof
                .unwrap_or_else(Default::default)
                .try_into()?,
        })
    }
}

impl From<GetTransactionsResponse> for crate::proto::storage::GetTransactionsResponse {
    fn from(response: GetTransactionsResponse) -> Self {
        Self {
            txn_list_with_proof: Some(response.txn_list_with_proof.into()),
        }
    }
}

/// Helper to construct and parse [`proto::storage::StartupInfo`]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct StartupInfo {
    pub ledger_info: LedgerInfo,
    pub latest_version: Version,
    pub account_state_root_hash: HashValue,
    pub ledger_frozen_subtree_hashes: Vec<HashValue>,
}

impl TryFrom<crate::proto::storage::StartupInfo> for StartupInfo {
    type Error = Error;

    fn try_from(proto: crate::proto::storage::StartupInfo) -> Result<Self> {
        let ledger_info = LedgerInfo::try_from(proto.ledger_info.unwrap_or_else(Default::default))?;
        let latest_version = proto.latest_version;
        let account_state_root_hash = HashValue::from_slice(&proto.account_state_root_hash[..])?;
        let ledger_frozen_subtree_hashes = proto
            .ledger_frozen_subtree_hashes
            .iter()
            .map(|x| &x[..])
            .map(HashValue::from_slice)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            ledger_info,
            latest_version,
            account_state_root_hash,
            ledger_frozen_subtree_hashes,
        })
    }
}

impl From<StartupInfo> for crate::proto::storage::StartupInfo {
    fn from(info: StartupInfo) -> Self {
        let ledger_info = Some(info.ledger_info.into());
        let latest_version = info.latest_version;
        let account_state_root_hash = info.account_state_root_hash.to_vec();
        let ledger_frozen_subtree_hashes = info
            .ledger_frozen_subtree_hashes
            .into_iter()
            .map(|x| x.to_vec())
            .collect();

        Self {
            ledger_info,
            latest_version,
            account_state_root_hash,
            ledger_frozen_subtree_hashes,
        }
    }
}

/// Helper to construct and parse [`proto::storage::GetStartupInfoResponse`]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct GetStartupInfoResponse {
    pub info: Option<StartupInfo>,
}

impl TryFrom<crate::proto::storage::GetStartupInfoResponse> for GetStartupInfoResponse {
    type Error = Error;

    fn try_from(proto: crate::proto::storage::GetStartupInfoResponse) -> Result<Self> {
        let info = proto.info.map(StartupInfo::try_from).transpose()?;

        Ok(Self { info })
    }
}

impl From<GetStartupInfoResponse> for crate::proto::storage::GetStartupInfoResponse {
    fn from(response: GetStartupInfoResponse) -> Self {
        Self {
            info: response.info.map(Into::into),
        }
    }
}

/// Helper to construct and parse [`proto::storage::GetLatestLedgerInfosPerEpochRequest`]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct GetLatestLedgerInfosPerEpochRequest {
    pub start_epoch: u64,
}

impl GetLatestLedgerInfosPerEpochRequest {
    /// Constructor.
    pub fn new(start_epoch: u64) -> Self {
        Self { start_epoch }
    }
}

impl TryFrom<crate::proto::storage::GetLatestLedgerInfosPerEpochRequest>
    for GetLatestLedgerInfosPerEpochRequest
{
    type Error = Error;

    fn try_from(proto: crate::proto::storage::GetLatestLedgerInfosPerEpochRequest) -> Result<Self> {
        Ok(Self {
            start_epoch: proto.start_epoch,
        })
    }
}

impl From<GetLatestLedgerInfosPerEpochRequest>
    for crate::proto::storage::GetLatestLedgerInfosPerEpochRequest
{
    fn from(request: GetLatestLedgerInfosPerEpochRequest) -> Self {
        Self {
            start_epoch: request.start_epoch,
        }
    }
}

/// Helper to construct and parse [`proto::storage::GetLatestLedgerInfosPerEpochResponse`]
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct GetLatestLedgerInfosPerEpochResponse {
    pub latest_ledger_infos: Vec<LedgerInfoWithSignatures>,
}

impl GetLatestLedgerInfosPerEpochResponse {
    /// Constructor.
    pub fn new(latest_ledger_infos: Vec<LedgerInfoWithSignatures>) -> Self {
        Self {
            latest_ledger_infos,
        }
    }
}

impl TryFrom<crate::proto::storage::GetLatestLedgerInfosPerEpochResponse>
    for GetLatestLedgerInfosPerEpochResponse
{
    type Error = Error;

    fn try_from(
        proto: crate::proto::storage::GetLatestLedgerInfosPerEpochResponse,
    ) -> Result<Self> {
        Ok(Self {
            latest_ledger_infos: proto
                .latest_ledger_infos
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl From<GetLatestLedgerInfosPerEpochResponse>
    for crate::proto::storage::GetLatestLedgerInfosPerEpochResponse
{
    fn from(response: GetLatestLedgerInfosPerEpochResponse) -> Self {
        Self {
            latest_ledger_infos: response
                .latest_ledger_infos
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl Into<Vec<LedgerInfoWithSignatures>> for GetLatestLedgerInfosPerEpochResponse {
    fn into(self) -> Vec<LedgerInfoWithSignatures> {
        self.latest_ledger_infos
    }
}

pub mod prelude {
    pub use super::*;
}

#[cfg(test)]
mod tests;
