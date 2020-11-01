use crate::geoip::{CIDR, GeoIP, GeoIPList};
use std::convert::TryInto;
use std::cmp::Ordering;

#[derive(Eq,PartialEq)]
pub struct V6{
    a:u64,
    b:u64
}

impl Ord for V6{
    fn cmp(&self, other: &Self) -> Ordering {
        if self.a<other.a||(self.a==other.a&&self.b<other.b){
            return Ordering::Less;
        } else if self==other {
            return Ordering::Equal;
        }
        Ordering::Greater
    }
}

impl PartialOrd for V6{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl V6{
    fn new(vec:&Vec<u8>)->V6{
        let inner = vec[0..8].try_into().unwrap();
        let inner2 = vec[0..8].try_into().unwrap();
        let mut v6=V6{
            a:u64::from_be_bytes(inner),
            b:u64::from_be_bytes(inner2)
        };
        v6
    }

    fn normalize_new(vec:&Vec<u8>,prefix:u8)->V6{
        let inner = vec[0..8].try_into().unwrap();
        let inner2 = vec[0..8].try_into().unwrap();
        let mut v6=V6{
            a:u64::from_be_bytes(inner),
            b:u64::from_be_bytes(inner2)
        };
        v6.normalize(prefix);
        v6
    }

    fn normalize(&mut self,prefix:u8){
        if prefix<=64{
            self.a=(self.a>>(64-prefix)<<(64-prefix));
            self.b=0;
        } else {
            self.b=(self.b>>(128-prefix)<<(128-prefix));
        }
    }
    fn normalize6(ip:&V6,prefix:u8)->V6{
        let mut a=0u64;
        let mut b=0u64;
        if prefix<=64{
            a=(ip.a>>(64-prefix)<<(64-prefix));
            b=0;
        } else {
            b=(ip.b>>(128-prefix)<<(128-prefix));
        }
        V6{
            a,b
        }
    }
}

pub struct GeoIPMatcher{
    country_code:String,
    ip4:Vec<u32>,
    prefix4:Vec<u8>,
    ip6:Vec<V6>,
    prefix6:Vec<u8>
}

impl GeoIPMatcher{
    fn match4(&self,ip:u32)->bool{
        if ip<self.ip4[0]{
            return false;
        }
        let mut r=self.ip4.len();
        let mut l:usize=0;
        while l<r{
            let x=((l+r)>>1);
            if ip<self.ip4[x]{
                r=x;
                continue;
            }
            let nip = ip>>(32-self.prefix4[x])<<(32-self.prefix4[x]);
            if nip==self.ip4[x]{
                return true;
            }
            l=x+1;
        }
        let nip = ip>>(32-self.prefix4[l-1])<<(32-self.prefix4[l-1]);
        return l>0&&nip==self.ip4[l-1];
    }

    fn match6(&self, ip:&V6) ->bool{
        if ip< &self.ip6[0] {
            return false;
        }
        let mut r=self.ip6.len();
        let mut l:usize=0;
        while l<r{
            let x=((l+r)>>1);
            if ip< &self.ip6[x] {
                r=x;
                continue;
            }
            let nip=V6::normalize6(ip,self.prefix6[x]);
            if nip==self.ip6[x]{
                return true;
            }
            l=x+1;
        }
        let nip = V6::normalize6(ip,self.prefix6[l-1]);
        return l>0 && nip==self.ip6[l-1];
    }

    pub fn put(&mut self, geoip: &mut GeoIP)
    {
        geoip.cidr.sort_by(|a,b|a.prefix.cmp(&b.prefix));
        self.country_code=geoip.country_code.to_uppercase().clone();
        for pair in geoip.cidr.iter() {
            let len = pair.ip.len();
            match len {
                16 => {
                    let prefix = pair.prefix as u8;
                    let v6=V6::normalize_new(&pair.ip, prefix);
                    self.ip6.push(v6);
                    self.prefix6.push(prefix);
                }
                4 => {
                    let inner = pair.ip.clone().try_into().unwrap();
                    let prefix = pair.prefix as u8;
                    let ip = u32::from_be_bytes(inner) >> (32 - pair.prefix) << (32 - pair.prefix);
                    self.ip4.push(ip);
                    self.prefix4.push(prefix);
                }
                _ => {
                    eprintln!("invalid ip length detected");
                }
            }
        }
    }

    pub fn new()->GeoIPMatcher{
        GeoIPMatcher{
            country_code: "".to_string(),
            ip4: vec![],
            prefix4: vec![],
            ip6: vec![],
            prefix6: vec![]
        }
    }

    pub fn match_ip(&self, v:&std::vec::Vec<u8>) ->bool{
        return match v.len() {
            4=>{
                let inner = v.clone().try_into().unwrap();
                self.match4(u32::from_be_bytes(inner))
            }
            16=>{
                let v6=V6::new(v);
                self.match6(&v6)
            }
            _=>{
                unimplemented!()
            }
        }
    }
}