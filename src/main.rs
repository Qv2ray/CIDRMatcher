mod bit_vec;
mod geoip;
mod lpc_trie;
use crate::lpc_trie::LPCTrie;
use std::convert::TryInto;
use std::fs::File;
use std::net::IpAddr;

fn main() {
    let file = "src/geoip.dat";
    let mut f = match File::open(&file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("open dat file {} failed: {}", file, e);
            return;
        }
    };
    let site_group_list: geoip::GeoIPList =
        match protobuf::parse_from_reader::<geoip::GeoIPList>(&mut f) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("dat file {} has invalid format: {}", file, e);
                return;
            }
        };

    let mut lpc_trie_cn_us_v6 = LPCTrie::<u128>::new();
    let mut lpc_trie_cn_v6 = LPCTrie::<u128>::new();
    let mut lpc_trie_cn_v4 = LPCTrie::<u32>::new();
    for i in site_group_list.entry.iter() {
        if i.country_code.to_uppercase() == "CN" {
            for pair in i.cidr.iter() {
                let len = pair.ip.len();
                match len {
                    16 => {
                        let inner = pair.ip.clone().try_into().unwrap();
                        lpc_trie_cn_v6.put(
                            u128::from_be_bytes(inner) >> (128 - pair.prefix)
                                << (128 - pair.prefix),
                            pair.prefix as u8,
                            "CN".to_string(),
                        );
                        lpc_trie_cn_us_v6.put(
                            u128::from_be_bytes(inner) >> (128 - pair.prefix)
                                << (128 - pair.prefix),
                            pair.prefix as u8,
                            "CN".to_string(),
                        );
                    }
                    4 => {
                        let inner = pair.ip.clone().try_into().unwrap();
                        lpc_trie_cn_v4.put(
                            u32::from_be_bytes(inner) >> (32 - pair.prefix) << (32 - pair.prefix),
                            pair.prefix as u8,
                            "CN".to_string(),
                        );
                    }
                    _ => {
                        eprintln!("invalid ip length detected");
                    }
                }
            }
        }
    }
    let mut lpc_trie_us_v6 = LPCTrie::<u128>::new();
    for i in site_group_list.entry.iter() {
        for pair in i.cidr.iter() {
            let len = pair.ip.len();
            match len {
                16 if i.country_code.to_uppercase() != "CN" => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    if i.country_code.to_uppercase() == "US" {
                        lpc_trie_us_v6.put(
                            u128::from_be_bytes(inner) >> (128 - pair.prefix)
                                << (128 - pair.prefix),
                            pair.prefix as u8,
                            "US".to_string(),
                        );
                        lpc_trie_cn_us_v6.put(
                            u128::from_be_bytes(inner) >> (128 - pair.prefix)
                                << (128 - pair.prefix),
                            pair.prefix as u8,
                            "US".to_string(),
                        );
                    }
                    assert_eq!(lpc_trie_cn_v6.get(u128::from_be_bytes(inner)), false);
                }
                16 => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    assert_eq!(lpc_trie_cn_v6.get(u128::from_be_bytes(inner)), true);
                }
                4 if i.country_code.to_uppercase() != "CN" => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    assert_eq!(lpc_trie_cn_v4.get(u32::from_be_bytes(inner)), false);
                }
                4 => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    assert_eq!(lpc_trie_cn_v4.get(u32::from_be_bytes(inner)), true);
                }
                _ => {
                    eprintln!("invalid ip length detected");
                }
            }
        }
    }
    {
        if let IpAddr::V4(not_cn_ip) = "8.8.8.8".parse().unwrap() {
            assert_eq!(
                lpc_trie_cn_v4.get(u32::from_be_bytes(not_cn_ip.octets())),
                false
            );
        }
    }
    {
        if let IpAddr::V6(us_ip) = "2001:4860:4860::8888".parse().unwrap() {
            assert_eq!(
                lpc_trie_us_v6.get(u128::from_be_bytes(us_ip.octets())),
                true
            );
            assert_eq!(
                lpc_trie_cn_us_v6.get_with_value(u128::from_be_bytes(us_ip.octets())),
                "US"
            );
        }
    }
    lpc_trie_cn_us_v6.clear();
    assert_eq!(lpc_trie_cn_us_v6.empty(), true);
    println!("Hello, CIDRMatcher!");
}
