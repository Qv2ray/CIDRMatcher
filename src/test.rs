use crate::geoip;
use crate::lpc_trie::LPCTrie;
use deepsize::DeepSizeOf;
use std::convert::TryInto;
use std::fs::File;
use std::net::IpAddr;

#[cfg(test)]
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
#[test]
fn test_lpc() {
    let geoip_list = read_file();
    let mut lpc_trie_cn_us_v6 = LPCTrie::<u128>::new();
    let mut lpc_trie_cn_v6 = LPCTrie::<u128>::new();
    let mut lpc_trie_cn_v4 = LPCTrie::<u32>::new();
    for i in geoip_list.entry.iter() {
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
    let s1 = lpc_trie_cn_v4.deep_size_of() as f64 / (1024.0 * 1024.0);
    let s2 = lpc_trie_cn_v6.deep_size_of() as f64 / (1024.0 * 1024.0);
    println!(
        "lpc_trie_cn_v4 size:{}M, lpc_trie_cn_v6:{}M, total: {}M",
        s1,
        s2,
        s1 + s2
    );
    let mut lpc_trie_us_v6 = LPCTrie::<u128>::new();
    for i in geoip_list.entry.iter() {
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
            assert_eq!(
                lpc_trie_us_v6.remove(u128::from_be_bytes(us_ip.octets())),
                true
            );
            assert_eq!(
                lpc_trie_us_v6.get(u128::from_be_bytes(us_ip.octets())),
                false
            );
            assert_eq!(
                lpc_trie_us_v6.get_with_value(u128::from_be_bytes(us_ip.octets())),
                ""
            );
        }
    }
    lpc_trie_cn_us_v6.clear();
    assert_eq!(lpc_trie_cn_us_v6.empty(), true);
    println!("Hello, CIDRMatcher!");
}
