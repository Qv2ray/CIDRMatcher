mod bit_vec;
mod geoip;
mod lpc_trie;
mod cidr_bs;

use crate::lpc_trie::LPCTrie;
use std::convert::TryInto;
use std::fs::File;
use std::net::IpAddr;
use deepsize::DeepSizeOf;
use crate::cidr_bs::GeoIPMatcher;
use crate::geoip::GeoIPList;
use radix_trie::{Trie, TrieCommon};
use std::time::SystemTime;

fn main() {
    let file = "src/geoip.dat";
    let mut f = match File::open(&file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("open dat file {} failed: {}", file, e);
            return;
        }
    };
    let mut site_group_list: geoip::GeoIPList =
        match protobuf::parse_from_reader::<geoip::GeoIPList>(&mut f) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("dat file {} has invalid format: {}", file, e);
                return;
            }
        };

    let mut geoip_matcher = GeoIPMatcher::new();
    let mut lpc_trie_cn_v6 = LPCTrie::<u128>::new();
    let mut lpc_trie_cn_v4 = LPCTrie::<u32>::new();
    let mut radix_trie_v6 = Trie::<Vec<u8>,String>::new();
    let mut radix_trie_v4 = Trie::<u32,String>::new();
    // insert all cn
    for i in site_group_list.entry.iter_mut() {
        if i.country_code.to_uppercase() == "CN" {
            geoip_matcher.put(i);
            for pair in i.cidr.iter() {
                let len = pair.ip.len();
                match len {
                    16 => {
                        let inner = pair.ip.clone().try_into().unwrap();
                        let key =u128::from_be_bytes(inner) >> (128 - pair.prefix)
                            << (128 - pair.prefix);
                        lpc_trie_cn_v6.put(
                            key,
                            pair.prefix as u8,
                            "CN".to_string(),
                        );
                        radix_trie_v6.insert(pair.ip.clone(),"CN".to_string());
                    }
                    4 => {
                        let inner = pair.ip.clone().try_into().unwrap();
                        let key=u32::from_be_bytes(inner) >> (32 - pair.prefix) << (32 - pair.prefix);
                        lpc_trie_cn_v4.put(
                            key,
                            pair.prefix as u8,
                            "CN".to_string(),
                        );
                        radix_trie_v4.insert(key,"CN".to_string());
                    }
                    _ => {
                        eprintln!("invalid ip length detected");
                    }
                }
            }
        }
    }

    let repeat_times= 5000;
    let now=SystemTime::now();
    for _ in 0..repeat_times {
        benchmark_lpc(&lpc_trie_cn_v6, &lpc_trie_cn_v4, &site_group_list);
    }
    println!("repeat times:{}, benchmark_lpc_matcher:{}", repeat_times,now.elapsed().unwrap().as_secs());
    let now=SystemTime::now();
    for _ in 0..repeat_times {
        benchmark_geoip_matcher(&geoip_matcher, &site_group_list);
    }
    println!("repeat times:{},benchmark_core_matcher:{}", repeat_times,now.elapsed().unwrap().as_secs());

    let now=SystemTime::now();
    for _ in 0..repeat_times {
        benchmark_radix_trie(&radix_trie_v4, &radix_trie_v6, &site_group_list);
    }
    println!("repeat times:{},benchmark_radix_trie:{}", repeat_times, now.elapsed().unwrap().as_secs());
}

fn benchmark_lpc(lpc_trie_v6:&LPCTrie<u128>,lpc_trie_v4:&LPCTrie<u32>,geoip_list:&GeoIPList){
    for i in geoip_list.entry.iter() {
        for pair in i.cidr.iter() {
            let len = pair.ip.len();
            match len {
                16 => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    lpc_trie_v6.get(u128::from_be_bytes(inner));
                }
                4 => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    lpc_trie_v4.get(u32::from_be_bytes(inner));
                }
                _ => {
                    eprintln!("invalid ip length detected");
                }
            }
        }
    }
}

fn benchmark_geoip_matcher(matcher:&GeoIPMatcher, geoip_list:&GeoIPList) {
    for i in geoip_list.entry.iter() {
        for pair in i.cidr.iter() {
            matcher.match_ip(&pair.ip);
        }
    }
}
fn benchmark_radix_trie(trie_v4:&Trie<u32,String>,trie_v6:&Trie<Vec<u8>,String>, geoip_list:&GeoIPList) {
    for i in geoip_list.entry.iter() {
        for pair in i.cidr.iter() {
            let len = pair.ip.len();
            match len {
                16 => {
                    trie_v6.get(&pair.ip);
                }
                4 => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    let bin_str=(u32::from_be_bytes(inner) >> (32- pair.prefix)
                        << (32- pair.prefix));
                    trie_v4.get(&bin_str);
                }
                _ => {
                    eprintln!("invalid ip length detected");
                }
            }
        }
    }
}
