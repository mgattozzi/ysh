use std::{
    borrow::Borrow,
    iter::FromIterator,
    collections::hash_map::{self, HashMap, RandomState},
    hash::{self, Hash},
    mem,
    marker::PhantomData,
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

pub struct Iter<'trie, K, V, S, Q>
where
    K: Hash + Eq,
    S: hash::BuildHasher,
    Q: FromIterator<&'trie K>,
{
    key: Vec<&'trie K>,
    stack: Vec<hash_map::Iter<'trie, K, Node<K, V, S>>>,
    _q: PhantomData<fn() -> Q>,
}

// pub struct PrefixMatches<'trie, K, V, S, Q>
// where
//     K: Hash + Eq,
//     S: hash::BuildHasher,
// {
//     _k: PhantomData<fn() -> Q>,
//     roots: hash_map::Iter<'trie, K, Node<K, V, S>>,
//     current_children: hash_map::Iter<'trie, K, Node<K, V, S>>,
// }

impl<K, S, V> TrieMap<K, V, S>
where
    K: Hash + Eq,
    S: hash::BuildHasher + Clone,
{

    pub fn prefix_matches<'q, 'trie, I, Q: 'q>(&'trie self, prefix: I) -> Option<Iter<'trie, K, V, S, I>>
    where
        I: IntoIterator<Item = &'q Q> + FromIterator<&'trie K>,
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get_node(&mut prefix.into_iter()).map(Node::iter)
    }

    fn get_node<'q, I, Q: 'q>(&self, key: &mut I) -> Option<&Node<K, V, S>>
    where
        I: Iterator<Item = &'q Q>,
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        key.try_fold(&self.root, Node::get_child)
    }

    fn last_node<'q, I, Q: 'q>(&self, key: &mut I) -> &Node<K, V, S>
    where
        I: Iterator<Item = &'q Q>,
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let mut current_node = &self.root;
        for frag in key {
            if let Some(child) = current_node.get_child(frag) {
                current_node = child;
            }
            break;
        }
        current_node
    }

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

    fn iter<'trie, Q>(&'trie self) -> Iter<'trie, K, V, S, Q>
    where
        Q: FromIterator<&'trie K>
    {
        self.root.iter()
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
    S: hash::BuildHasher,
{
    fn with_hasher(state: S) -> Self {
        Self {
            children: HashMap::with_hasher(state),
            value: None,
        }
    }

    fn children(&self) -> hash_map::Iter<K, Self> {
        self.children.iter()
    }

    fn get_child<'q, Q>(&self, key: &'q Q) ->Option<&Node<K, V, S>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.children.get(key)
    }

    // #[cfg(test)]
    #[inline]
    fn continue_inserting<I>(&mut self, mut key: I, value: V) -> Option<V>
    where
        I: Iterator<Item = K>,
        S: Clone,
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

    fn iter<'trie, Q>(&'trie self) -> Iter<'trie, K, V, S, Q>
    where
        Q: FromIterator<&'trie K>
    {
        let stack = vec![ self.children() ];
        Iter {
            key: Vec::new(),
            stack,
            _q: PhantomData,
        }
    }
}

impl<'trie, K, V, S, Q> Iterator for Iter<'trie, K, V, S, Q>
where
    K: Hash + Eq,
    S: hash::BuildHasher,
    Q: FromIterator<&'trie K>
{
    type Item = (Q, &'trie V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let curr = self.stack.first_mut()?;
            if let Some((k, next_node)) = curr.next() {
                self.stack.push(next_node.children());
                self.key.push(k);
                if let Some(value) = next_node.value.as_ref() {
                    return Some((self.key.iter().cloned().collect(), value));
                }
            } else {
                self.stack.pop();
                self.key.pop();
            }
        }

    }
}

// impl<'trie, K, V, S, Q> Iterator for PrefixMatches<'trie, K, V, S, Q>
// where
//     Q: FromIterator<K>,
//     K: Hash + Eq,
//     S: hash::BuildHasher,
// {
//     type Item = Q;
//     fn next(&mut self) -> Option<Q> {
//         let node = match self.current_children.next() {
//             None => {
//                 self.current_children
//             }
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn iter() {
        let words = &["about", "abbot", "abelian", "alphabet", "alcazar", "crawfish", "crawdad", "crazy"];
        let mut trie: TrieMap<char, usize> = TrieMap::new();

        for (i, word) in words.iter().enumerate() {
            trie.insert(word.chars(), i);
        }

        for ((word1, &i), (j, &word2)) in trie.root.iter::<String>().zip(words.iter().enumerate()) {
            assert_eq!(&word1, word2);
            assert_eq!(i, j);
        }
    }

    // #[test]
    // fn prefix_matches() {
    //     let words = &["about", "abbot", "abelian", "alphabet", "alcazar", "crawfish", "crawdad", "crazy"];
    //     let mut trie: TrieMap<char, ()> = TrieMap::new();

    //     for (i, word) in words.iter().enumerate() {
    //         trie.insert(word.chars(), ());
    //     }

    //     assert_eq!(trie.prefix_matches("ab".to_owned().chars()).collect::<Vec<_>>(), vec![ "out", "bot", "elian"]);
    // }
}
