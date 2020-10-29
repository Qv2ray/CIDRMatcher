mod lpc_trie {
    use crate::bit_vec::BitVector;

    #[derive(Debug)]
    struct InternalNode {
        key: BitVector,
        pos: u32,
        // For branching, which indicates must greater than 1.
        bits: u32,
        full_children: u32,
        empty_children: u32,
        child: Vec<TrieNode>,
    }

    impl Default for InternalNode{
        fn default() -> Self {
            InternalNode{
                key: BitVector::empty(),
                pos: 0,
                bits: 0,
                full_children: 0,
                empty_children: 0,
                child: vec![]
            }
        }
    }

    impl InternalNode {
        const HALVE_THRESHOLD:u32 = 25;
        const INFLATE_THRESHOLD:u32 = 50;
        pub fn new(key:BitVector,pos:u32,bits:u32)->InternalNode{
            InternalNode{
                key,
                pos,
                bits,
                full_children: 0,
                empty_children: 1<<bits,
                child: vec![TrieNode::NONE;1<<bits]
            }
        }

        pub fn get_mut_child(&mut self, idx:u32) ->&mut TrieNode{
            &mut self.child[idx as usize]
        }

        pub fn get_child(&self, idx:u32) ->&TrieNode{
            &self.child[idx as usize]
        }

        // add a child at position idx overwriting the old value.
        pub fn put_child(&mut self, idx: usize, n:& mut TrieNode) {
            let child: &TrieNode = &self.child[idx];
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

        fn resize(&mut self) -> TrieNode {
            if self.empty_children as usize == self.child.len() {
                return TrieNode::NONE
            }
            if self.empty_children as usize == self.child.len() - 1 {
                for i in &mut self.child {
                    if i.is_some() {
                        return std::mem::take(i);
                    }
                }
            } else {
                while self.full_children > 0 && 50 * (self.full_children + self.child.len() as u32 - self.empty_children)
                    >= InternalNode::INFLATE_THRESHOLD * self.child.len() as u32 {
                    self.inflate();
                }

                while self.bits > 1 && 100 * (self.child.len() as u32 - self.empty_children) < InternalNode::HALVE_THRESHOLD * self.child.len() as u32 {
                    self.halve();
                }
            }

            if self.empty_children as usize==self.child.len()-1{
                for i in &mut self.child {
                    if i.is_some() {
                        return std::mem::take(i);
                    }
                }
            }
            TrieNode::NODE(std::mem::take(self))
        }


        fn refresh_key(&mut self){
            for i in &self.child{
                if i.is_some(){
                    self.key=i.key();
                }
            }
        }

        fn inflate(&mut self){
            let mut old_child = std::mem::take(&mut self.child);
            self.bits+=1;
            self.child = vec![TrieNode::NONE;1<<self.bits];
            self.full_children=0;
            self.empty_children=1<<self.bits;
            for (idx,node) in old_child.iter_mut().enumerate(){
                match node{
                    TrieNode::NODE(n) if n.pos>self.pos+self.bits-1 => {
                        if n.key.extract_bits(self.pos+self.bits-1,1)==0{
                            self.put_child(2*idx,node);
                        } else {
                            self.put_child(2*idx+1, node);
                        }
                    }
                    TrieNode::NODE(n)=>{
                        if n.bits==1{
                            self.put_child(2*idx, &mut n.child[0]);
                            self.put_child(2*idx+1, &mut n.child[1]);
                        } else {
                            let mut left=InternalNode::new(BitVector::empty(),n.pos+1,n.bits-1);
                            let mut right=InternalNode::new(BitVector::empty(),n.pos+1,n.bits-1);
                            let size = (1<<(n.bits-1)) as usize;
                            for idx in 0..size{
                                left.put_child(idx, &mut n.child[idx]);
                                right.put_child(idx, &mut n.child[idx + size]);
                            }
                            left.refresh_key();
                            right.refresh_key();
                            self.put_child(2*idx, &mut left.resize());
                            self.put_child(2*idx+1, &mut right.resize());
                        }
                    }
                    TrieNode::LEAF(n) => {
                        if n.key.extract_bits(self.pos+self.bits-1,1)==0{
                            self.put_child(2*idx,node);
                        } else {
                            self.put_child(2*idx+1, node);
                        }
                    }
                    TrieNode::NONE => {
                    }
                }
            }
        }

        fn halve(&mut self){
            let mut old_child = std::mem::take(&mut self.child);
            self.bits-=1;
            self.child = vec![TrieNode::NONE;1<<self.bits];
            self.full_children=0;
            self.empty_children=1<<self.bits;
            let old_length = old_child.len();
            for i in (0..old_length).step_by(2){
                let left=&old_child[i];
                let right=&old_child[i+1];
                match left{
                    TrieNode::NONE if right.is_none() => {continue;}
                    TrieNode::NONE if right.is_some() => {self.put_child(i/2, &mut old_child[i + 1])}
                    _=>{
                        match right {
                            TrieNode::NONE => {self.put_child(i/2, &mut old_child[i])}
                            // both child is nonempty
                            _=>{
                                let mut binary_node = InternalNode::new(left.key(),
                                                                        self.pos+self.bits,1);
                                binary_node.put_child(0, &mut old_child[i]);
                                binary_node.put_child(1, &mut old_child[i+1]);
                                self.put_child(i/2, &mut binary_node.resize());
                            }
                        }
                    }
                }
            }
        }

        fn full(&self, child: &TrieNode) -> bool {
            match child {
                TrieNode::NODE(v) => {
                    return v.pos == self.pos + self.bits;
                }
                TrieNode::LEAF(_) => {
                    return false
                }
                _ => { false }
            }
        }
    }

    #[derive(Debug)]
    struct Leaf {
        key: BitVector
    }

    #[derive(Debug)]
    enum TrieNode {
        NODE(InternalNode),
        LEAF(Leaf),
        NONE,
    }

    impl TrieNode {
        fn is_none(&self) -> bool {
            match self {
                TrieNode::NONE => { true }
                _ => { false }
            }
        }
        fn is_some(&self) -> bool {
            match self {
                TrieNode::NONE => { false }
                _ => { true }
            }
        }

        fn key(&self) -> BitVector{
            match self {
                TrieNode::NODE(n) => {n.key}
                TrieNode::LEAF(l) => {l.key}
                TrieNode::NONE => {BitVector::empty()}
            }
        }
    }

    impl Default for TrieNode {
        fn default() -> Self {
            TrieNode::NONE
        }
    }

    impl Clone for TrieNode{
        // Only TrieNode NONE is possible clone!
        fn clone(&self) -> Self {
            match self {
                TrieNode::NONE => {TrieNode::NONE}
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
                TrieNode::LEAF(l)=>{
                    TrieNode::LEAF(Leaf{
                        key:l.key
                    })
                }
            }
        }
    }

    impl TrieNode{
        fn lightweight_clone(&mut self)->Self{
            match self {
                TrieNode::NONE => {TrieNode::NONE}
                TrieNode::NODE(n) => {
                    TrieNode::NODE(InternalNode{
                        key: n.key,
                        pos: n.pos,
                        bits: n.bits,
                        full_children: n.full_children,
                        empty_children: n.empty_children,
                        child: std::mem::take(&mut n.child)
                    })
                }
                TrieNode::LEAF(l)=>{
                    TrieNode::LEAF(Leaf{
                        key:l.key
                    })
                }
            }

        }
    }

    pub struct LPCTrie{
        trie:TrieNode,
        size:u32,
        key_found:bool
    }

    impl LPCTrie{
        fn new()->LPCTrie{
            LPCTrie{
                trie: Default::default(),
                size: 0,
                key_found: false
            }
        }

        pub fn clear(&mut self){
            self.trie=TrieNode::NONE;
            self.size=0;
        }

        pub fn put(&mut self,key:BitVector){
            self.key_found = false;
            let mut trie = std::mem::take(&mut self.trie);
            let trie= self.insert_impl(key, &mut trie, 0);
            self.trie = trie;
            if !self.key_found{
                self.size+=1;
            }
        }

        pub fn get(&self, key:BitVector)->bool{
            let mut t:Option<&TrieNode> = Some(&self.trie);
            loop {
                if let Some(node)=t {
                    match node {
                        TrieNode::NODE(n) => {
                            t = Some(n.get_child(key.extract_bits(n.pos,n.bits)));
                        }
                        TrieNode::LEAF(l) => {
                            return if l.key == key {
                                true
                            } else {
                                false
                            }

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

        pub fn empty(&self)->bool{
            self.size==0
        }

        fn insert_impl(&mut self,key:BitVector,trie:& mut TrieNode,pos:u32)->TrieNode{
            return match trie {
                TrieNode::NODE(inode) if inode.key.sub_equal(pos, inode.pos - pos, &key) => {
                    let bitpat = key.extract_bits(inode.pos, inode.bits);
                    let insert_pos = inode.pos + inode.bits;
                    let mut n = self.insert_impl(key, &mut inode.get_mut_child(bitpat).lightweight_clone(),
                                                 insert_pos
                    );
                    inode.put_child(bitpat as usize, &mut n);
                    inode.resize()
                }
                TrieNode::LEAF(l) if key == l.key => {
                    self.key_found = true;
                    std::mem::take(trie)
                }
                TrieNode::NODE(_) | TrieNode::LEAF(_) => {
                    let new_pos = key.mismatch(pos, &trie.key());
                    let mut node = InternalNode::new(trie.key(), new_pos, 1);
                    let mut leaf = TrieNode::LEAF(Leaf { key });
                    if key.extract_bits(new_pos, 1) == 0 {
                        node.put_child(0, &mut leaf);
                        node.put_child(1, trie);
                    } else {
                        node.put_child(0, trie);
                        node.put_child(1, &mut leaf);
                    }
                    node.resize()
                }
                TrieNode::NONE => {
                    TrieNode::LEAF(Leaf { key })
                }
            }
        }
    }

    #[test]
    fn test_lpc_trie(){
        let mut trie=LPCTrie::new();
        let bitvecs:Vec<BitVector> = vec![
            "00010000".into(),
            "01000010".into(),
            "00001010".into(),
            "00101011".into(),
            "10101101".into(),
            "10110110".into(),
            "11011011".into(),
            "01101110".into(),
            "10111010".into(),
            "11101001".into(),
            "10100111".into(),
            "10011110".into()
        ];
        for bv in bitvecs{
            trie.put(bv);
        }
        assert_eq!(trie.get("00010000".into()),true);
        assert_eq!(trie.get("01000010".into()),true);
        assert_eq!(trie.get("00001010".into()),true);
        assert_eq!(trie.get("00101011".into()),true);
        assert_eq!(trie.get("10101101".into()),true);
        assert_eq!(trie.get("10110110".into()),true);
        assert_eq!(trie.get("11011011".into()),true);
        assert_eq!(trie.get("01101110".into()),true);
        assert_eq!(trie.get("10111010".into()),true);
        assert_eq!(trie.get("11101001".into()),true);
        assert_eq!(trie.get("10100111".into()),true);
        assert_eq!(trie.get("10011110".into()),true);
    }
}