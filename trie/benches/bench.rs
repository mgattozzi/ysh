#![feature(test)]
use ysh_trie::TrieMap;
use test::Bencher;

use std::collections::HashMap;

fn keys() -> Vec<String> {
    let mut keys = Vec::new();
    for a in 1..10 {
        for b in 1..10 {
            for c in 1..10 {
                for d in 1..10 {
                    keys.push(format!("{}{}{}{}", a, b, c, d));
                }
            }
        }
    }
    keys
}

#[bench]
fn bench_insert(b: &mut Bencher) {
    let keys = keys();
    b.iter(|| {
        let mut trie: TrieMap<char, ()> = TrieMap::new();
        for key in &keys {
            trie.insert(key.chars(), ());
        }
        trie
    });
}

#[bench]
fn bench_insert_recursive(b: &mut Bencher) {
    let keys = keys();
    b.iter(|| {
        let mut trie: TrieMap<char, ()> = TrieMap::new();
        for key in &keys {
            trie.insert_recursive(key.chars(), ());
        }
        trie
    });
}

#[bench]
fn bench_insert_hashmap(b: &mut Bencher) {
    let keys = keys();
    b.iter(|| {
        let mut map = HashMap::new();
        for key in &keys {
            map.insert(key, ());
        }
        map
    });
}
