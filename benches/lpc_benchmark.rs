use cidr_matcher::cidr_bs::GeoIPMatcher;
use cidr_matcher::geoip;
use cidr_matcher::geoip::GeoIPList;
use cidr_matcher::lpc_trie::LPCTrie;
use criterion::{criterion_group, criterion_main, Criterion};
use radix_trie::Trie;
use std::convert::TryInto;
use std::fs::File;

pub fn read_file() -> geoip::GeoIPList {
    let file = "data/geoip.dat";
    let mut f = match File::open(&file) {
        Ok(f) => f,
        Err(e) => {
            panic!("open dat file {} failed: {}", file, e);
        }
    };
    let geo_ip_list: geoip::GeoIPList = match protobuf::Message::parse_from_reader(&mut f) {
        Ok(v) => v,
        Err(e) => {
            panic!("dat file {} has invalid format: {}", file, e);
        }
    };
    geo_ip_list
}

fn benchmark_lpc(b: &mut Criterion) {
    let mut geoip_list = read_file();
    let mut lpc_trie_cn_v6 = LPCTrie::<u128>::new();
    let mut lpc_trie_cn_v4 = LPCTrie::<u32>::new();
    for i in geoip_list.entry.iter_mut() {
        if i.country_code.to_uppercase() == "CN" {
            for pair in i.cidr.iter() {
                let len = pair.ip.len();
                match len {
                    16 => {
                        let inner = pair.ip.clone().try_into().unwrap();
                        let key = u128::from_be_bytes(inner) >> (128 - pair.prefix)
                            << (128 - pair.prefix);
                        lpc_trie_cn_v6.put(key, pair.prefix as u8, "CN".to_string());
                    }
                    4 => {
                        let inner = pair.ip.clone().try_into().unwrap();
                        let key =
                            u32::from_be_bytes(inner) >> (32 - pair.prefix) << (32 - pair.prefix);
                        lpc_trie_cn_v4.put(key, pair.prefix as u8, "CN".to_string());
                    }
                    _ => {
                        eprintln!("invalid ip length detected");
                    }
                }
            }
        }
    }
    b.bench_function("benchmark lpc", |b| {
        b.iter(|| benchmark_lpc_impl(&lpc_trie_cn_v6, &lpc_trie_cn_v4, &geoip_list))
    });
}

fn benchmark_radix(b: &mut Criterion) {
    let mut geoip_list = read_file();
    let mut radix_trie_v6 = Trie::<Vec<u8>, String>::new();
    let mut radix_trie_v4 = Trie::<u32, String>::new();
    for i in geoip_list.entry.iter_mut() {
        if i.country_code.to_uppercase() == "CN" {
            for pair in i.cidr.iter() {
                let len = pair.ip.len();
                match len {
                    16 => {
                        radix_trie_v6.insert(pair.ip.clone(), "CN".to_string());
                    }
                    4 => {
                        let inner = pair.ip.clone().try_into().unwrap();
                        let key =
                            u32::from_be_bytes(inner) >> (32 - pair.prefix) << (32 - pair.prefix);
                        radix_trie_v4.insert(key, "CN".to_string());
                    }
                    _ => {
                        eprintln!("invalid ip length detected");
                    }
                }
            }
        }
    }
    b.bench_function("benchmark radix trie", |b| {
        b.iter(|| benchmark_radix_trie_impl(&radix_trie_v4, &radix_trie_v6, &geoip_list))
    });
}

fn benchmark_v2ray_core_matcher(b: &mut Criterion) {
    let mut geoip_list = read_file();
    let mut matcher = GeoIPMatcher::new();
    for i in geoip_list.entry.iter_mut() {
        if i.country_code.to_uppercase() == "CN" {
            matcher.put(i);
        }
    }
    b.bench_function("benchmark bs-matcher", |b| {
        b.iter(|| benchmark_geoip_matcher_impl(&matcher, &geoip_list))
    });
}

fn benchmark_lpc_impl(
    lpc_trie_v6: &LPCTrie<u128>,
    lpc_trie_v4: &LPCTrie<u32>,
    geoip_list: &GeoIPList,
) {
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

fn benchmark_geoip_matcher_impl(matcher: &GeoIPMatcher, geoip_list: &GeoIPList) {
    for i in geoip_list.entry.iter() {
        for pair in i.cidr.iter() {
            matcher.match_ip(&pair.ip);
        }
    }
}

fn benchmark_radix_trie_impl(
    trie_v4: &Trie<u32, String>,
    trie_v6: &Trie<Vec<u8>, String>,
    geoip_list: &GeoIPList,
) {
    for i in geoip_list.entry.iter() {
        for pair in i.cidr.iter() {
            let len = pair.ip.len();
            match len {
                16 => {
                    trie_v6.get(&pair.ip);
                }
                4 => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    let bin_str =
                        u32::from_be_bytes(inner) >> (32 - pair.prefix) << (32 - pair.prefix);
                    trie_v4.get(&bin_str);
                }
                _ => {
                    eprintln!("invalid ip length detected");
                }
            }
        }
    }
}
criterion_group!(
    benches,
    benchmark_radix,
    benchmark_lpc,
    benchmark_v2ray_core_matcher
);
criterion_main!(benches);
