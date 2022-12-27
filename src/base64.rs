use base64::{
    alphabet::Alphabet,
    engine::fast_portable::{self, FastPortable},
};

const ENGINE: FastPortable =
    match Alphabet::from_str("0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_") {
        Ok(alphabet) => FastPortable::from(&alphabet, fast_portable::PAD),
        Err(_e) => unreachable!(),
    };

/// PlantUML has its own base64 dialect
pub fn encode(data: &[u8]) -> String {
    base64::encode_engine(data, &ENGINE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn encodes_bytes() {
        assert_eq!(String::from("Pd9lOczw"), encode(b"froboz"));
        assert_eq!(
            String::from("CJ8pD452GqHXOcDa4WW1"),
            encode(b"1234ABCDabcd\x12\x08\x01")
        );

        // How would one pass 256 here?
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
            encode(&data)
        );
    }
}
