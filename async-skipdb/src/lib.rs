//! Blazing fast ACID and MVCC in memory database.
//!
//! `async-skipdb` uses the same SSI (Serializable Snapshot Isolation) transaction model used in [`badger`](https://github.com/dgraph-io/badger).
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![allow(clippy::type_complexity)]

use std::{borrow::Borrow, hash::BuildHasher, ops::RangeBounds, sync::Arc};

use async_txn::{error::TransactionError, AsyncRtm, AsyncTm, AsyncWtm, HashCm, HashCmOptions};

/// `EquivalentDb` implementation, which requires `K` implements both [`Hash`](core::hash::Hash) and [`Ord`].
/// If your `K` does not implement [`Hash`](core::hash::Hash), you can use [`ComparableDb`] instead.
pub mod equivalent;

/// `ComparableDb` implementation, which requires `K` implements [`Ord`] and [`CheapClone`](cheap_clone::CheapClone). If your `K` implements both [`Hash`](core::hash::Hash) and [`Ord`], you are recommended to use [`EquivalentDb`](crate::equivalent::EquivalentDb) instead.
pub mod comparable;

pub use skipdb_core::{
  iter::*,
  range::*,
  rev_iter::*,
  types::{Ref, ValueRef},
};

use skipdb_core::{AsSkipCore, Database, SkipCore};

mod read;
pub use read::*;

pub use async_txn::{AsyncSpawner, BTreePwm, Detach};

#[cfg(feature = "smol")]
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
pub use async_txn::SmolSpawner;

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub use async_txn::TokioSpawner;

#[cfg(feature = "async-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
pub use async_txn::AsyncStdSpawner;

#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
pub use async_txn::WasmSpawner;
