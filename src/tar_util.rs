pub struct TarEntry { pub name: String, pub size: u64, pub offset: usize }

pub fn parse_ustar(data: &[u8]) -> Vec<TarEntry> {
    let mut entries = Vec::new();
    let mut i = 0;
    while i + 512 <= data.len() {
        let header = &data[i..i + 512];
        let name = parse_name_field(&header[0..100]);
        if name.is_empty() { break; }
        let size_str = std::str::from_utf8(&header[124..136]).unwrap_or("").trim_end_matches('\0').trim_end();
        let size: u64 = u64::from_str_radix(size_str, 8).unwrap_or(0);
        let name_s = name.clone();
        entries.push(TarEntry { name: name_s, size, offset: i + 512 });
        i += 512 + ((size as usize + 511) / 512) * 512;
    }
    entries
}

fn parse_name_field(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    std::str::from_utf8(&data[..end]).map(|s| s.to_string()).unwrap_or_default()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn empty() { assert_eq!(parse_ustar(&[]).len(), 0); }
    #[test] fn header_only() {
        let mut buf = vec![0u8; 512];
        buf[0..9].copy_from_slice(b"test.txt");
        let s = format!("{:011o}", 0u64);
        buf[124..124 + s.len()].copy_from_slice(s.as_bytes());
        let entries = parse_ustar(&buf);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test.txt");
        assert_eq!(entries[0].size, 0);
    }
}
