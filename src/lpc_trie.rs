use crate::bit_vec::BitVec;

#[derive(Debug)]
struct InternalNode<T> {
    key: T,
    pos: u32,
    // For branching, which indicates must greater than 1.
    bits: u32,
    full_children: u32,
    empty_children: u32,
    child: Vec<TrieNode<T>>,
}

impl<T: BitVec> Default for InternalNode<T> {
    fn default() -> Self {
        InternalNode {
            key: T::empty(),
            pos: 0,
            bits: 0,
            full_children: 0,
            empty_children: 0,
            child: vec![],
        }
    }
}

impl<T: BitVec> InternalNode<T> {
    const HALVE_THRESHOLD: u32 = 25;
    const INFLATE_THRESHOLD: u32 = 50;
    pub fn new(key: T, pos: u32, bits: u32) -> InternalNode<T> {
        InternalNode {
            key,
            pos,
            bits,
            full_children: 0,
            empty_children: 1 << bits,
            child: vec![TrieNode::NONE; 1 << bits],
        }
    }

    pub fn get_mut_child(&mut self, idx: usize) -> &mut TrieNode<T> {
        &mut self.child[idx]
    }

    pub fn get_child(&self, idx: usize) -> &TrieNode<T> {
        &self.child[idx]
    }

    // add a child at position idx overwriting the old value.
    pub fn put_child(&mut self, idx: usize, n: &mut TrieNode<T>) {
        let child: &TrieNode<T> = &self.child[idx];
        if n.is_none() && child.is_some() {
            self.empty_children += 1;
        } else if n.is_some() && child.is_none() {
            self.empty_children -= 1;
        }
        let was_full = self.full(child);
        let is_full = self.full(&n);
        if was_full && !is_full {
            self.full_children -= 1;
        } else if !was_full && is_full {
            self.full_children += 1;
        }
        std::mem::swap(&mut self.child[idx], n);
    }

    fn resize(&mut self) -> TrieNode<T> {
        if self.empty_children as usize == self.child.len() {
            return TrieNode::NONE;
        }
        if self.empty_children as usize == self.child.len() - 1 {
            for i in &mut self.child {
                if i.is_some() {
                    return std::mem::take(i);
                }
            }
        } else {
            while self.full_children > 0
                && 50 * (self.full_children + self.child.len() as u32 - self.empty_children)
                    >= InternalNode::<T>::INFLATE_THRESHOLD * self.child.len() as u32
            {
                self.inflate();
            }

            while self.bits > 1
                && 100 * (self.child.len() as u32 - self.empty_children)
                    < InternalNode::<T>::HALVE_THRESHOLD * self.child.len() as u32
            {
                self.halve();
            }
        }

        if self.empty_children as usize == self.child.len() - 1 {
            for i in &mut self.child {
                if i.is_some() {
                    return std::mem::take(i);
                }
            }
        }
        TrieNode::NODE(Box::new(std::mem::take(self)))
    }

    fn refresh_key(&mut self) {
        for i in &self.child {
            if i.is_some() {
                self.key = i.key();
            }
        }
    }

    fn inflate(&mut self) {
        let mut old_child = std::mem::take(&mut self.child);
        self.bits += 1;
        self.child = vec![TrieNode::NONE; 1 << self.bits];
        self.full_children = 0;
        self.empty_children = 1 << self.bits;
        for (idx, node) in old_child.iter_mut().enumerate() {
            match node {
                TrieNode::NODE(n) if n.pos > self.pos + self.bits - 1 => {
                    if n.key.extract_bits(self.pos + self.bits - 1, 1).is_empty() {
                        self.put_child(2 * idx, node);
                    } else {
                        self.put_child(2 * idx + 1, node);
                    }
                }
                TrieNode::NODE(n) => {
                    if n.bits == 1 {
                        self.put_child(2 * idx, &mut n.child[0]);
                        self.put_child(2 * idx + 1, &mut n.child[1]);
                    } else {
                        let mut left = InternalNode::new(T::empty(), n.pos + 1, n.bits - 1);
                        let mut right = InternalNode::new(T::empty(), n.pos + 1, n.bits - 1);
                        let size = (1 << (n.bits - 1)) as usize;
                        for idx in 0..size {
                            left.put_child(idx, &mut n.child[idx]);
                            right.put_child(idx, &mut n.child[idx + size]);
                        }
                        left.refresh_key();
                        right.refresh_key();
                        self.put_child(2 * idx, &mut left.resize());
                        self.put_child(2 * idx + 1, &mut right.resize());
                    }
                }
                TrieNode::LEAF(n) => {
                    if n.key.extract_bits(self.pos + self.bits - 1, 1).is_empty() {
                        self.put_child(2 * idx, node);
                    } else {
                        self.put_child(2 * idx + 1, node);
                    }
                }
                TrieNode::NONE => {}
            }
        }
    }

    fn halve(&mut self) {
        let mut old_child = std::mem::take(&mut self.child);
        self.bits -= 1;
        self.child = vec![TrieNode::NONE; 1 << self.bits];
        self.full_children = 0;
        self.empty_children = 1 << self.bits;
        let old_length = old_child.len();
        for i in (0..old_length).step_by(2) {
            let left = &old_child[i];
            let right = &old_child[i + 1];
            match left {
                TrieNode::NONE if right.is_none() => {
                    continue;
                }
                TrieNode::NONE if right.is_some() => self.put_child(i / 2, &mut old_child[i + 1]),
                _ => {
                    match right {
                        TrieNode::NONE => self.put_child(i / 2, &mut old_child[i]),
                        // both child is nonempty
                        _ => {
                            let mut binary_node =
                                InternalNode::new(left.key(), self.pos + self.bits, 1);
                            binary_node.put_child(0, &mut old_child[i]);
                            binary_node.put_child(1, &mut old_child[i + 1]);
                            self.put_child(i / 2, &mut binary_node.resize());
                        }
                    }
                }
            }
        }
    }

    fn full(&self, child: &TrieNode<T>) -> bool {
        match child {
            TrieNode::NODE(v) => {
                return v.pos == self.pos + self.bits;
            }
            TrieNode::LEAF(_) => return false,
            _ => false,
        }
    }
}

#[derive(Debug)]
struct Leaf<T> {
    key: T,
    prefix: u8,
    value: String,
}

#[derive(Debug)]
enum TrieNode<T> {
    NODE(Box<InternalNode<T>>),
    LEAF(Box<Leaf<T>>),
    NONE,
}

impl<T: BitVec> TrieNode<T> {
    fn is_none(&self) -> bool {
        match self {
            TrieNode::NONE => true,
            _ => false,
        }
    }
    fn is_some(&self) -> bool {
        match self {
            TrieNode::NONE => false,
            _ => true,
        }
    }

    fn key(&self) -> T {
        match self {
            TrieNode::NODE(n) => n.key,
            TrieNode::LEAF(l) => l.key,
            TrieNode::NONE => T::empty(),
        }
    }
}

impl<T> Default for TrieNode<T> {
    fn default() -> Self {
        TrieNode::NONE
    }
}

impl<T: BitVec> Clone for TrieNode<T> {
    // Only TrieNode NONE is possible clone!
    fn clone(&self) -> Self {
        match self {
            TrieNode::NONE => TrieNode::NONE,
            TrieNode::NODE(_) => {
                unimplemented!()
                //TrieNode::NODE(InternalNode{
                //    key: n.key,
                //    pos: n.pos,
                //    bits: n.bits,
                //    full_children: n.full_children,
                //    empty_children: n.empty_children,
                //    child: n.child.clone()
                //})
            }
            TrieNode::LEAF(_) => {
                unimplemented!();
                //TrieNode::LEAF(Leaf{
                //    key:l.key,
                //    prefix:l.prefix
                //})
            }
        }
    }
}

impl<T: BitVec> TrieNode<T> {
    fn lightweight_clone(&mut self) -> Self {
        match self {
            TrieNode::NONE => TrieNode::NONE,
            TrieNode::NODE(n) => TrieNode::NODE(Box::new(InternalNode {
                key: n.key,
                pos: n.pos,
                bits: n.bits,
                full_children: n.full_children,
                empty_children: n.empty_children,
                child: std::mem::take(&mut n.child),
            })),
            TrieNode::LEAF(l) => TrieNode::LEAF(Box::new(Leaf {
                key: l.key,
                prefix: l.prefix,
                value: std::mem::take(&mut l.value),
            })),
        }
    }
}

pub struct LPCTrie<T> {
    trie: TrieNode<T>,
    size: u32,
    key_found: bool,
}

impl<T: BitVec> LPCTrie<T> {
    pub fn new() -> LPCTrie<T> {
        LPCTrie {
            trie: Default::default(),
            size: 0,
            key_found: false,
        }
    }

    pub fn clear(&mut self) {
        self.trie = TrieNode::NONE;
        self.size = 0;
    }

    pub fn put(&mut self, key: T, prefix: u8, value: String) {
        self.key_found = false;
        let mut trie = std::mem::take(&mut self.trie);
        let trie = self.insert_impl(key, prefix, value, &mut trie, 0);
        self.trie = trie;
        if !self.key_found {
            self.size += 1;
        }
    }

    pub fn get_with_value(&self, key: T) -> &str {
        let mut t: Option<&TrieNode<T>> = Some(&self.trie);
        loop {
            if let Some(node) = t {
                match node {
                    TrieNode::NODE(n) => {
                        t = Some(n.get_child(key.extract_bits(n.pos, n.bits).safe_to_usize()));
                    }
                    TrieNode::LEAF(l) => {
                        // full length prefix
                        return if l.prefix == (std::mem::size_of::<T>() * 8) as u8 && l.key == key {
                            l.value.as_str()
                        } else if l.key.sub_equal(0, l.prefix as u32, &key) {
                            l.value.as_str()
                        } else {
                            ""
                        };
                    }
                    TrieNode::NONE => return "",
                }
            } else {
                return "";
            }
        }
    }
    pub fn get(&self, key: T) -> bool {
        let mut t: Option<&TrieNode<T>> = Some(&self.trie);
        loop {
            if let Some(node) = t {
                match node {
                    TrieNode::NODE(n) => {
                        t = Some(n.get_child(key.extract_bits(n.pos, n.bits).safe_to_usize()));
                    }
                    TrieNode::LEAF(l) => {
                        // full length prefix
                        return if l.prefix == (std::mem::size_of::<T>() * 8) as u8 {
                            l.key == key
                        } else {
                            l.key.sub_equal(0, l.prefix as u32, &key)
                        };
                    }
                    TrieNode::NONE => {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
    }

    pub fn empty(&self) -> bool {
        self.size == 0
    }

    fn insert_impl(
        &mut self,
        key: T,
        prefix: u8,
        value: String,
        trie: &mut TrieNode<T>,
        pos: u32,
    ) -> TrieNode<T> {
        return match trie {
            TrieNode::NODE(inode) if inode.key.sub_equal(pos, inode.pos - pos, &key) => {
                let bitpat = key.extract_bits(inode.pos, inode.bits);
                let insert_pos = inode.pos + inode.bits;
                let mut n = self.insert_impl(
                    key,
                    prefix,
                    value,
                    &mut inode
                        .get_mut_child(bitpat.safe_to_usize())
                        .lightweight_clone(),
                    insert_pos,
                );
                inode.put_child(bitpat.safe_to_usize(), &mut n);
                inode.resize()
            }
            TrieNode::LEAF(l) if key == l.key => {
                self.key_found = true;
                std::mem::take(trie)
            }
            TrieNode::NODE(_) | TrieNode::LEAF(_) => {
                let new_pos = key.mismatch(pos, &trie.key());
                let mut node = InternalNode::new(trie.key(), new_pos, 1);
                let mut leaf = TrieNode::LEAF(Box::new(Leaf { key, prefix, value }));
                if key.extract_bits(new_pos, 1).is_empty() {
                    node.put_child(0, &mut leaf);
                    node.put_child(1, trie);
                } else {
                    node.put_child(0, trie);
                    node.put_child(1, &mut leaf);
                }
                node.resize()
            }
            TrieNode::NONE => TrieNode::LEAF(Box::new(Leaf { key, prefix, value })),
        };
    }
}

#[test]
fn test_lpc_trie() {
    let mut trie = LPCTrie::new();
    let bitvecs: Vec<u32> = vec![
        u32::from_bit_str("00010000"),
        u32::from_bit_str("01000010"),
        u32::from_bit_str("00001010"),
        u32::from_bit_str("00101011"),
        u32::from_bit_str("10101101"),
        u32::from_bit_str("10110110"),
        u32::from_bit_str("11011011"),
        u32::from_bit_str("01101110"),
        u32::from_bit_str("10111010"),
        u32::from_bit_str("11101001"),
        u32::from_bit_str("10100111"),
        u32::from_bit_str("10011110"),
    ];
    for bv in bitvecs {
        trie.put(bv, 1, "fake".to_string());
    }
    assert_eq!(trie.get(u32::from_bit_str("00010000")), true);
    assert_eq!(trie.get(u32::from_bit_str("01000010")), true);
    assert_eq!(trie.get(u32::from_bit_str("00001010")), true);
    assert_eq!(trie.get(u32::from_bit_str("00101011")), true);
    assert_eq!(trie.get(u32::from_bit_str("10101101")), true);
    assert_eq!(trie.get(u32::from_bit_str("10110110")), true);
    assert_eq!(trie.get(u32::from_bit_str("11011011")), true);
    assert_eq!(trie.get(u32::from_bit_str("01101110")), true);
    assert_eq!(trie.get(u32::from_bit_str("10111010")), true);
    assert_eq!(trie.get(u32::from_bit_str("11101001")), true);
    assert_eq!(trie.get(u32::from_bit_str("10100111")), true);
    assert_eq!(trie.get(u32::from_bit_str("10011110")), true);

    let mut trie = LPCTrie::new();
    let bitvecs: Vec<u64> = vec![
        u64::from_bit_str("00010000"),
        u64::from_bit_str("01000010"),
        u64::from_bit_str("00001010"),
        u64::from_bit_str("00101011"),
        u64::from_bit_str("10101101"),
        u64::from_bit_str("10110110"),
        u64::from_bit_str("11011011"),
        u64::from_bit_str("01101110"),
        u64::from_bit_str("10111010"),
        u64::from_bit_str("11101001"),
        u64::from_bit_str("10100111"),
        u64::from_bit_str("10011110"),
    ];
    for bv in bitvecs {
        trie.put(bv, 1, "fake".to_string());
    }
    assert_eq!(trie.get(u64::from_bit_str("00010000")), true);
    assert_eq!(trie.get(u64::from_bit_str("01000010")), true);
    assert_eq!(trie.get(u64::from_bit_str("00001010")), true);
    assert_eq!(trie.get(u64::from_bit_str("00101011")), true);
    assert_eq!(trie.get(u64::from_bit_str("10101101")), true);
    assert_eq!(trie.get(u64::from_bit_str("10110110")), true);
    assert_eq!(trie.get(u64::from_bit_str("11011011")), true);
    assert_eq!(trie.get(u64::from_bit_str("01101110")), true);
    assert_eq!(trie.get(u64::from_bit_str("10111010")), true);
    assert_eq!(trie.get(u64::from_bit_str("11101001")), true);
    assert_eq!(trie.get(u64::from_bit_str("10100111")), true);
    assert_eq!(trie.get(u64::from_bit_str("10011110")), true);

    let mut trie = LPCTrie::new();

    let bitvecs: Vec<u128> = vec![
        u128::from_bit_str("00010000"),
        u128::from_bit_str("01000010"),
        u128::from_bit_str("00001010"),
        u128::from_bit_str("00101011"),
        u128::from_bit_str("10101101"),
        u128::from_bit_str("10110110"),
        u128::from_bit_str("11011011"),
        u128::from_bit_str("01101110"),
        u128::from_bit_str("10111010"),
        u128::from_bit_str("11101001"),
        u128::from_bit_str("10100111"),
        u128::from_bit_str("10011110"),
    ];
    for bv in bitvecs {
        trie.put(bv, 7, "fake".to_string());
    }
    assert_eq!(trie.get(u128::from_bit_str("00110000")), false);
    assert_eq!(trie.get(u128::from_bit_str("00010000")), true);
    assert_eq!(trie.get(u128::from_bit_str("01000010")), true);
    assert_eq!(trie.get(u128::from_bit_str("00001010")), true);
    assert_eq!(trie.get(u128::from_bit_str("00101011")), true);
    assert_eq!(trie.get(u128::from_bit_str("10101101")), true);
    assert_eq!(trie.get(u128::from_bit_str("10110110")), true);
    assert_eq!(trie.get(u128::from_bit_str("11011011")), true);
    assert_eq!(trie.get(u128::from_bit_str("01101110")), true);
    assert_eq!(trie.get(u128::from_bit_str("10111010")), true);
    assert_eq!(trie.get(u128::from_bit_str("11101001")), true);
    assert_eq!(trie.get(u128::from_bit_str("10100111")), true);
    assert_eq!(trie.get(u128::from_bit_str("10011110")), true);
    assert_eq!(trie.get(u128::from_bit_str("10011100")), false);
}
