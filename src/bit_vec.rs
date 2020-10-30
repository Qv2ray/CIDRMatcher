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
                <$T>::leading_zeros((other^self)<<offset>>offset)
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
                        data|=1<<(bit_size_of::<$T>()-1-i);
                    }
                }
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


