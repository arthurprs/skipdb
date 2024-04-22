use std::{borrow::Borrow, ops::RangeBounds};

use indexmap::IndexSet;
use smallvec_wrapper::TinyVec;

use super::{types::*, *};

mod hash_cm;
pub use hash_cm::*;

mod btree_cm;
pub use btree_cm::*;

/// Default hasher used by the conflict manager.
pub type DefaultHasher = std::hash::DefaultHasher;

/// A marker used to mark the keys that are read.
pub struct Marker<'a, C> {
  marker: &'a mut C,
}

impl<'a, C> Marker<'a, C> {
  /// Returns a new marker.
  #[inline]
  pub fn new(marker: &'a mut C) -> Self {
    Self { marker }
  }
}

impl<'a, C: Cm> Marker<'a, C> {
  /// Marks a key is operated.
  pub fn mark(&mut self, k: &C::Key) {
    self.marker.mark_read(k);
  }
}

/// The conflict manager that can be used to manage the conflicts in a transaction.
///
/// The conflict normally needs to have:
///
/// 1. Contains fingerprints of keys read.
/// 2. Contains fingerprints of keys written. This is used for conflict detection.
pub trait Cm: Sized {
  /// The error type returned by the conflict manager.
  type Error: std::error::Error;

  /// The key type.
  type Key;

  /// The options type used to create the conflict manager.
  type Options;

  /// Create a new conflict manager with the given options.
  fn new(options: Self::Options) -> Result<Self, Self::Error>;

  /// Mark the key is read.
  fn mark_read(&mut self, key: &Self::Key);

  /// Mark the key is .
  fn mark_conflict(&mut self, key: &Self::Key);

  /// Returns true if we have a conflict.
  fn has_conflict(&self, other: &Self) -> bool;

  /// Rollback the conflict manager.
  fn rollback(&mut self) -> Result<(), Self::Error>;
}

/// An optimized version of the [`Cm`] trait that if your conflict manager is depend on hash.
pub trait CmEquivalent: Cm {
  /// Optimized version of [`mark_read`] that accepts borrowed keys. Optional to implement.
  fn mark_read_equivalent<Q>(&mut self, key: &Q)
  where
    Self::Key: core::borrow::Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized;

  /// Optimized version of [`mark_conflict`] that accepts borrowed keys. Optional to implement.
  fn mark_conflict_equivalent<Q>(&mut self, key: &Q)
  where
    Self::Key: core::borrow::Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized;
}

/// An optimized version of the [`Cm`] trait that if your conflict manager is depend on the order.
pub trait CmComparable: Cm {
  /// Optimized version of [`mark_read`] that accepts borrowed keys. Optional to implement.
  fn mark_read_comparable<Q>(&mut self, key: &Q)
  where
    Self::Key: core::borrow::Borrow<Q>,
    Q: Ord + ?Sized;

  /// Optimized version of [`mark_conflict`] that accepts borrowed keys. Optional to implement.
  fn mark_conflict_comparable<Q>(&mut self, key: &Q)
  where
    Self::Key: core::borrow::Borrow<Q>,
    Q: Ord + ?Sized;
}

/// A pending writes manager that can be used to store pending writes in a transaction.
///
/// By default, there are two implementations of this trait:
/// - [`IndexMap`]: A hash map with consistent ordering and fast lookups.
/// - [`BTreeMap`]: A balanced binary tree with ordered keys and fast lookups.
///
/// But, users can create their own implementations by implementing this trait.
/// e.g. if you want to implement a recovery transaction manager, you can use a persistent
/// storage to store the pending writes.
pub trait Pwm: Sized {
  /// The error type returned by the conflict manager.
  type Error: std::error::Error;

  /// The key type.
  type Key;
  /// The value type.
  type Value;

  /// The iterator type.
  type Iter<'a>: Iterator<Item = (&'a Self::Key, &'a EntryValue<Self::Value>)>
  where
    Self: 'a;

  /// The IntoIterator type.
  type IntoIter: Iterator<Item = (Self::Key, EntryValue<Self::Value>)>;

  /// The options type used to create the pending manager.
  type Options;

  /// Create a new pending manager with the given options.
  fn new(options: Self::Options) -> Result<Self, Self::Error>;

  /// Returns true if the buffer is empty.
  fn is_empty(&self) -> bool;

  /// Returns the number of elements in the buffer.
  fn len(&self) -> usize;

  /// Validate if the entry is valid for this database.
  ///
  /// e.g.
  /// - If the entry is expired
  /// - If the key or the value is too large
  /// - If the key or the value is empty
  /// - If the key or the value contains invalid characters
  /// - and etc.
  fn validate_entry(&self, entry: &Entry<Self::Key, Self::Value>) -> Result<(), Self::Error>;

  /// Returns the maximum batch size in bytes
  fn max_batch_size(&self) -> u64;

  /// Returns the maximum entries in batch
  fn max_batch_entries(&self) -> u64;

  /// Returns the estimated size of the entry in bytes when persisted in the database.
  fn estimate_size(&self, entry: &Entry<Self::Key, Self::Value>) -> u64;

  /// Returns a reference to the value corresponding to the key.
  fn get(&self, key: &Self::Key) -> Result<Option<&EntryValue<Self::Value>>, Self::Error>;

  /// Returns true if the pending manager contains the key.
  fn contains_key(&self, key: &Self::Key) -> Result<bool, Self::Error>;

  /// Inserts a key-value pair into the er.
  fn insert(&mut self, key: Self::Key, value: EntryValue<Self::Value>) -> Result<(), Self::Error>;

  /// Removes a key from the pending writes, returning the key-value pair if the key was previously in the pending writes.
  fn remove_entry(
    &mut self,
    key: &Self::Key,
  ) -> Result<Option<(Self::Key, EntryValue<Self::Value>)>, Self::Error>;

  /// Returns an iterator over the pending writes.
  fn iter(&self) -> Self::Iter<'_>;

  /// Returns an iterator that consumes the pending writes.
  fn into_iter(self) -> Self::IntoIter;

  /// Rollback the pending writes.
  fn rollback(&mut self) -> Result<(), Self::Error>;
}

/// An trait that can be used to get a range over the pending writes.
pub trait PwmRange: Pwm {
  /// The iterator type.
  type Range<'a>: IntoIterator<Item = (&'a Self::Key, &'a EntryValue<Self::Value>)>
  where
    Self: 'a;

  /// Returns an iterator over the pending writes.
  fn range<R: RangeBounds<Self::Key>>(&self, range: R) -> Self::Range<'_>;
}

/// An trait that can be used to get a range over the pending writes.
pub trait PwmComparableRange: PwmRange + PwmComparable {
  /// Returns an iterator over the pending writes.
  fn range_comparable<T, R>(&self, range: R) -> Self::Range<'_>
  where
    T: ?Sized + Ord,
    Self::Key: Borrow<T> + Ord,
    R: RangeBounds<T>;
}

/// An trait that can be used to get a range over the pending writes.
pub trait PwmEquivalentRange: PwmRange + PwmEquivalent {
  /// Returns an iterator over the pending writes.
  fn range_equivalent<T, R>(&self, range: R) -> Self::Range<'_>
  where
    T: ?Sized + Eq + core::hash::Hash,
    Self::Key: Borrow<T> + Eq + core::hash::Hash,
    R: RangeBounds<T>;
}

/// An optimized version of the [`Pwm`] trait that if your pending writes manager is depend on hash.
pub trait PwmEquivalent: Pwm {
  /// Optimized version of [`Pwm::get`] that accepts borrowed keys.
  fn get_equivalent<Q>(&self, key: &Q) -> Result<Option<&EntryValue<Self::Value>>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized;

  fn get_entry_equivalent<Q>(
    &self,
    key: &Q,
  ) -> Result<Option<(&Self::Key, &EntryValue<Self::Value>)>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized;

  /// Optimized version of [`Pwm::contains_key`] that accepts borrowed keys.
  fn contains_key_equivalent<Q>(&self, key: &Q) -> Result<bool, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized;

  /// Optimized version of [`Pwm::remove_entry`] that accepts borrowed keys.
  fn remove_entry_equivalent<Q>(
    &mut self,
    key: &Q,
  ) -> Result<Option<(Self::Key, EntryValue<Self::Value>)>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized;
}

/// An optimized version of the [`Pwm`] trait that if your pending writes manager is depend on the order.
pub trait PwmComparable: Pwm {
  /// Optimized version of [`Pwm::get`] that accepts borrowed keys.
  fn get_comparable<Q>(&self, key: &Q) -> Result<Option<&EntryValue<Self::Value>>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: Ord + ?Sized;

  fn get_entry_comparable<Q>(
    &self,
    key: &Q,
  ) -> Result<Option<(&Self::Key, &EntryValue<Self::Value>)>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: Ord + ?Sized;

  /// Optimized version of [`Pwm::contains_key`] that accepts borrowed keys.
  fn contains_key_comparable<Q>(&self, key: &Q) -> Result<bool, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: Ord + ?Sized;

  /// Optimized version of [`Pwm::remove_entry`] that accepts borrowed keys.
  fn remove_entry_comparable<Q>(
    &mut self,
    key: &Q,
  ) -> Result<Option<(Self::Key, EntryValue<Self::Value>)>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: Ord + ?Sized;
}

/// A type alias for [`Pwm`] that based on the [`IndexMap`].
pub type IndexMapManager<K, V, S = std::hash::RandomState> = IndexMap<K, EntryValue<V>, S>;
/// A type alias for [`Pwm`] that based on the [`BTreeMap`].
pub type BTreeMapManager<K, V> = BTreeMap<K, EntryValue<V>>;

impl<K, V, S> Pwm for IndexMap<K, EntryValue<V>, S>
where
  K: Eq + core::hash::Hash,
  S: BuildHasher + Default,
{
  type Error = std::convert::Infallible;
  type Key = K;
  type Value = V;
  type Iter<'a> = indexmap::map::Iter<'a, K, EntryValue<V>> where Self: 'a;
  type IntoIter = indexmap::map::IntoIter<K, EntryValue<V>>;

  type Options = Option<S>;

  fn new(options: Self::Options) -> Result<Self, Self::Error> {
    Ok(match options {
      Some(hasher) => Self::with_hasher(hasher),
      None => Self::default(),
    })
  }

  fn is_empty(&self) -> bool {
    self.is_empty()
  }

  fn len(&self) -> usize {
    self.len()
  }

  fn validate_entry(&self, _entry: &Entry<Self::Key, Self::Value>) -> Result<(), Self::Error> {
    Ok(())
  }

  fn max_batch_size(&self) -> u64 {
    u64::MAX
  }

  fn max_batch_entries(&self) -> u64 {
    u64::MAX
  }

  fn estimate_size(&self, _entry: &Entry<Self::Key, Self::Value>) -> u64 {
    core::mem::size_of::<Self::Key>() as u64 + core::mem::size_of::<Self::Value>() as u64
  }

  fn get(&self, key: &K) -> Result<Option<&EntryValue<V>>, Self::Error> {
    Ok(self.get(key))
  }

  fn contains_key(&self, key: &K) -> Result<bool, Self::Error> {
    Ok(self.contains_key(key))
  }

  fn insert(&mut self, key: K, value: EntryValue<V>) -> Result<(), Self::Error> {
    self.insert(key, value);
    Ok(())
  }

  fn remove_entry(&mut self, key: &K) -> Result<Option<(K, EntryValue<V>)>, Self::Error> {
    Ok(self.shift_remove_entry(key))
  }
  fn iter(&self) -> Self::Iter<'_> {
    IndexMap::iter(self)
  }

  fn into_iter(self) -> Self::IntoIter {
    core::iter::IntoIterator::into_iter(self)
  }

  fn rollback(&mut self) -> Result<(), Self::Error> {
    self.clear();
    Ok(())
  }
}

impl<K, V, S> PwmEquivalent for IndexMap<K, EntryValue<V>, S>
where
  K: Eq + core::hash::Hash,
  S: BuildHasher + Default,
{
  fn get_equivalent<Q>(&self, key: &Q) -> Result<Option<&EntryValue<V>>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized,
  {
    Ok(self.get(key))
  }

  fn get_entry_equivalent<Q>(
    &self,
    key: &Q,
  ) -> Result<Option<(&Self::Key, &EntryValue<Self::Value>)>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized,
  {
    Ok(self.get_full(key).map(|(_, k, v)| (k, v)))
  }

  fn contains_key_equivalent<Q>(&self, key: &Q) -> Result<bool, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized,
  {
    Ok(self.contains_key(key))
  }

  fn remove_entry_equivalent<Q>(
    &mut self,
    key: &Q,
  ) -> Result<Option<(K, EntryValue<V>)>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized,
  {
    Ok(self.shift_remove_entry(key))
  }
}

impl<K, V> Pwm for BTreeMap<K, EntryValue<V>>
where
  K: Ord,
{
  type Error = std::convert::Infallible;
  type Key = K;
  type Value = V;

  type Iter<'a> = std::collections::btree_map::Iter<'a, K, EntryValue<V>> where Self: 'a;

  type IntoIter = std::collections::btree_map::IntoIter<K, EntryValue<V>>;

  type Options = ();

  fn new(_: Self::Options) -> Result<Self, Self::Error> {
    Ok(Self::default())
  }

  fn is_empty(&self) -> bool {
    self.is_empty()
  }

  fn len(&self) -> usize {
    self.len()
  }

  fn validate_entry(&self, _entry: &Entry<Self::Key, Self::Value>) -> Result<(), Self::Error> {
    Ok(())
  }

  fn max_batch_size(&self) -> u64 {
    u64::MAX
  }

  fn max_batch_entries(&self) -> u64 {
    u64::MAX
  }

  fn estimate_size(&self, _entry: &Entry<Self::Key, Self::Value>) -> u64 {
    core::mem::size_of::<Self::Key>() as u64 + core::mem::size_of::<Self::Value>() as u64
  }

  fn get(&self, key: &K) -> Result<Option<&EntryValue<Self::Value>>, Self::Error> {
    Ok(self.get(key))
  }

  fn contains_key(&self, key: &K) -> Result<bool, Self::Error> {
    Ok(self.contains_key(key))
  }

  fn insert(&mut self, key: K, value: EntryValue<Self::Value>) -> Result<(), Self::Error> {
    self.insert(key, value);
    Ok(())
  }

  fn remove_entry(&mut self, key: &K) -> Result<Option<(K, EntryValue<Self::Value>)>, Self::Error> {
    Ok(self.remove_entry(key))
  }
  fn iter(&self) -> Self::Iter<'_> {
    BTreeMap::iter(self)
  }

  fn into_iter(self) -> Self::IntoIter {
    core::iter::IntoIterator::into_iter(self)
  }

  fn rollback(&mut self) -> Result<(), Self::Error> {
    self.clear();
    Ok(())
  }
}

impl<K, V> PwmRange for BTreeMap<K, EntryValue<V>>
where
  K: Ord,
{
  type Range<'a> = std::collections::btree_map::Range<'a, K, EntryValue<V>> where Self: 'a;

  fn range<R: RangeBounds<Self::Key>>(&self, range: R) -> Self::Range<'_> {
    BTreeMap::range(self, range)
  }
}

impl<K, V> PwmComparableRange for BTreeMap<K, EntryValue<V>>
where
  K: Ord,
{
  fn range_comparable<T, R>(&self, range: R) -> Self::Range<'_>
  where
    T: ?Sized + Ord,
    Self::Key: Borrow<T> + Ord,
    R: RangeBounds<T>,
  {
    BTreeMap::range(self, range)
  }
}

impl<K, V> PwmComparable for BTreeMap<K, EntryValue<V>>
where
  K: Ord,
{
  fn get_comparable<Q>(&self, key: &Q) -> Result<Option<&EntryValue<Self::Value>>, Self::Error>
  where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
  {
    Ok(BTreeMap::get(self, key))
  }

  fn get_entry_comparable<Q>(
    &self,
    key: &Q,
  ) -> Result<Option<(&Self::Key, &EntryValue<Self::Value>)>, Self::Error>
  where
    Self::Key: Borrow<Q>,
    Q: Ord + ?Sized,
  {
    Ok(BTreeMap::get_key_value(self, key))
  }

  fn contains_key_comparable<Q>(&self, key: &Q) -> Result<bool, Self::Error>
  where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
  {
    Ok(BTreeMap::contains_key(self, key))
  }

  fn remove_entry_comparable<Q>(
    &mut self,
    key: &Q,
  ) -> Result<Option<(K, EntryValue<V>)>, Self::Error>
  where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
  {
    Ok(BTreeMap::remove_entry(self, key))
  }
}
