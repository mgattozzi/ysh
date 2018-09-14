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
struct Node<K, V, S>
where
    K: Hash + Eq,
    S: hash::BuildHasher,
{
    children: HashMap<K, Node<K, V, S>, S>,
    value: Option<V>,
}

impl<K, S, V> TrieMap<K, V, S>
where
    K: Hash + Eq,
    S: hash::BuildHasher + Clone,
{
    pub fn insert<'q, I, Q: 'q + ?Sized>(&mut self, key: I, value: V) -> Option<V>
    where
        I: IntoIterator<Item = &'q Q>,
        Q: ToOwned<Owned = K>,
        K: Borrow<Q>,
    {
        let previous = key.into_iter().fold(&mut self.root, |node, frag| {
            let hasher = node.children.hasher().clone();
            node.children
                .entry(frag.to_owned())
                .or_insert_with(|| Node::new(hasher))
        })
        .value;
        mem::replace(previous, Some(value))
    }
}

impl<K, S, V> Node<K, V, S>
where
    K: Hash + Eq,
    S: hash::BuildHasher,
{

    fn new(state: S) -> Self {
        Self {
            children: HashMap::with_hasher(state),
            value: None,
        }
    }
}
