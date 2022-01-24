/// PlantUML has its own base64 dialect, this struct provides the implementation for that
pub struct Base64PlantUML {}

impl Base64PlantUML {
    pub fn encode(data: &[u8]) -> String {
        let mut encoded = String::from("");

        let len = data.len();
        for i in (0..len).step_by(3) {
            if i + 2 == len {
                encoded.push_str(&encode3bytes(data[i], data[i + 1], 0));
            } else if i + 1 == len {
                encoded.push_str(&encode3bytes(data[i], 0, 0));
            } else {
                encoded.push_str(&encode3bytes(data[i], data[i + 1], data[i + 2]));
            }
        }

        encoded
    }
}

fn encode3bytes(b1: u8, b2: u8, b3: u8) -> String {
    let c1 = b1 >> 2;
    let c2 = ((b1 & 0x3) << 4) | (b2 >> 4);
    let c3 = ((b2 & 0xF) << 2) | (b3 >> 6);
    let c4 = b3 & 0x3F;

    let mut res = String::from("");
    res.push(encode6bit(c1 & 0x3F));
    res.push(encode6bit(c2 & 0x3F));
    res.push(encode6bit(c3 & 0x3F));
    res.push(encode6bit(c4 & 0x3F));

    res
}

fn encode6bit(c: u8) -> char {
    let mut b = c;
    if b < 10 {
        return (48 + b) as char;
    }

    b -= 10;
    if b < 26 {
        return (65 + b) as char;
    }

    b -= 26;
    if b < 26 {
        return (97 + b) as char;
    }

    b -= 26;
    if b == 0 {
        return '-';
    }

    if b == 1 {
        return '_';
    }

    '?'
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn encodes_bytes() {
        let data: Vec<u8> = b"froboz".to_vec();
        assert_eq!(String::from("Pd9lOczw"), Base64PlantUML::encode(&data));

        let data: Vec<u8> = b"1234ABCDabcd\x12\x08\x01".to_vec();
        assert_eq!(
            String::from("CJ8pD452GqHXOcDa4WW1"),
            Base64PlantUML::encode(&data)
        );

        //How would one pass 256 here?
        let data: Vec<u8> = (0_u8..255_u8).collect();
        assert_eq!(
            String::from(
                "00420mG51WS82GeB30qE3n0H4XCK5HON61aQ6nmT7XyW8I8Z92Kc9o\
                 WfAYiiBIulC34oCpGrDZSuEJexF3q-Fq11GaD4HKP7I4bAIqnDJazGKL9JL5LMLr\
                 XPMbjSNLvVO65YOsHbPcTeQMfhR6rkRt1nSdDqTNPtU7bwUtnzVd-0WOA3X8M6Xu\
                 Y9YekCZOwFa96IavILbfUOcPgRd9sUdw2XegEafQQdgAcggwojhg-miRApjBMsjx\
                 YvkhkylRw_mC72myJ5niV8oShBpCtEpz3HqjFKrTRNsDdQszpTtj_WuUBZvENcv-\
                 ZfwklixUxlyF7oy_JrzlVu-Vhx_Ft-"
            ),
            Base64PlantUML::encode(&data)
        );
    }
}
