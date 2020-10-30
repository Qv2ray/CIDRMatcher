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

const fn bit_size_of<T>()->usize{
    std::mem::size_of::<T>()*8
}

macro_rules! bit_vec_impl{
    ($T:ty) => {
        impl BitVec for $T{

            #[inline(always)]
            fn empty()->$T{
                0
            }

            // match sub bit vector start at offset and it's length is bits.
            #[inline]
            fn sub_equal(&self, offset:u32, mut bits:u32, other:&$T) -> bool{
                if bits==0 || offset as usize>=bit_size_of::<$T>(){
                    return true
                }
                bits = if bits as usize>bit_size_of::<$T>(){bit_size_of::<$T>() as u32}else{bits};
                ((other^self)<<offset>>(bit_size_of::<$T>()-bits as usize)) == 0
            }

            // extract a sub vector start at offset and it's length is bits.
            #[inline]
            fn extract_bits(&self,offset:u32, bits:u32)->$T{
                if (offset as usize)<bit_size_of::<$T>(){
                    return self<<offset>>(bit_size_of::<$T>()-bits as usize)
                }
                0
            }


            // find the left most significant bit position of mismatch sub vec start at offset.
            // start at 0..31 or 0..63
            #[inline]
            fn mismatch(&self,offset:u32,other:&$T)->u32{
                ((other^self)<<offset>>offset).msb()
            }

            #[inline(always)]
            fn safe_to_usize(&self) -> usize {
                *self as usize
            }

            #[inline]
            fn from_bit_str(value: &str) -> Self {
                let mut data:$T=0;
                let len=value.len();
                for (i,c) in value.chars().enumerate(){
                    if c=='1'{
                        println!("f");
                        data|=1<<(bit_size_of::<$T>()-1-i);
                    }
                }
                println!("g");
                data|=1<<(bit_size_of::<$T>()-1-len);
                data
            }

            #[inline(always)]
            fn is_empty(&self)->bool
            {
                *self == 0
            }
        }
    };
}

bit_vec_impl!(u32);
bit_vec_impl!(u64);
bit_vec_impl!(u128);

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

impl MSB for u128{
    fn msb(&self) -> u32 {
        let mut n=0;
        if ((0xffffffffffffffff0000000000000000>>n)&self)==0{n+=64}
        if ((0xffffffff000000000000000000000000>>n)&self)==0{n+=32}
        if ((0xffff0000000000000000000000000000>>n)&self)==0{n+=16}
        if ((0xff000000000000000000000000000000>>n)&self)==0{n+=8}
        if ((0xf0000000000000000000000000000000>>n)&self)==0{n+=4}
        if ((0xc0000000000000000000000000000000>>n)&self)==0{n+=2}
        if ((0x80000000000000000000000000000000>>n)&self)==0{n+=1}
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

