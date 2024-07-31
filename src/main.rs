use std::{collections::HashMap, net::Ipv4Addr};

use etherparse::IpNumber;
use tcp::{Connection, State};
mod tcp;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Quad { 
    src : (Ipv4Addr, u16),
    dst : (Ipv4Addr, u16)
}
fn main() -> std::io::Result<()>{
    let mut nic = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun)?;
    let mut buffer = [0u8; 1504];
    let mut connections : HashMap<Quad, tcp::Connection> = Default::default();
    loop { 
        let n_bytes = nic.recv(&mut buffer)?;
        // let _eth_flags = u16::from_be_bytes([buffer[0], buffer[1]]);
        // let eth_proto = u16::from_be_bytes([buffer[2], buffer[3]]);
        // if eth_proto != 0x0800 {
        //     continue;
        // }
        match etherparse::Ipv4HeaderSlice::from_slice(&buffer[..n_bytes]) {
            Ok(iph) => {
                let src = iph.source_addr();
                let dst = iph.destination_addr();
                let proto = iph.protocol();
                if proto != IpNumber::TCP {
                    continue;
                }
                match etherparse::TcpHeaderSlice::from_slice(&buffer[iph.slice().len()..n_bytes]) {
                    Ok(tcph) => {
                        use std::collections::hash_map::Entry;
                        let datai = iph.slice().len() + tcph.slice().len();
                        let quad = Quad { 
                            src : (src, tcph.source_port()),
                            dst : (dst, tcph.destination_port())
                        };
                        match connections.
                        entry(quad) {
                            Entry::Occupied(mut c) => { 
                                c.get_mut().on_packet(&mut nic, iph, tcph, &buffer[datai..n_bytes]);
                            },
                            Entry::Vacant(mut e) => {
                                if let Some(conn) = tcp::Connection::accept(&mut nic, iph, tcph, &buffer[datai..n_bytes])? {
                                    e.insert(conn);
                                }
                            },
                        };
                        // println!("written {} bytes", written);
                        
                    },
                    Err(e) => eprintln!("not a tcp packet error : {:?}", e),
                }
            },
            Err(e) => eprintln!("weird packet arrived, errord :{}", e),
        }
        
    }
    
    
}
