use ::std::io::{self, prelude::*}; // OR use std::io::{Write, Read}
use ::std::net::{TcpListener, TcpStream};
use ::std::error::{Error};
use ::std::{fmt, thread};
use ::sha1::*;
use ::log::*;
use ::utils::*;
use openssl::ssl::{SslMethod, SslAcceptor, SslFiletype};
use std::sync::Arc;


const HYPHENID :&str = "HYPHEN";
const HYPHENVER :[u8;3] = [0, 0, 0];

fn word<S : Read > (stream :&mut S) -> Option<String> {
    let mut s = String::new();
    let mut c :[u8;1] = [0];
    loop {
        if 0 == stream.read(&mut c).unwrap() { break; }; // TODO retry?
        if c[0] == b' ' || c[0] == b'\n' { break; }
        if c[0] == b'\r' { continue; } // ignore returns
        s.push(c[0] as char);
    }
    //info!("  word>>> {:?}", s);
    if 0 == s.len() { None } else { Some(s) }
}

fn read<S :Read> (
    tcp :&mut S,
    buff: &mut [u8]
) -> io::Result<usize> {
    let buff_len = buff.len();
    let count = tcp.read(buff)?;
    Ok({
        if buff_len == count { // Buffer successfully filled
            debug!(">>> {:?}", bytes_pretty(&buff));
            count
        } else if 0 == count { // Give up if no bytes read
            return Err(io::Error::new(io::ErrorKind::Other, "tcp.read returned length 0"))
        } else { // Try to fill rest of buff
            count + read(tcp, &mut buff[count..])?
        }
    })
}

fn write<S : Write > (
    tcp: &mut S,
    buff:&[u8]
) -> io::Result<usize> {
    debug!("<<< {:?}", bytes_pretty(&buff));
    let buff_len = buff.len();
    let count = tcp.write(buff)?;
    Ok({
        if buff_len == count { // Entire buffer successfully wrote
            count
        } else if 0 == count { // Give up if no bytes sent at all
            return Err(io::Error::new(io::ErrorKind::Other, "tcp.write returned length 0"))
        } else { // Try to fill rest of buff
            count + write(tcp, &buff[count..])?
        }
    })
}

// S TcpStream SSLStream?
fn negotiate_connection<S : Read + Write> (tcp: &mut S) -> io::Result<&'static str> {
    // Parse HTTP request fields
    let httprequest = word(tcp).unwrap();
    let httppath = word(tcp).unwrap();
    let httpver = word(tcp).unwrap();
    // Read all HTTP headers and capture key
    let mut key :String = String::new();
    while let Some(s) = word(tcp) {
        if s == "Sec-WebSocket-Key:" {
            key += &word(tcp).unwrap();
        }
    }

    info!(" > negotiate_connection \"{} {} {} {}\"", httprequest, httppath, httpver, key);

    // Create upgrade token
    let mut sha1 = Sha1::new();
    key += "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    sha1.update(key.as_ref());
    let hash = base64::encode(&sha1.digest().bytes());

    // Send back upgrade response/request
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\
         \r\n", hash.as_str());

    info!(" < negotiate_connection {:?}", response);
    write(tcp, response.as_bytes())?;

    info!(" . negotiate_connection done");
    Ok("")
} // fn negotiate_connection 


fn bytes_pretty (bytes :&[u8]) -> String {
    let mut s = String::new();
    for b in bytes {
        s.push(*b as char)
    }
    s
}

fn typetostr (t: u8) -> &'static str {
    match t {
        0 => "CONTINUATION",
        1 => "TEXT",
        2 => "BINARY",
        8 => "CLOSE",
        9 => "PING",
       10 => "PONG",
        _ => "UNKNOWN"
    }
}

fn payload<S :Write> (tcp: &mut S, t:u8, msg: &[u8]) -> io::Result<usize> {
    info!("< {} payload {:?}", typetostr(t), bytes_pretty(&msg));
    let mut b = Vec::<u8>::with_capacity(2 + msg.len());
    b.push(0x80 + t); // TODO handle writing framents 0x80 implies single/final fragment
    let len = msg.len();
    if len <= 125 {
        b.push(msg.len() as u8);
    } else if len <= 65535 {
        b.push(126);
        b.push(((len >>  8)       ) as u8);
        b.push(((len      ) & 0xff) as u8);
    } else {
        b.push(127);
        b.push(((len >> 56)       ) as u8);
        b.push(((len >> 48) & 0xff) as u8);
        b.push(((len >> 40) & 0xff) as u8);
        b.push(((len >> 32) & 0xff) as u8);
        b.push(((len >> 24) & 0xff) as u8);
        b.push(((len >> 16) & 0xff) as u8);
        b.push(((len >>  8) & 0xff) as u8);
        b.push(((len      ) & 0xff) as u8);
    }
    b.extend(msg);
    write(tcp, &b)
}

fn text<S :Write>   (tcp: &mut S, msg: &[u8]) -> io::Result<usize> { payload(tcp, 0x1, msg) }
fn binary<S :Write> (tcp: &mut S, msg: &[u8]) -> io::Result<usize> { payload(tcp, 0x2, msg) }
fn close<S :Write>  (tcp: &mut S, msg: &[u8]) -> io::Result<usize> { payload(tcp, 0x8, msg) }
fn ping<S :Write>   (tcp: &mut S, msg: &[u8]) -> io::Result<usize> { payload(tcp, 0x9, msg) }
fn pong<S :Write>   (tcp: &mut S, msg: &[u8]) -> io::Result<usize> { payload(tcp, 0xa, msg) }

enum Frame {
    Text(Vec<u8>),
    Binary(Vec<u8>),
    Close(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>)
}

fn frame<S: Read> (tcp: &mut S) -> io::Result<Frame> {
    // Frame type/operation
    let mut op = [0;2];
    read(tcp, &mut op)?;

    // Payload len is either 7 bits, 16 bits or 64 bits
    let len :usize = match op[1]&0x7f {
        126 => {
            let mut b = [0;2];
            read(tcp, &mut b)?;
            ((b[0] as usize) << 8) + b[1] as usize
        },
        127 => {
            let mut b = [0;8];
            read(tcp, &mut b)?;
            ((b[0] as usize)<<56) + ((b[1] as usize)<<48) +
            ((b[2] as usize)<<40) + ((b[3] as usize)<<32) +
            ((b[4] as usize)<<24) + ((b[5] as usize)<<16) +
            ((b[6] as usize)<<8)  + (b[7] as usize)
        },
        len => len as usize
    };

    info!(">frame OP {:02x?}  LEN {}", op, len);

    // Mask key bytes
    let key :Option<[u8;4]>= match op[1]&0x80 {
        0 => None,
        _ => {
            let mut m = [0;4];
            read(tcp, &mut m)?;
            info!(">frame MASK KEY {:02x?}", m);
            Some(m)
        }
    };

    let mut payload = Vec::<u8>::with_capacity(len);
    let slice :&mut [u8]= unsafe { std::slice::from_raw_parts_mut(payload.as_mut_ptr(), len) };
    read(tcp, slice)?;
    if let Some(mask_bytes) = key {
        for i in 0..len { payload.push(slice[i] ^ mask_bytes[i%4]) }
    }
    info!(">frame PAYLOAD {:02x?}", slice);

    match op[0] &0x0f {
        1 => Ok(Frame::Text(payload)),
        2 => Ok(Frame::Binary(payload)),
        8 => Ok(Frame::Close(payload)),
        9 => Ok(Frame::Ping(payload)),
       10 => Ok(Frame::Pong(payload)),
        o => Err(io::Error::new(io::ErrorKind::Other, format!("Unknown Frame Op Type {}", o)))
    }
}

// S : TcpStream SslStream
fn fun_websocket_client<S : Read + Write> (tcp: &mut S) -> io::Result<&'static str> {
    negotiate_connection(tcp)?;

    text(tcp, HYPHENID.as_bytes())?;
    binary(tcp, &HYPHENVER)?;
    if !true { ping(tcp, "BALZ".as_bytes())?; }

    loop {
        match frame(tcp)? {
            Frame::Text(buff) => {
                info!("< TEXT {:?}", std::str::from_utf8(&buff).map_err(db));
                if buff[0] == b'Q' { close(tcp, "bye".as_bytes())?; }
                if 3 < buff.len() {
                    if &buff[0..=3] == "get1".as_bytes() { text(tcp, "J4i3hwWl00LaPMnXUG026AK01snlTt1XStC10li000LaPMnXUG02D7q11bDvRdHeCGB_014g0046KtbkT6Wo0ly006G20GPJUMvqQ3C2_m8060C11bDvRdHeD0F_0kx_0082K3O410GB1bDvRdHeCMO0HKL5RKL5HMq51GKA2We820WC20mC30K51GeA2WW820S730SC1bDvRdHeCly01GK5QGK51Ma51GeA20WC30K71GKA2WW81mm730PJUMvqQ3Fa0000G7q5041V7IGb92GW91qO92aiAoaa7H8uC3WkSsvXScKkP79rRIvoONTc000100400G01WGyuC3WkRM5oOMDXSovoONTc04G004H40014WmiuC3WkScbjBd9XTsO00G0V40007n253ZWmE2vZR65sPNCkSc5tPW00H00004G008SCE30uBcDiON0kSc5tPW0000100000G8WME30uBcXfPsXeONGkRt1bRcLaBd9XTo000000LG00K5M93ZWmE2vZUMrYOMmkSc5tNm010000000008eDEJ0vBdHeTMrmBd9XT_C04H4H4H4H4H6B0W9GCGG410m6KtbkT6WnPW15HKLjHKL5RGK51GeA2WW820m830mC1GK52WeA20W830WC2Wm6KtbkT6Wo_m051GLf1GK5QGK52We820mC1GS51GeA20WC20S31bDvRdHeC-8000004TbrGFCiAoaT92Ka81yT614fC2WiAIGT1bDvRdHeD1W000000051G14mBYaaB30IE30uBdDkON9bBcHoTMqkSc5tPW000G0100400O4FE30uBcrXSc5ZONCkSc5tPW140014H000H8CBE30uBd9fRIvoONTc00407n0001yGXGuuC3WkOsnXTcLpBd9XTsO004G00014002733WmE2vZR65mBd9XTsO00000G00004285ZWmE2veQMTeQ65qBczmPMvbP2voONSW000005K0051LYGuuC3WkOtbjOc5iBd9XTry00G000000002A3JamEIvqQ7LjS2voONVp014H4H4H4H4HYm82K38410GC1bDvRdHeCMO0HKL5RKL5HMq51GKA2We820WC20mC30K51GeA2WW820m830eC1bDvRdHeCe801GK5QGK51Ma51GeA20WC30K71GKA2WW830mA30PJUMvqQ3Fa00K51n4NSLbv81qY7I0V7IGbAIGT61aM7HyW82GbAHWT6146KtbkT6Wq4m0100410G410J0rD3KmB2aIE30uBdDkON9bBcHoTMqkSc5tPW000G0100400O4FE30uBcrXSc5ZONCkSc5tPW140014H000H8CBE30uBd9fRIvoONTc00407n0001yGXGuuC3WkOsnXTcLpBd9XTsO004G00014002733WmE2vZR65mBd9XTsO00000G00004285ZWmE2veQMTeQ65qBczmPMvbP2voONSW000005K0051LYGuuC3WkOtbjOc5iBd9XTry00G000000002A3JamEIvqQ7LjS2voONVp014H4H4H4H4HYm82K3G410GB1bDvRdHeCkG01GK5QGK51Ma51GeA20WC30K71GKA2WW830W70mPJUMvqQ3Fa00K51n4TLq4L81qY7I0V7IGb92GeAIGW7nWf7I0W7ny6KtbkT6Wq600000000G010JKiAH8uC3WkSsvXScKkP79rRIvoONTc000100400G01WGyuC3WkRM5oOMDXSovoONTc04G004H40014WmiuC3WkScbjBd9XTsO00G0V40007n253ZWmE2vZR65sPNCkSc5tPW00H00004G008SCE30uBcDiON0kSc5tPW0000100000G8WME30uBcXfPsXeONGkRt1bRcLaBd9XTo000000LG00K5M93ZWmE2vZUMrYOMmkSc5tNm010000000008eDEJ0vBdHeTMrmBd9XT_C04H4H4H4H4H6B0W9GDGG410i6KtbkT6Wov0051GLf1GK5QGK52We820mC1GS51GeA20WC20S31bDvRdHeC-G01GK74HTPHtaW7I8T81yT92Kf91qO6HWW9I0V7I8W61qT60PJUMvqQ3GO00400G410G41C3KqDJ0a818uC3WkSsvXScKkP79rRIvoONTc000100400GG1WGyuC3WkRM5oOMDXSovoONTc04G004H40014WmiuC3WkScbjBd9XTsO00G0V40007n253ZWmE2vZR65sPNCkSc5tPW00H00004G008SCE30uBcDiON0kSc5tPW0000100000G8WME30uBcXfPsXeONGkRt1bRcLaBd9XTo000000LG00K5M93ZWmE2vZUMrYOMmkSc5tNm010000000008eDEJ0vBdHeTMrmBd9XT_C04H4H4H4H4H6B0m9GCGC2K3830b0o".as_bytes())?; }
                    if &buff[0..=3] == "get2".as_bytes() { text(tcp, "J4i3cT0v00LaPMnXUG020dy01snlTt1XStC10kL_0GPJUMvqQ342_nm2_W011bDvRdHeCWB_008P0046KtbkT6Wp0ly00ZG00GPJUMvqQ3G1_m02MW020b0q10G41mPJUMvqQ35V00400G010040000300PJUMvqQ39Y05LLLLLLLLLL30mC30mC30mC30mC30mC30yF3myE3WuA30mC30mC30mIE30uBdDkON9bBcHoTMqkSc5tPW0G410G410G484FE30uBcrXSc5ZONCkSc5tCG3_4VyH_n7_0OCME30uBcXfPsXeONGkOsnlSsLaBd9XTsO0410G410G41263JamEIvqQ7LjS2voONTb014H4H4H4H4HYnCvC3akSsvXScKkRczfSsKkSc5tD00G410G410G48m20b0r10G420PJUMvqQ35c00400G010040000300PJUMvqQ39b05LLLLLLLLLL30mC30mC30mC30mC30mC30yF3myE3WuA30mC30mC30m6KtbkT6WpPG1LLLLLLLLLLHWR61iO6nWR61iO6nWR61iO7nqR6XOO6nWR61iO6nWR4ZWmE2vpRc5oPIvaSdLjBd9XTsO0410G410G41213pWmE2vjON9XOs5pBd9XTp40_n7_4VyH_m635ZWmE2veQMTeQ65qBcDiRtDbP2voONTc010G410G410GXWqvC3akT6XrRN0kSc5tPG0H4H4H4H4H4OiJEJ0vBdDkON9bBcvlQNDbBd9XTpG0410G410G412C0W9GCGG410a6KtbkT6WnPm0100400G0100000m06KtbkT6WoOW1LLLLLLLLLLGmC30mC30mC30mC30mC30mF3myF3WuE2WmC30mC30mC1bDvRdHeCsS0LLLLLLLLLLKO6nWR61iO6nWR61iO6nWR61yT6neM61iO6nWR61iO6mPJUMvqQ3HC015L4H4HLH4H92Sa7nqR91Wa7oaaAoSc8YGV6oGIE30uBdDkON9bBcHoTMqkSc5tPW0G410G410G484FE30uBcrXSc5ZONCkSc5tCG3_4VyH_n7_0OCME30uBcXfPsXeONGkOsnlSsLaBd9XTsO0410G410G41263JamEIvqQ7LjS2voONTb014H4H4H4H4HYnCvC3akSsvXScKkRczfSsKkSc5tD00G410G410G48m20b0s10G420PJUMvqQ35c00400G010040000300PJUMvqQ39b05LLLLLLLLLL30mC30mC30mC30mC30mC30yF3myE3WuA30mC30mC30m6KtbkT6WpPW1LLLLLLLLLLHWR61iO6nWR61iO6nWR61iO7nqR6XOO6nWR61iO6nWR4ZWmE2vpRc5oPIvaSdLjBd9XTsO0410G410G41213pWmE2vjON9XOs5pBd9XTp40_n7_4VyH_m635ZWmE2veQMTeQ65qBcDiRtDbP2voONTc010G410G410GXWqvC3akT6XrRN0kSc5tPG0H4H4H4H4H4OiJEJ0vBdDkON9bBcvlQNDbBd9XTpG0410G410G412C0W9GCmG410a6KtbkT6WnPW0100400G0100000m06KtbkT6WoNm1LLLLLLLLLLGmC30mC30mC30mC30mC30mF3myF3WuE2WmC30mC30mC1bDvRdHeCsS0LLLLLLLLLLKO6nWR61iO6nWR61iO6nWR61yT6neM61iO6nWR61iO6mPJUMvqQ3HC05LH4HLH4LLL92Oa7oGa8XyT7HyaAIad92Oa7oGR91iV918uC3WkSsvXScKkP79rRIvoONTc010G410G410GWGyuC3WkRM5oOMDXSovoONSn0FyH_n7_4Vy1WnOuC3WkQ6bdQ6XXT2vZR6zpPMGkSc5tPW0G410G410G48ODEJ0vBdHeTMrmBd9XTsK04H4H4H4H4H6B4pamEIvpRc5oPIvkRsbpPIvoONSq010G410G410GZ0C2K3430b0s0W9GDmG410a6KtbkT6WnPW0100400G0100000m06KtbkT6WoNm1LLLLLLLLLLGmC30mC30mC30mC30mC30mF3myF3WuE2WmC30mC30mC1bDvRdHeCsS0LLLLLLLLLLKO6nWR61iO6nWR61iO6nWR61yT6neM61iO6nWR61iO6n8uC3WkSsvXScKkP79rRIvoONTc010G410G410GWGyuC3WkRM5oOMDXSovoONSn0FyH_n7_4Vy1WnOuC3WkQ6bdQ6XXT2vZR6zpPMGkSc5tPW0G410G410G48ODEJ0vBdHeTMrmBd9XTsK04H4H4H4H4H6B4pamEIvpRc5oPIvkRsbpPIvoONSq010G410G410GZ0PJUMvqQ3HC014H4H4H4H4H61CF31WJ61iV91yR7HiQ6082K3W410GA1bDvRdHeCKm00G0100400G0000C01bDvRdHeCby0LLLLLLLLLLKC30mC30mC30mC30mC30mC3myF3muE3WeC30mC30mC30PJUMvqQ3Dd05LLLLLLLLLL61iO6nWR61iO6nWR61iO6nWV7HiQ5XWR61iO6nWR61i6KtbkT6WqJ01LKH4LKH5LLIGc91ya928V7HqV92af9oGc91ya6oGR7oGIE30uBdDkON9bBcHoTMqkSc5tPW0G410G410G484FE30uBcrXSc5ZONCkSc5tCG3_4VyH_n7_0OCME30uBcXfPsXeONGkOsnlSsLaBd9XTsO0410G410G41263JamEIvqQ7LjS2voONTb014H4H4H4H4HYnCvC3akSsvXScKkRczfSsKkSc5tD00G410G410G48m6KtbkT6WqIG0H4H4H4H4H4HWJ3mmO4nWR7oGV6nqR6XW20b0v10G42GPJUMvqQ35c00400G010040000300PJUMvqQ39V05LLLLLLLLLL30mC30mC30mC30mC30mC30yF3myE3WuA30mC30mC30m6KtbkT6WpPm1LLLLLLLLLLHWR61iO6nWR61iO6nWR61iO7nqR6XOO6nWR61iO6nWR4ZWmE2vpRc5oPIvaSdLjBd9XTsO0410G410G41213pWmE2vjON9XOs5pBd9XTp40_n7_4VyH_m635ZWmE2veQMTeQ65qBcDiRtDbP2voONTc010G410G410GXWqvC3akT6XrRN0kSc5tPG0H4H4H4H4H4OiJEJ0vBdDkON9bBcvlQNDbBd9XTpG0410G410G412C1bDvRdHeD4m04H4H4L5L4H4mAoGV6nWM4mmO6nyY92Sh91yO0WDGCJ0410GA1bDvRdHeCMO00G0100400G0000C01bDvRdHeCby0LLLLLLLLLLKC30mC30mC30mC30mC30mC3myF3muE3WeC30mC30mC30PJUMvqQ3Db05LLLLLLLLLL61iO6nWR61iO6nWR61iO6nWV7HiQ5XWR61iO6nWR61i6KtbkT6WqJm1LKH4LKH5LLIGc91ya928V7HqV92af9oGc91ya6oGR7oGIE30uBdDkON9bBcHoTMqkSc5tPW0G410G410G484FE30uBcrXSc5ZONCkSc5tCG3_4VyH_n7_0OCME30uBcXfPsXeONGkOsnlSsLaBd9XTsO0410G410G41263JamEIvqQ7LjS2voONTb014H4H4H4H4HYnCvC3akSsvXScKkRczfSsKkSc5tD00G410G410G48m6KtbkT6WqJm0H4H4HKLKH4J0h91yR61OJ31WR7o8a9oia7nW30b0u".as_bytes())?; }
                }
            },
            Frame::Binary(buff) => info!("< BINARY {:?}", bytes_pretty(&buff)),
            Frame::Close(buff) => {
                info!("< CLOSE {:?}", bytes_pretty(&buff));
                break Ok("Client closed connection.");
            },
            Frame::Ping(buff) => {
                info!("< PING {:?}", bytes_pretty(&buff));
                pong(tcp, &buff)?;
            },
            Frame::Pong(buff) => info!("< PONG {:?}", bytes_pretty(&buff))
        }
    }

} // fn fun_websocket_client 


// An io::Result<T> (which is a std::Result<T, io::Error>
//  will be destructured and the Err component passed here
//
// .map_err will get the Result, destructure the Err, pass error to Fn which return error, then is wrapped in Err
// .or_else will get the Result, destructure the Err, pass error to Fn which return error in a Result
//
fn db    <O: fmt::Debug>    (o: O) -> O            { error!("!!! {:?}", o); o }      // .map_err(db)
fn _dberr <O: fmt::Debug, R> (o: O) -> Result<R, O> { error!("!e! {:?}", o); Err(o) } // .or_else(dberr)

fn _info  <O: fmt::Debug> (o: O) -> O { info!("{:?}", o); o }
fn _error <O: fmt::Debug> (o: O) -> O { error!("{:?}", o); o }

fn dbresult <O: fmt::Debug, E: fmt::Debug> (r: Result<O,E>) -> Result<O,E> {
    match r {
        Ok(ref o) => info!("{:?}", o),
        Err(ref e) => error!("{:?}", e),
    }
    r
}

pub fn server () -> Result<&'static str, Box<dyn Error>> {
    ::std::println!("== {}:{} ::{}::server() ====", std::file!(), core::line!(), core::module_path!());
    info!("Server Start - {}v{}.{}.{}", HYPHENID, HYPHENVER[0], HYPHENVER[1], HYPHENVER[2]);
    let listener :TcpListener = TcpListener::bind("127.0.0.1:7180")?;
    for stream in listener.incoming() { // This blocks
        let tcp :TcpStream = stream?;
        info!("Client thread starting with {:?}", tcp);
        thread::spawn( move || {
            let mut tcp = tcp;
            dbresult(fun_websocket_client(&mut tcp)).unwrap();
            let ret = tcp.shutdown(std::net::Shutdown::Both);
            info!("TCP socket shutdon returned {:?}", ret);
        });
    }
    Err(io::Error::new(io::ErrorKind::Other, "Listener loop exited.").into())
}

pub fn server_ssl () -> Result<&'static str, Box<dyn Error>> {
    ::std::println!("== {}:{} ::{}::server_ssl() ====", std::file!(), core::line!(), core::module_path!());
    info!("Server SSL Start - {}v{}.{}.{}", HYPHENID, HYPHENVER[0], HYPHENVER[1], HYPHENVER[2]);

    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor.set_private_key_file("key.pem", SslFiletype::PEM)?;
    acceptor.set_certificate_chain_file("certificate.pem")?;
    acceptor.check_private_key()?;
    let acceptor = Arc::new(acceptor.build());

    let listener = TcpListener::bind("0.0.0.0:7179").unwrap();

    for stream in listener.incoming() {
        let tcp :TcpStream = stream?;
        info!("SSL client thread starting with {:?}", tcp);
        let acceptor = acceptor.clone();
        thread::spawn(move || {
            let tcp = tcp;
            let mut stream = acceptor.accept(tcp).unwrap();
            info!("SSL stream thread starting with {:?}", stream);
            dbresult(fun_websocket_client(&mut stream)).unwrap();
            info!("SSL shutdon returned {:?}", stream.shutdown());
        });
    }
    Err(io::Error::new(io::ErrorKind::Other, "SSL Listener loop exited.").into())
}

pub fn main () {
    ::std::println!("== {}:{} ::{}::main() ====", std::file!(), core::line!(), core::module_path!());
    if true { ::pretty_env_logger::try_init().map_err(db).unwrap(); }
    // http server in a new thread
    thread::spawn(|| { dbresult(crate::server()).unwrap(); } );
    // https server in this thread
    dbresult(crate::server_ssl()).unwrap();
    /*
    Err::<(), io::Error>
       (io::Error::new(io::ErrorKind::Other, "wat"))
       .map_err(db) //  -> Err(db())  FnOnce(E:Result<>) -> F
       .or_else(dberr); // -> db()       FnOnce(E:Result<>) -> Result<T, F>*/
}
