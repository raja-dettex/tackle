use std::io::Error;

use etherparse::{Ipv4Header, Ipv4HeaderSlice, TcpHeader, TcpHeaderSlice};
pub enum State { 
    SynRcvd,
    Estab
}

pub struct Connection { 
    state: State,
    snd: SndSequenceSpace,
    rcvd: RcvdSequeneceSpace,
    ip: Ipv4Header
}

struct SndSequenceSpace { 
    ///SND.UNA - send unacknowledged
    una: u32,
    ///  SND.NXT - send next
    nxt: u32,
    ///  SND.WND - send window
    wnd: u16,
    ///  SND.UP  - send urgent pointer
    up: bool,
    ///  SND.WL1 - segment sequence number used for last window update
    wl1: usize,
    ///  SND.WL2 - segment acknowledgment number used for last window update
    wl2 : usize,
    ///  ISS     - initial send sequence number
    iss: u32
}
struct RcvdSequeneceSpace { 
    /// RCV.NXT - receive next
    nxt: u32,
    /// RCV.WND - receive window
    wnd: u16,
    /// RCV.UP  - receive urgent pointer
    up: bool,
    /// IRS     - initial receive sequence number
    irs: u32
}

// impl Default for Connection { 
//     fn default() -> Self {
//         Self { state: State::Listen}
//     }
// }


impl Connection { 
    pub fn on_packet<'a>(&mut self, nic : &mut tun_tap::Iface, iph: Ipv4HeaderSlice<'a>, tcph: TcpHeaderSlice<'a>, payload : &'a [u8] ) 
    -> std::io::Result<()>{
        // acceptable ack check as of RFC : 793
        // send.una < seq.ack <= send.next
        let ackn = tcph.acknowledgment_number();
        // wrap and overflow check 
        if self.snd.una < ackn {
            // if next is between ack and una , case is violated
            if self.snd.nxt > self.snd.una && self.snd.nxt < ackn {
                return Ok(());
            }
        }
        match self.state {
            State::SynRcvd => todo!(),
            State::Estab => todo!(),
        }                
    }
    pub fn accept<'a>(
        nic: &mut tun_tap::Iface, 
        iph: Ipv4HeaderSlice<'a>, 
        tcph: TcpHeaderSlice<'a>,
        _payload: &'a [u8]
    ) -> std::io::Result<Option<Connection>> {
        if !tcph.syn() || tcph.ack() {
            return Ok(None);
            // return Err(Error::from("not syn"));
        }
        let iss = 0;
        let mut ip = etherparse::Ipv4Header::new(
            0, 
            64, 
            etherparse::IpNumber::TCP, 
            iph.destination_addr().octets(), 
            iph.source_addr().octets()).unwrap();
        let mut conn = Connection { 
            state: State::SynRcvd,
            snd: SndSequenceSpace {
                iss : iss,
                una : iss,
                nxt : iss + 1,
                wnd : 10, 
                up : false,
                wl1: 0,
                wl2: 0     
            }, 
            rcvd : RcvdSequeneceSpace{
                irs : tcph.sequence_number(),
                nxt : tcph.sequence_number() + 1,
                wnd : tcph.window_size(),
                up : false
            },
            ip: ip
        };
        let mut syn_ack = etherparse::TcpHeader::new(
            tcph.destination_port(), 
            tcph.source_port(), 
            conn.snd.iss, 
            conn.snd.wnd);
        let mut buff = [0u8; 1500];
        syn_ack.acknowledgment_number = conn.rcvd.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;
        let _ = conn.ip.set_payload_len(syn_ack.header_len() as usize + 0).unwrap();
        syn_ack.checksum =  syn_ack.calc_checksum_ipv4(&conn.ip, &[0]).expect("checksum");
        let unwritten = { 
            let mut unwritten = &mut buff[..];
            let _ = conn.ip.write(&mut unwritten);
            let _= syn_ack.write(&mut unwritten);
            unwritten.len()
        };
        let bytes_written = buff.len() - unwritten;
        nic.send(&buff[..bytes_written])?;
        Ok(Some(conn))
    }
}