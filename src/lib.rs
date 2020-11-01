#![feature(test)]
#[cfg(test)]
pub mod benchmark;
pub mod bit_vec;
#[cfg(test)]
pub mod cidr_bs;
#[cfg(test)]
pub mod geoip;
pub mod lpc_trie;
#[cfg(test)]
mod test;
