//! A map to give all the children associated with an item.
//!
//! # Example
//! ```
//! use consolidated_map::ConsolidatedMapBuilder;
//!
//! fn main() {
//!     let mut builder = ConsolidatedMapBuilder::new();
//!
//!     // associate the child 20 with the parent 10.
//!     builder.insert(10usize, 20);
//!
//!     // associate the child 30 with the parent 20.
//!     builder.insert(20, 30);
//!
//!     // build the ConsolidatedMap
//!     let map = builder.build();
//!
//!     // the parent 10 should have the children 20 and 30.
//!     assert_eq!(map.children(10).collect::<Vec<_>>(), vec![20, 30]);
//!
//!     // the parent 20 should have the children 30.
//!     assert_eq!(map.children(20).collect::<Vec<_>>(), vec![30]);
//!
//!     // the parent 30 does not have any children.
//!     assert!(map.children(30).collect::<Vec<_>>().is_empty());
//!
//!     // consolidated children contains also the key item.
//!     assert_eq!(map.consolidated(10).collect::<Vec<_>>(), vec![10, 20, 30]);
//!
//!     // if the item has not been inserted, the consolidated fn returns only the item.
//!     assert_eq!(map.consolidated(5).collect::<Vec<_>>(), vec![5]);
//! }
//! ```
use std::cmp::max;
use std::fmt::Display;
use std::marker::PhantomData;

/// A consolidated map that represent a list of children associated with a key.
///
/// The ConsolidatedMap is readonly and must be build using the ConsolidatedMapBuilder
/// or by and iterator.
pub struct ConsolidatedMap<T> {
    _t: PhantomData<T>,
    data: Vec<u32>,
    index: Vec<usize>,
}

impl<T> Clone for ConsolidatedMap<T> {
    fn clone(&self) -> Self {
        ConsolidatedMap {
            _t: PhantomData,
            data: self.data.clone(),
            index: self.index.clone(),
        }
    }
}

impl<T> ConsolidatedMap<T> {
    /// Returns an iterator containing all the children of an item.
    ///
    /// # Example
    ///
    /// ```
    /// use consolidated_map::ConsolidatedMap;
    ///
    /// // insert 2 as a child on 1 eg (parent, child)
    /// let map: ConsolidatedMap<usize> = vec![(1, 2)].into_iter().collect();
    ///
    /// assert_eq!(map.children(1).collect::<Vec<usize>>(), vec![2]);
    /// assert_eq!(map.children(3).collect::<Vec<usize>>(), Vec::new());
    /// ```
    pub fn children(&self, item: T) -> Children<T>
    where
        T: From<usize> + Into<usize>,
    {
        Children(
            self.get_children_slice(item)
                .map(|s| s.iter())
                .unwrap_or_else(|| [].iter()),
            None,
        )
    }

    /// Returns an iterator containing all the children of an item with the specified item.
    ///
    /// # Example
    ///
    /// ```
    /// use consolidated_map::ConsolidatedMap;
    ///
    /// // insert 2 as a child on 1 eg (parent, child)
    /// let map: ConsolidatedMap<usize> = vec![(1, 2)].into_iter().collect();
    ///
    /// assert_eq!(vec![1, 2], map.consolidated(1).collect::<Vec<_>>());
    /// assert_eq!(vec![3], map.consolidated(3).collect::<Vec<_>>());
    /// ```
    pub fn consolidated(&self, item: T) -> Children<T>
    where
        T: Copy + From<usize> + Into<usize>,
    {
        Children(
            self.get_children_slice(item)
                .map(|s| s.iter())
                .unwrap_or_else(|| [].iter()),
            Some(item),
        )
    }

    /// Returns true if a parent is contains the child.
    pub fn contains_child(&self, parent: T, child: T) -> bool
    where
        T: Into<usize>,
    {
        self.get_children_slice(parent)
            .map(|data| data.contains(&(child.into() as u32)))
            .unwrap_or(false)
    }

    fn get_children_slice(&self, parent: T) -> Option<&[u32]>
    where
        T: Into<usize>,
    {
        let parent = parent.into();
        let index = *self.index.get(parent)? as usize;
        let len = *self.data.get(index)? as usize;
        Some(&self.data[index + 1..index + 1 + len])
    }
}

impl<T> Default for ConsolidatedMap<T> {
    fn default() -> Self {
        ConsolidatedMap {
            _t: PhantomData,
            data: Vec::new(),
            index: Vec::new(),
        }
    }
}

impl<T> std::iter::FromIterator<(T, T)> for ConsolidatedMap<T>
where
    T: Copy + Display + Eq + From<usize> + Into<usize> + Ord,
{
    fn from_iter<I: IntoIterator<Item = (T, T)>>(src: I) -> Self {
        let mut builder = ConsolidatedMapBuilder::new();
        for (parent, child) in src {
            builder.insert(parent, child);
        }
        builder.build()
    }
}

#[cfg(feature = "storm")]
impl<T> storm::Gc for ConsolidatedMap<T> {}

/// An iterator for the children associated with a key.
#[derive(Clone)]
pub struct Children<'a, T>(::std::slice::Iter<'a, u32>, Option<T>);

impl<'a, T> Iterator for Children<'a, T>
where
    T: From<usize>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.1.take() {
            Some(item) => Some(item),
            None => match self.0.next() {
                Some(u) => Some((*u as usize).into()),
                None => None,
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let a = self.0.size_hint();
        let b = self.1.iter().size_hint();

        let upper = match (a.1, b.1) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        };

        (a.0 + b.0, upper)
    }
}

#[derive(Clone)]
struct Entry {
    children: Vec<u32>,
    parent: Option<u32>,
}

/// A builder pattern for the ConsolidatedMap.
pub struct ConsolidatedMapBuilder<T> {
    _t: PhantomData<T>,
    entries: Vec<Entry>,
    len: usize,
}

impl<T> ConsolidatedMapBuilder<T> {
    pub fn new() -> Self {
        ConsolidatedMapBuilder {
            _t: PhantomData,
            entries: Vec::new(),
            len: 0,
        }
    }

    fn ensure(&mut self, v: usize) {
        while self.entries.len() <= v {
            self.entries.push(Entry {
                children: Vec::new(),
                parent: None,
            });
            self.len += 1;
        }
    }

    pub fn insert(&mut self, parent: T, child: T)
    where
        T: Copy + Display + Into<usize>,
    {
        let parent_idx = parent.into();
        let child_idx = child.into();

        if parent_idx != child_idx {
            self.ensure(max(parent_idx, child_idx));

            let entry = {
                let entry = self
                    .entries
                    .get_mut(child_idx)
                    .expect("Unable to find child");
                let parent_idx = parent_idx as u32;

                if entry.children.contains(&parent_idx) {
                    panic!(
                        "Circular reference parent: {}, child: {} failed.",
                        parent, child
                    );
                }

                if let Some(p) = entry.parent {
                    if p != parent_idx {
                        panic!("Child {} has already a parent.", child);
                    }
                    return;
                }

                entry.parent = Some(parent_idx);

                // - do not track the entry reference so that
                //   we can change other reference.
                // - make sure to not change the entries vec to prevent reallocation.
                unsafe { &*(entry as *const Entry) }
            };

            let mut parent = entry.parent;

            while let Some(p) = parent {
                let p = self
                    .entries
                    .get_mut(p as usize)
                    .expect("Unable to find parent in consolidated map.");

                p.children.extend(entry.children.iter().cloned());
                p.children.push(child_idx as u32);

                self.len += entry.children.len() + 1;
                parent = p.parent.clone();
            }
        }
    }

    /// Take the ConsolidatedMapBuilder and return a ConsolidatedMap.
    ///
    /// # Example
    /// ```
    /// use consolidated_map::ConsolidatedMapBuilder;
    ///
    /// let mut builder = ConsolidatedMapBuilder::new();
    ///
    /// // insert child 2 associated with parent 1
    /// builder.insert(1usize, 2usize);
    ///
    /// // construct the ConsolidatedMap
    /// let map = builder.build();
    ///
    /// assert!(map.contains_child(1, 2));
    /// ```
    pub fn build(self) -> ConsolidatedMap<T> {
        let mut data = Vec::with_capacity(self.len);
        let mut index = Vec::with_capacity(self.entries.len());

        for mut entry in self.entries.into_iter() {
            entry.children.sort_unstable();
            index.push(data.len());
            data.push(entry.children.len() as u32);
            data.extend(entry.children.into_iter());
        }

        ConsolidatedMap {
            _t: PhantomData,
            data,
            index,
        }
    }
}

/// Returns an Iterator that gives all the children and the key
/// itself associated with a key.
pub trait ConsolidatedBy<K> {
    fn consolidated_by(&self, key: K) -> Children<K>;
}

impl<K, T> ConsolidatedBy<K> for &T
where
    T: ConsolidatedBy<K>,
{
    fn consolidated_by(&self, key: K) -> Children<K> {
        (*self).consolidated_by(key)
    }
}

impl<K> ConsolidatedBy<K> for ConsolidatedMap<K>
where
    K: Copy + From<usize> + Into<usize>,
{
    fn consolidated_by(&self, key: K) -> Children<K> {
        (*self).consolidated(key)
    }
}
