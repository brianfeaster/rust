const BASE64TABLE :&[char] = &['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z','a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','0','1','2','3','4','5','6','7','8','9','+','/'];

pub fn encode (text: &[u8]) -> String {
    let mut out = String::new();
    let mut bits = 0;
    let mut idx :usize = 0;
    let mut text = text.iter();
    loop {
        let ch = text.next(); // Consider next byte
        if ch.is_none() && 0 == idx { return out; } // Exit on empty input and frame boundary.
        bits = bits<<8 | *ch.unwrap_or(&0) as usize; // Append next byte's bits or 0 if empty stream and mid frame
        out.push(BASE64TABLE[bits>>([2,4,6][idx]) & 63]); // Encode next 6 bits
        if ch.is_none() { break } // Finished encoding final partial frame
        if 2==idx { out.push(BASE64TABLE[bits & 63]); } // Completed full frame
        idx = (idx+1) % 3;
    }
    for _ in 0..3-idx { out.push('=') } // Pad partial frame
    out
}