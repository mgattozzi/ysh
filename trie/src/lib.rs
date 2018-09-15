#![feature(option_replace)]

use std::{
    borrow::Borrow,
    collections::hash_map::{HashMap, RandomState},
    hash::{self, Hash},
    mem,
};

pub struct TrieMap<K, V, S = RandomState>
where
    K: Hash + Eq,
    S: hash::BuildHasher,
{
    root: Node<K, V, S>,
}

#[derive(Debug, Default)]
struct Node<K, V, S = RandomState>
where
    K: Hash + Eq,
    S: hash::BuildHasher,
{
    children: HashMap<K, Node<K, V, S>, S>,
    value: Option<V>,
}

impl<K, V> TrieMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            root: Node::new(),
        }
    }
}

impl<K, S, V> TrieMap<K, V, S>
where
    K: Hash + Eq,
    S: hash::BuildHasher + Clone,
{
    pub fn insert<I>(&mut self, key: I, value: V) -> Option<V>
    where
        I: IntoIterator<Item = K>,
    {
        let previous = key.into_iter().fold(&mut self.root, |node, frag| {
            let hasher = node.children.hasher().clone();
            node.children
                .entry(frag)
                .or_insert_with(|| Node::with_hasher(hasher))
        });
        mem::replace(&mut previous.value, Some(value))
    }

    // #[cfg(test)]
    pub fn insert_recursive<I>(&mut self, key: I, value: V) -> Option<V>
    where
        I: IntoIterator<Item = K>,
    {
        self.root.continue_inserting(key.into_iter(), value)
    }
}

impl<K, V> Node<K, V>
where
    K: Hash + Eq,
{
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            value: None,
        }
    }
}

impl<K, S, V> Node<K, V, S>
where
    K: Hash + Eq,
    S: hash::BuildHasher + Clone,
{
    fn with_hasher(state: S) -> Self {
        Self {
            children: HashMap::with_hasher(state),
            value: None,
        }
    }

    // #[cfg(test)]
    #[inline]
    fn continue_inserting<I>(&mut self, mut key: I, value: V) -> Option<V>
    where
        I: Iterator<Item = K>,
    {
        match key.next() {
            None => mem::replace(&mut self.value, Some(value)),
            Some(frag) => {
                let hasher = self.children.hasher().clone();
                self.children
                    .entry(frag)
                    .or_insert_with(|| Node::with_hasher(hasher))
                    .continue_inserting(key, value)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn keys() -> Vec<String> {
        let mut keys = Vec::new();
        for a in 1..=26 {
            for b in 1..=26 {
                for c in 1..=26 {
                    for d in 1..=26 {
                        keys.push(format!("{}{}{}{}", a, b, c, d));
                    }
                }
            }
        }
        keys
    }

    #[test]
    fn insert() {
        let mut trie: TrieMap<char, ()> = TrieMap::new();
        for key in &keys() {
            trie.insert(key.chars(), ());
        }
    }
}
