//#![feature(unchecked_math)]

pub trait MSB{
    fn msb(&self)->u32;
}

pub trait BitVec:Sized+Copy+Clone+Eq+PartialEq{
    fn empty()->Self;
    // match sub bit vector start at offset and it's length is bits.
    fn sub_equal(&self, offset:u32,bits:u32, other:&Self) -> bool;
    // extract a sub vector start at offset and it's length is bits.
    fn extract_bits(&self,offset:u32, bits:u32)->Self;
    // find the left most significant bit position of mismatch sub vec start at offset.
    // start at 0..31 or 0..63 or 0..128
    fn mismatch(&self,offset:u32,other:&Self)->u32;

    fn safe_to_usize(&self) -> usize;

    fn from_bit_str(_: &str)->Self;

    fn is_empty(&self)->bool;
}


impl BitVec for u32{
    fn empty()->u32{
        0
    }

    // match sub bit vector start at offset and it's length is bits.
    fn sub_equal(&self, offset:u32, mut bits:u32, other:&u32) -> bool{
        if bits==0 || offset>=32{
            return true
        }
        bits = if bits>32{32}else{bits};
        ((other^self)<<offset>>(32-bits)) == 0
    }

    // extract a sub vector start at offset and it's length is bits.
    fn extract_bits(&self,offset:u32, bits:u32)->u32{
        if offset<32{
            return self<<offset>>(32-bits)
        }
        0
    }


    // find the left most significant bit position of mismatch sub vec start at offset.
    // start at 0..31 or 0..63
    fn mismatch(&self,offset:u32,other:&u32)->u32{
        ((other^self)<<offset).msb()+offset
    }

    fn safe_to_usize(&self) -> usize {
        *self as usize
    }

    fn from_bit_str(value: &str) -> Self {
        let mut data:u32=0;
        let len:u32=value.len() as u32;
        for (i,c) in value.chars().enumerate(){
            if c=='1'{
                data|=1<<(31-i);
            }
        }
        data|=1<<(31-len);
        data
    }

    fn is_empty(&self)->bool
    {
        *self == 0
    }
}

impl MSB for u32{
    fn msb(&self) -> u32 {
        let mut n=0;
        if (0xffff0000&self)==0{
            n+=16;
        }
        if ((0xff000000>>n)&self)==0 {
            n+=8;
        }
        if ((0xf0000000>>n)&self)==0 {
            n+=4;
        }
        if ((0xC0000000>>n)&self)==0 {
            n+=2;
        }
        if ((0x80000000>>n)&self)==0 {
            n+=1;
        }
        n
    }
}


impl MSB for u64{
    fn msb(&self) -> u32 {
        let mut n=0;
        if ((0xffffffff00000000>>n)&self)==0{n+=32}
        if ((0xffff000000000000>>n)&self)==0{n+=16}
        if ((0xff00000000000000>>n)&self)==0{n+=8}
        if ((0xf000000000000000>>n)&self)==0{n+=4}
        if ((0xc000000000000000>>n)&self)==0{n+=2}
        if ((0x8000000000000000>>n)&self)==0{n+=1}
        n
    }
}


#[test]
fn test_bit_vec()
{
    let m:u64=1;
    assert_eq!(m.msb(),63);
}

//const fn msbidx(idx:usize)->u32{
//    const MSB_POSITION:[u32;32] =[31, 22, 30, 21, 18, 10, 29, 2,
//                                  20, 17, 15, 13, 9, 6, 28, 1,
//                                  23, 19, 11, 3, 16, 14, 7, 24,
//                                  12, 4, 8, 25, 5, 26, 27, 0];
//    MSB_POSITION[idx]
//}
//
//
//fn msb(mut v:u32)->u32{
//    v |= v >> 1; // first round down to one less than a power of 2
//    v |= v >> 2;
//    v |= v >> 4;
//    v |= v >> 8;
//    v |= v >> 16;
//    unsafe {
//        msbidx(((v.unchecked_mul(0x07C4ACDD) >> 27) as usize) )
//    }
//}

