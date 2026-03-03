use rand::Rng;

pub fn generate_segment(length: usize) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();

    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARS.len());
            CHARS[idx] as char
        })
        .collect()
}

pub fn generate_license_key(tier: &str) -> String {
    let prefix = tier.to_uppercase();
    let seg1 = generate_segment(4);
    let seg2 = generate_segment(4);
    let seg3 = generate_segment(4);
    
    format!("{}-{}-{}-{}", prefix, seg1, seg2, seg3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pro_key() {
        let key = generate_license_key("PRO");
        assert_eq!(key.len(), 18);
        assert!(key.starts_with("PRO-"));
    }

}

