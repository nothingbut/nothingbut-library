use md5;

pub fn wr_hash(input: &str) -> String {
    if input.chars().all(|c| c.is_ascii_digit()) {
        let chunks = encode_number(input);
        let md5_prefix = &format!("{:x}", md5::compute(input.as_bytes()))[..3];
        let md5_suffix = &format!("{:x}", md5::compute(chunks.as_bytes()))[..2];
        format!("{}{}{}", md5_prefix, chunks, md5_suffix)
    } else {
        let encoded = encode_string(input);
        let md5_prefix = &format!("{:x}", md5::compute(input.as_bytes()))[..3];
        let md5_suffix = &format!("{:x}", md5::compute(encoded.as_bytes()))[..2];
        format!("{}{}{}", md5_prefix, encoded, md5_suffix)
    }
}

fn encode_number(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        let end = (i + 9).min(chars.len());
        let chunk: String = chars[i..end].iter().collect();
        let hex = format!("{:x}", chunk.parse::<u64>().unwrap_or(0));
        result.push_str(&format!("{:x}{}", hex.len(), hex));
        i = end;
    }

    result
}

fn encode_string(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        result.push_str(&format!("{:x}", ch as u32));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wr_hash_numeric() {
        let hash = wr_hash("123456789");
        assert!(!hash.is_empty());
        assert!(hash.len() > 5);
    }

    #[test]
    fn test_wr_hash_large_number() {
        let hash = wr_hash("1234567890123");
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_encode_number_single_chunk() {
        let result = encode_number("123456789");
        let expected_hex = format!("{:x}", 123456789u64);
        assert!(result.contains(&expected_hex));
    }

    #[test]
    fn test_encode_number_multi_chunk() {
        let result = encode_number("1234567890123");
        assert!(!result.is_empty());
    }
}
