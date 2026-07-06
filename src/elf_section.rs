// Minimal ELF section header walker. Parses ELF32 and ELF64 section headers,
// extracts the section name string table, and lists sections with their type.
// This is intentionally smaller than `elf_section::macho_parse` siblings: no
// program-header interpretation, no relocations, no symbol table.

pub const ELFCLASS32: u8 = 1;
pub const ELFCLASS64: u8 = 2;
pub const ELFDATA2LSB: u8 = 1;
pub const ELFDATA2MSB: u8 = 2;

pub const SHT_NULL: u32 = 0;
pub const SHT_PROGBITS: u32 = 1;
pub const SHT_SYMTAB: u32 = 2;
pub const SHT_STRTAB: u32 = 3;
pub const SHT_RELA: u32 = 4;
pub const SHT_HASH: u32 = 5;
pub const SHT_DYNAMIC: u32 = 6;
pub const SHT_NOTE: u32 = 7;
pub const SHT_NOBITS: u32 = 8;
pub const SHT_REL: u32 = 9;
pub const SHT_DYNSYM: u32 = 11;

pub const SHF_WRITE: u64 = 0x1;
pub const SHF_ALLOC: u64 = 0x2;
pub const SHF_EXECINSTR: u64 = 0x4;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section {
    pub name: String,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
    pub data: Vec<u8>,
}

pub struct ElfMeta {
    pub is_64: bool,
    pub is_le: bool,
    pub e_machine: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

fn read_u16(is_le: bool, b: &[u8], off: usize) -> u16 {
    if is_le { u16::from_le_bytes([b[off], b[off+1]]) } else { u16::from_be_bytes([b[off], b[off+1]]) }
}
fn read_u32(is_le: bool, b: &[u8], off: usize) -> u32 {
    if is_le { u32::from_le_bytes([b[off], b[off+1], b[off+2], b[off+3]]) } else { u32::from_be_bytes([b[off], b[off+1], b[off+2], b[off+3]]) }
}
fn read_u64(is_le: bool, b: &[u8], off: usize) -> u64 {
    let arr: [u8; 8] = b[off..off+8].try_into().unwrap();
    if is_le { u64::from_le_bytes(arr) } else { u64::from_be_bytes(arr) }
}

pub fn parse_meta(input: &[u8]) -> Result<ElfMeta, String> {
    if input.len() < 52 { return Err("input too short".into()); }
    if &input[0..4] != b"\x7fELF" { return Err("not ELF".into()); }
    let is_64 = input[4] == ELFCLASS64;
    let is_le = input[5] == ELFDATA2LSB;
    let machine_off = if is_64 { 18 } else { 18 };
    let shnum_off = if is_64 { 60 } else { 48 };
    let shstrndx_off = if is_64 { 62 } else { 50 };
    if input.len() < shstrndx_off + 2 { return Err("header truncated".into()); }
    Ok(ElfMeta {
        is_64,
        is_le,
        e_machine: read_u16(is_le, input, machine_off),
        e_shnum: read_u16(is_le, input, shnum_off),
        e_shstrndx: read_u16(is_le, input, shstrndx_off),
    })
}

fn read_cstr(data: &[u8], off: usize) -> &str {
    if off >= data.len() { return ""; }
    let mut end = off;
    while end < data.len() && data[end] != 0 { end += 1; }
    std::str::from_utf8(&data[off..end]).unwrap_or("")
}

pub fn parse_sections(input: &[u8]) -> Result<Vec<Section>, String> {
    let meta = parse_meta(input)?;
    if meta.e_shnum == 0 { return Ok(Vec::new()); }
    let shoff_off = if meta.is_64 { 40 } else { 32 };
    let shentsize_off = if meta.is_64 { 58 } else { 46 };
    if input.len() < shentsize_off + 2 { return Err("header truncated".into()); }
    let e_shoff = if meta.is_64 { read_u64(meta.is_le, input, shoff_off) } else { read_u32(meta.is_le, input, shoff_off) as u64 };
    let e_shentsize = read_u16(meta.is_le, input, shentsize_off);
    if e_shoff == 0 { return Ok(Vec::new()); }
    let entry_size = if meta.is_64 { 64 } else { 40 };
    let actual_size = if e_shentsize as usize > 0 { e_shentsize as usize } else { entry_size };
    let mut raw_sections = Vec::with_capacity(meta.e_shnum as usize);
    for i in 0..meta.e_shnum as usize {
        let off = e_shoff as usize + i * actual_size;
        if off + actual_size > input.len() { return Err(format!("section header {} past EOF", i)); }
        let s = Section {
            name: String::new(),
            sh_type: read_u32(meta.is_le, input, off + 4),
            sh_flags: if meta.is_64 { read_u64(meta.is_le, input, off + 8) } else { read_u32(meta.is_le, input, off + 8) as u64 },
            sh_addr: if meta.is_64 { read_u64(meta.is_le, input, off + 16) } else { read_u32(meta.is_le, input, off + 12) as u64 },
            sh_offset: if meta.is_64 { read_u64(meta.is_le, input, off + 24) } else { read_u32(meta.is_le, input, off + 16) as u64 },
            sh_size: if meta.is_64 { read_u64(meta.is_le, input, off + 32) } else { read_u32(meta.is_le, input, off + 20) as u64 },
            sh_link: read_u32(meta.is_le, input, if meta.is_64 { off + 40 } else { off + 24 }),
            sh_info: read_u32(meta.is_le, input, if meta.is_64 { off + 44 } else { off + 28 }),
            sh_addralign: if meta.is_64 { read_u64(meta.is_le, input, off + 48) } else { read_u32(meta.is_le, input, off + 32) as u64 },
            sh_entsize: if meta.is_64 { read_u64(meta.is_le, input, off + 56) } else { read_u32(meta.is_le, input, off + 36) as u64 },
            data: Vec::new(),
        };
        raw_sections.push((read_u32(meta.is_le, input, off), s));
    }
    let shstrndx = meta.e_shstrndx as usize;
    let shstrtab_data: Vec<u8> = if shstrndx < raw_sections.len() {
        let (_, ref sec) = raw_sections[shstrndx];
        if sec.sh_type == SHT_STRTAB && sec.sh_offset as usize + sec.sh_size as usize <= input.len() {
            input[sec.sh_offset as usize..(sec.sh_offset + sec.sh_size) as usize].to_vec()
        } else { Vec::new() }
    } else { Vec::new() };
    let mut out = Vec::with_capacity(raw_sections.len());
    for (name_off, mut s) in raw_sections {
        s.name = read_cstr(&shstrtab_data, name_off as usize).to_string();
        if s.sh_type != SHT_NOBITS && s.sh_offset as usize + s.sh_size as usize <= input.len() && s.sh_size > 0 {
            s.data = input[s.sh_offset as usize..(s.sh_offset + s.sh_size) as usize].to_vec();
        }
        out.push(s);
    }
    Ok(out)
}

pub fn section_type_str(t: u32) -> &'static str {
    match t {
        0 => "NULL",
        1 => "PROGBITS",
        2 => "SYMTAB",
        3 => "STRTAB",
        4 => "RELA",
        5 => "HASH",
        6 => "DYNAMIC",
        7 => "NOTE",
        8 => "NOBITS",
        9 => "REL",
        11 => "DYNSYM",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn not_elf() {
        assert!(parse_meta(b"not elf").is_err());
    }
    #[test] fn truncated() {
        assert!(parse_meta(&[0u8; 10]).is_err());
    }
    #[test] fn parse_meta_64_le() {
        let mut elf = Vec::new();
        elf.extend_from_slice(b"\x7fELF");
        elf.push(2); elf.push(1); elf.push(1);
        elf.extend_from_slice(&[0u8; 9]);
        elf.extend_from_slice(&2u16.to_le_bytes()); // e_type = ET_EXEC
        elf.extend_from_slice(&62u16.to_le_bytes()); // machine
        elf.extend_from_slice(&1u32.to_le_bytes());
        elf.extend_from_slice(&[0u8; 8]); // e_entry
        elf.extend_from_slice(&[0u8; 8]); // e_phoff
        elf.extend_from_slice(&[0u8; 8]); // e_shoff
        elf.extend_from_slice(&[0u8; 4]); // e_flags
        elf.extend_from_slice(&64u16.to_le_bytes()); // e_ehsize
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_phentsize
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_phnum
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shentsize
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shnum
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shstrndx
        elf.extend_from_slice(&[0u8; 2]); // pad to 64
        let m = parse_meta(&elf).unwrap();
        assert!(m.is_64);
        assert!(m.is_le);
        assert_eq!(m.e_machine, 62);
    }
    #[test] fn parse_sections_empty() {
        let mut elf = Vec::new();
        elf.extend_from_slice(b"\x7fELF");
        elf.push(2); elf.push(1); elf.push(1);
        elf.extend_from_slice(&[0u8; 9]);
        elf.extend_from_slice(&2u16.to_le_bytes()); // e_type
        elf.extend_from_slice(&62u16.to_le_bytes());
        elf.extend_from_slice(&1u32.to_le_bytes());
        elf.extend_from_slice(&[0u8; 8]);
        elf.extend_from_slice(&[0u8; 8]);
        elf.extend_from_slice(&[0u8; 8]);
        elf.extend_from_slice(&[0u8; 4]);
        elf.extend_from_slice(&64u16.to_le_bytes());
        elf.extend_from_slice(&0u16.to_le_bytes());
        elf.extend_from_slice(&0u16.to_le_bytes());
        elf.extend_from_slice(&0u16.to_le_bytes());
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shnum
        elf.extend_from_slice(&0u16.to_le_bytes()); // e_shstrndx
        elf.extend_from_slice(&[0u8; 2]); // pad to 64
        let secs = parse_sections(&elf).unwrap();
        assert!(secs.is_empty());
    }
    #[test] fn read_cstr_works() {
        assert_eq!(read_cstr(b"\x00hello\x00", 1), "hello");
        assert_eq!(read_cstr(b"\x00", 0), "");
        assert_eq!(read_cstr(b"abc", 100), "");
    }
    #[test] fn sh_type_known() {
        assert_eq!(section_type_str(SHT_SYMTAB), "SYMTAB");
        assert_eq!(section_type_str(SHT_STRTAB), "STRTAB");
    }
}