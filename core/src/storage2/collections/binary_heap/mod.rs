// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

//! Binary heap data structure and utilities.
//!
//! todo: more module description (see original bit_stash and other collections)

mod impls;
mod iter;

#[cfg(test)]
mod tests;

pub use self::iter::Iter;
use crate::storage2::{
    lazy::{
        Lazy,
        LazyIndexMap,
    },
    traits::PackedLayout,
};

/// A binary heap type, providing `O(log(n))` push and pop operations.
///
/// # Note
///
/// The heap is a *max-heap* by default, i.e. the first element is the largest.
/// In order to make it a *min-heap*, implement the `Ord` trait explicitly on the type
/// which is stored in the heap.
#[derive(Debug)]
pub struct BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    /// The length of the vector.
    len: Lazy<u32>,
    /// The synchronized cells to operate on the contract storage.
    elems: LazyIndexMap<T>,
}

impl<T> BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    /// Creates a new empty storage heap.
    pub fn new() -> Self {
        Self {
            len: Lazy::new(0),
            elems: LazyIndexMap::new(),
        }
    }

    /// Returns the number of elements in the heap, also referred to as its 'length'.
    pub fn len(&self) -> u32 {
        *self.len
    }

    /// Returns `true` if the heap contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    /// Returns an iterator yielding shared references to all elements of the heap.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over large heaps.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    /// Returns a shared reference to the greatest element of the heap
    ///
    /// Returns `None` if the heap is empty
    pub fn peek(&self) -> Option<&T> {
        self.elems.get(0)
    }

    /// Pops greatest element from the heap and returns it
    ///
    /// Returns `None` if the heap is empty
    pub fn pop(&mut self) -> Option<T> {
        self.elems.put_get(0, None)
    }
}

impl<T> BinaryHeap<T>
where
    T: PackedLayout + Ord,
{
    // todo: optimize!
    fn bubble_up(&mut self, index: u32) {
        let parent_index = index.saturating_sub(1) / 2;
        let parent = self.elems.get(parent_index)
            .expect("parent must exist in fully compacted sequence of elements");
        let child = self.elems.get(index)
            .expect("child must exist, either just inserted or a previous parent");

        if child > parent {
            self.elems.swap(parent_index, index);
            self.bubble_up(parent_index);
        }
    }

    /// Pushes the given element to the binary heap.
    pub fn push(&mut self, value: T) {
        let index = self.len();
        assert!(
            index < core::u32::MAX,
            "cannot push more elements into the heap"
        );
        *self.len += 1;

        self.elems.put(index, Some(value));

        if index > 0 {
            self.bubble_up(index)
        }
    }
}
