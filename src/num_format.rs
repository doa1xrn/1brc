// format an integer with thousands separators
pub fn thousands(u: u64) -> String {
    let s = u.to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut s = String::with_capacity(len + (len / 3));
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            s.push(',');
        }
        s.push(*b as char);
    }
    s
}
