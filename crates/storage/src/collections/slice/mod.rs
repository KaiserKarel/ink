// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod impls;
mod iter;

use crate::{
    collections::slice::iter::{
        Iter,
        IterMut,
    },
    lazy::{
        LazyArray,
        LazyIndexMap,
    },
    traits::PackedLayout,
};
use core::ops::Range;

/// A view into contiguous storage.
#[derive(Clone, Debug)]
pub struct Slice<T> {
    /// The start and end indices inside the `storage`. Indexing the slice using `n` means that we
    /// access `n + range.start`.
    range: Range<u32>,
    /// The underlying storage structure, such as `LazyIndexMap` or `LazyArray`.
    backing_storage: T,
}

impl<T> Slice<T>
where
    T: ContiguousStorage,
{
    /// Returns the number of elements in the slice.
    pub fn len(&self) -> u32 {
        self.range.end - self.range.start
    }

    /// Returns true if the slice has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.range.end == self.range.start
    }

    /// Returns the first element of the slice, or None if it is empty.
    pub fn first(&self) -> Option<&T::Item> {
        self.get(0)
    }

    /// Returns the first element of the slice, or None if it is empty.
    pub fn last(&self) -> Option<&T::Item> {
        self.get(self.len())
    }

    /// Returns the first and all the rest of the elements of the slice, or None if it is empty.
    pub fn split_first(&self) -> Option<(&T::Item, Slice<&T>)> {
        let first = self.first()?;
        Some((
            first,
            Slice::new(self.range.start + 1..self.range.end, &self.backing_storage),
        ))
    }

    /// Returns the last and all the rest of the elements of the slice, or None if it is empty.
    pub fn split_last(&self) -> Option<(&T::Item, Slice<&T>)> {
        let first = self.last()?;
        Some((
            first,
            Slice::new(self.range.start..self.range.end - 1, &self.backing_storage),
        ))
    }

    pub fn new(range: Range<u32>, backing_storage: T) -> Slice<T> {
        Slice {
            range,
            backing_storage,
        }
    }

    pub fn get(&self, index: u32) -> Option<&T::Item> {
        self.backing_storage.get(index + self.range.start)
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            index: 0,
            range: self.range.clone(),
            backing_storage: &self.backing_storage,
        }
    }

    #[inline]
    pub fn split_at<'a>(&'a self, mid: u32) -> (Slice<&T>, Slice<&T>)
        where
            &'a T: ContiguousStorage,
    {
        assert!(mid <= self.len());
        (
            Slice::new(0..mid, &self.backing_storage),
            Slice::new(mid..self.len(), &self.backing_storage),
        )
    }

    #[inline]
    pub fn split_at_mut<'a>(&'a mut self, mid: u32) -> (SliceMut<&T>, SliceMut<&T>)
        where
            &'a T: ContiguousStorage,
    {
        assert!(mid <= self.len());

        // SAFETY: SliceMut::new requires that the ranges do not overlap.
        unsafe {
            (
                SliceMut::new(0..mid, &self.backing_storage),
                SliceMut::new(mid..self.len(), &self.backing_storage),
            )
        }
    }
}

/// A view into a storage `Vec`.
#[derive(Clone, Debug)]
pub struct SliceMut<T> {
    /// The start and end indices inside the `index_map`. Indexing the slice using `n` means that we
    /// access `n + range.start`.
    range: Range<u32>,
    backing_storage: T,
}

impl<T> SliceMut<T>
where
    T: ContiguousStorage,
{
    /// Returns the number of elements in the slice.
    pub fn len(&self) -> u32 {
        self.range.end - self.range.start
    }

    /// Returns true if the slice has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.range.end == self.range.start
    }

    /// Returns the first element of the slice, or None if it is empty.
    pub fn first(&self) -> Option<&T::Item> {
        self.get(0)
    }

    /// Returns the first element of the slice, or None if it is empty.
    pub fn last(&self) -> Option<&T::Item> {
        self.get(self.len())
    }

    /// Returns the first element of the slice, or None if it is empty.
    pub fn last_mut(&mut self) -> Option<&mut T::Item> {
        self.get_mut(self.len())
    }

    /// Returns the first and all the rest of the elements of the slice, or None if it is empty.
    pub fn first_mut(&mut self) -> Option<&mut T::Item> {
        self.get_mut(0)
    }

    /// Returns the first and all the rest of the elements of the slice, or None if it is empty.
    pub fn split_first(&self) -> Option<(&T::Item, Slice<&T>)> {
        let first = self.first()?;
        Some((
            first,
            Slice::new(self.range.start + 1..self.range.end, &self.backing_storage),
        ))
    }

    /// Returns the first and all the rest of the elements of the slice, or None if it is empty.
    pub fn split_first_mut(&mut self) -> Option<(&mut T::Item, SliceMut<&T>)> {
        let first =
            // Safety: we have exclusive access to the slice through the &mut receiver, thus this
            // mutable borrow is guaranteed to be unique.
            unsafe {
            self.backing_storage.get_mut(self.range.start)?
        };
        Some((
            first,
            // Safety: By taking &mut self, we ensure that other getters of items become cannot be
            // called until the newly returned SliceMut is dropped. Thus only a single slice has
            // mutable access to the underlying items.
            unsafe {
                SliceMut::new(self.range.start + 1..self.range.end, &self.backing_storage)
            },
        ))
    }

    /// Returns the last and all the rest of the elements of the slice, or None if it is empty.
    pub fn split_last_mut(&mut self) -> Option<(&mut T::Item, SliceMut<&T>)> {
        let last =
        // Safety: we have exclusive access to the slice through the &mut receiver, thus this
        // mutable borrow is guaranteed to be unique.
        unsafe {
            self.backing_storage.get_mut(self.range.end)?
        };
        Some((
            last,
            // Safety: By taking &mut self, we ensure that other getters of items become cannot be
            // called until the newly returned SliceMut is dropped. Thus only a single slice has
            // mutable access to the underlying items.
            unsafe {
                SliceMut::new(self.range.start..self.range.end - 1, &self.backing_storage)
            },
        ))
    }

    /// Returns the last and all the rest of the elements of the slice, or None if it is empty.
    pub fn split_last(&self) -> Option<(&T::Item, Slice<&T>)> {
        let first = self.last()?;
        Some((
            first,
            Slice::new(self.range.start..self.range.end - 1, &self.backing_storage),
        ))
    }

    /// Creates a new `SliceMut`.
    ///
    /// # Safety:
    ///
    /// The caller must ensure that mutable slices do not overlap.
    pub(crate) unsafe fn new(range: Range<u32>, backing_storage: T) -> SliceMut<T> {
        SliceMut {
            range,
            backing_storage,
        }
    }

    pub fn get(&self, index: u32) -> Option<&T::Item> {
        self.backing_storage.get(index + self.range.start)
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut T::Item> {
        unsafe { self.backing_storage.get_mut(index) }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            index: 0,
            range: self.range.clone(),
            backing_storage: &self.backing_storage,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            index: 0,
            range: self.range.clone(),
            backing_storage: &self.backing_storage,
        }
    }

    #[inline]
    pub fn split_at<'a>(&'a self, mid: u32) -> (Slice<&T>, Slice<&T>)
    where
        &'a T: ContiguousStorage,
    {
        assert!(mid <= self.len());
        (
            Slice::new(0..mid, &self.backing_storage),
            Slice::new(mid..self.len(), &self.backing_storage),
        )
    }

    #[inline]
    pub fn split_at_mut<'a>(&'a mut self, mid: u32) -> (SliceMut<&T>, SliceMut<&T>)
    where
        &'a T: ContiguousStorage,
    {
        assert!(mid <= self.len());

        // SAFETY: SliceMut::new requires that the ranges do not overlap.
        unsafe {
            (
                SliceMut::new(0..mid, &self.backing_storage),
                SliceMut::new(mid..self.len(), &self.backing_storage),
            )
        }
    }
}

/// Describes collections which can soundly provide multiple mutable references to its items. The
/// canonical example is a slice, where it is sound to obtain a mutable reference to `slice[0]` and
/// `slice[1]`. However `borrowck` has trouble with this through mutable methods such as `IndexMut`,
/// since it cannot prove that there is no overlap.
pub trait ContiguousStorage {
    type Item;

    /// Obtain a mutable reference through an immutable self.
    ///
    /// # Safety
    /// Callers must ensure that only a single mutable reference per `index` exist at a given time.
    unsafe fn get_mut(&self, index: u32) -> Option<&mut Self::Item>;

    /// Obtain the item at `index`.
    fn get(&self, index: u32) -> Option<&Self::Item>;
}

impl<T: ContiguousStorage> ContiguousStorage for &T {
    type Item = T::Item;

    unsafe fn get_mut(&self, index: u32) -> Option<&mut Self::Item> {
        T::get_mut(self, index)
    }

    fn get(&self, index: u32) -> Option<&Self::Item> {
        T::get(self, index)
    }
}

impl<T> ContiguousStorage for &LazyIndexMap<T>
where
    T: PackedLayout,
{
    type Item = T;

    unsafe fn get_mut(&self, index: u32) -> Option<&mut T> {
        // SAFETY:
        //  - lazily_load requires that there is exclusive access to the T. The contract of
        //    ContiguousStorage ensures this variant.
        //  - lazily_load always returns a valid pointer.
        self.lazily_load(index).as_mut().value_mut().as_mut()
    }

    fn get(&self, index: u32) -> Option<&Self::Item> {
        LazyIndexMap::get(self, index)
    }
}

impl<T, const N: usize> ContiguousStorage for &LazyArray<T, N>
where
    T: PackedLayout,
{
    type Item = T;

    unsafe fn get_mut(&self, index: u32) -> Option<&mut T> {
        self.cached_entries
            .get_entry_mut(index)
            .map(|i| i.value_mut().as_mut())
            .flatten()
    }

    fn get(&self, index: u32) -> Option<&Self::Item> {
        LazyArray::get(self, index)
    }
}
