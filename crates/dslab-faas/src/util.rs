use std::hash::BuildHasherDefault;
use std::ops::{Index, IndexMut};

use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHasher;

#[derive(Default)]
pub struct Counter {
    value: u64,
}

impl Counter {
    pub fn curr(&self) -> u64 {
        self.value
    }

    pub fn increment(&mut self) -> u64 {
        let curr = self.value;
        self.value += 1;
        curr
    }
}

#[derive(Clone)]
/// A simple mapping type for storing (key, value) pairs where the keys are assumed to be integers taken
/// from some unknown but not very big interval [0, MAX_KEY]. This map does not support deletion.
pub struct VecMap<T> {
    data: Vec<Option<T>>,
}

impl<T> Default for VecMap<T> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<T> VecMap<T> {
    pub fn insert(&mut self, id: usize, value: T) {
        while self.data.len() <= id {
            self.data.push(None);
        }
        self.data[id] = Some(value);
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        if id < self.data.len() {
            self.data[id].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        if id < self.data.len() {
            self.data[id].as_mut()
        } else {
            None
        }
    }
}

impl<T> Index<usize> for VecMap<T> {
    type Output = T;

    fn index(&self, id: usize) -> &Self::Output {
        self.data[id].as_ref().unwrap()
    }
}

impl<T> IndexMut<usize> for VecMap<T> {
    fn index_mut(&mut self, id: usize) -> &mut Self::Output {
        self.data[id].as_mut().unwrap()
    }
}

/// Similar to VecMap, but returns the default value instead of None and auto-extends to keys in
/// `get_mut` query.
#[derive(Clone, Default)]
pub struct DefaultVecMap<T: Default> {
    data: Vec<T>,
}

impl<T> DefaultVecMap<T>
where
    T: Default,
{
    fn extend(&mut self, id: usize) {
        while self.data.len() <= id {
            self.data.push(Default::default());
        }
    }

    pub fn insert(&mut self, id: usize, value: T) {
        self.extend(id);
        self.data[id] = value;
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        if id < self.data.len() {
            Some(&self.data[id])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: usize) -> &mut T {
        self.extend(id);
        &mut self.data[id]
    }
}

impl<T> Index<usize> for DefaultVecMap<T>
where
    T: Default,
{
    type Output = T;

    fn index(&self, id: usize) -> &Self::Output {
        &self.data[id]
    }
}

impl<T> IndexMut<usize> for DefaultVecMap<T>
where
    T: Default,
{
    fn index_mut(&mut self, id: usize) -> &mut Self::Output {
        &mut self.data[id]
    }
}

pub type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;
