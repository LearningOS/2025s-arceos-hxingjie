use core::usize;

use alloc::vec;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

use arceos_api::arceos_random;

//use crate::println;

pub struct HashMap {
    capacity: usize,
    sz: usize,
    random_val: usize,
    items: Vec<Vec<(String, u32)>>,
}

impl HashMap {
    pub fn new() -> Self {
        Self {
            capacity: 13,
            sz: 0,
            random_val: (arceos_random() % (usize::MAX as u128)) as usize,
            items: vec![vec![]; 13], 
        }
    }
    fn hash(&self, bytes: &[u8]) -> usize {
        let mut hash_val: usize = 0;
        for byte in bytes {
            hash_val = hash_val.wrapping_add(*byte as usize);
        }
        (hash_val ^ self.random_val) % self.capacity
    }
    pub fn insert(&mut self, key: String, value: u32) {
        let key_str = key.to_string();
        let idx = self.hash(key_str.as_str().as_bytes());
        self.items[idx].push((key, value));

        self.sz += 1;
    }

    pub fn iter(&self) -> MyHashMapIterator {
        let mut elems: Vec<(&String, &u32)> = Vec::new();
        for list in self.items.iter() {
            for elem in list.iter() {
                elems.push((&elem.0, &elem.1));
            }
        }
        MyHashMapIterator {
            current: 0,
            len: elems.len(),
            data: elems
        }
    }
}

pub struct MyHashMapIterator<'a> {
    current: usize,
    len: usize,
    data: Vec<(&'a String, &'a u32)>,
}
impl<'a> Iterator for MyHashMapIterator<'a> {
    type Item = (&'a String, &'a u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.len {
            None
        } else {
            self.current += 1;
            Some(self.data[self.current-1])
        }
    }
}
