use encoding_rs::WINDOWS_1250;

pub fn cp1250_to_utf8(bytes: &[u8]) -> String {
    let (cow, _, _) = WINDOWS_1250.decode(bytes);
    cow.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_czech_diacritics() {
        // cp1250: 'TRÁVICÍ' — 0xC1='Á', 0xCD='Í'
        let bytes = [b'T', b'R', 0xC1, b'V', b'I', b'C', 0xCD];
        assert_eq!(cp1250_to_utf8(&bytes), "TRÁVICÍ");
    }
}
