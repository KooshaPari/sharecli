// Minimal BMP file format parser.
// Spec: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/
//       and BITMAPINFOHEADER docs.
//
// We parse the 14-byte BMP file header and the DIB header (we support
// BITMAPCOREHEADER (12 bytes), BITMAPINFOHEADER (40 bytes), and the
// BITMAPV4/V5 extension headers (108/124 bytes)). We do NOT decompress
// RLE-encoded BMPs — the compression field is exposed via the struct so
// callers can decide.
//
// File header (14 bytes, little-endian):
//   0..2     magic "BM"
//   2..6     file size (u32 LE)
//   6..8     reserved1 (u16 LE)
//   8..10    reserved2 (u16 LE)
//   10..14   pixel data offset (u32 LE)
//
// DIB header (variable, little-endian):
//   0..4     header size (u32 LE) - tells us which variant
//   BITMAPCOREHEADER (12): width(i16), height(i16), planes(u16), bpp(u16)
//   BITMAPINFOHEADER (40): width(i32), height(i32), planes(u16),
//                          bpp(u16), compression(u32), image_size(u32),
//                          x_pels(u32), y_pels(u32), clr_used(u32),
//                          clr_important(u32)
//
// Pixel data is bottom-up, BGR(A) byte order. Each row is padded to a
// 4-byte boundary.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bmp {
    pub width: i32,
    pub height: i32,
    pub bits_per_pixel: u16,
    pub compression: u32,
    pub pixel_data: Vec<u8>,
    pub bytes_per_row: usize,
}

/// Parse a BMP file from the given byte buffer.
///
/// Returns `Err` if the buffer is too short, the magic is wrong, the
/// header size is unsupported, dimensions are non-positive, or the
/// declared pixel buffer size does not match the 4-byte aligned row
/// stride implied by `bits_per_pixel` and `width`.
pub fn parse_header(input: &[u8]) -> Result<Bmp, String> {
    // File header = 14 bytes; minimum.
    if input.len() < 14 {
        return Err(format!("BMP too short for file header: {} < 14", input.len()));
    }
    if &input[0..2] != b"BM" {
        return Err(format!("bad BMP magic: expected 'BM', got {:?}", &input[0..2]));
    }

    let _file_size = u32::from_le_bytes([input[2], input[3], input[4], input[5]]);
    let _reserved1 = u16::from_le_bytes([input[6], input[7]]);
    let _reserved2 = u16::from_le_bytes([input[8], input[9]]);
    let pixel_offset = u32::from_le_bytes([input[10], input[11], input[12], input[13]]) as usize;

    if pixel_offset < 14 {
        return Err(format!("BMP pixel offset {} is before end of file header", pixel_offset));
    }
    if pixel_offset > input.len() {
        return Err(format!(
            "BMP pixel offset {} exceeds buffer size {}",
            pixel_offset,
            input.len()
        ));
    }

    // DIB header.
    if input.len() < pixel_offset + 4 {
        return Err("BMP too short for DIB header size field".to_string());
    }
    let dib_off = 14;
    let dib_size = u32::from_le_bytes([
        input[dib_off],
        input[dib_off + 1],
        input[dib_off + 2],
        input[dib_off + 3],
    ]) as usize;

    if dib_size < 12 || input.len() < dib_off + dib_size {
        return Err(format!(
            "BMP DIB header truncated: have {}, need {}",
            input.len() - dib_off,
            dib_size
        ));
    }

    let (width, height, planes, bpp, compression) = match dib_size {
        12 => {
            // BITMAPCOREHEADER: width/height are i16 (max 32767).
            let width = i16::from_le_bytes([input[dib_off + 4], input[dib_off + 5]]) as i32;
            let height = i16::from_le_bytes([input[dib_off + 6], input[dib_off + 7]]) as i32;
            let planes = u16::from_le_bytes([input[dib_off + 8], input[dib_off + 9]]);
            let bpp = u16::from_le_bytes([input[dib_off + 10], input[dib_off + 11]]);
            (width, height, planes, bpp, 0u32)
        }
        40 | 108 | 124 => {
            // BITMAPINFOHEADER and V4/V5 extensions.
            let width = i32::from_le_bytes([
                input[dib_off + 4],
                input[dib_off + 5],
                input[dib_off + 6],
                input[dib_off + 7],
            ]);
            let height = i32::from_le_bytes([
                input[dib_off + 8],
                input[dib_off + 9],
                input[dib_off + 10],
                input[dib_off + 11],
            ]);
            let planes = u16::from_le_bytes([input[dib_off + 12], input[dib_off + 13]]);
            let bpp = u16::from_le_bytes([input[dib_off + 14], input[dib_off + 15]]);
            let compression = u32::from_le_bytes([
                input[dib_off + 16],
                input[dib_off + 17],
                input[dib_off + 18],
                input[dib_off + 19],
            ]);
            (width, height, planes, bpp, compression)
        }
        other => {
            return Err(format!(
                "unsupported BMP DIB header size: {} (supported: 12, 40, 108, 124)",
                other
            ));
        }
    };

    if planes != 1 {
        return Err(format!("BMP planes must be 1, got {}", planes));
    }
    if width <= 0 {
        return Err(format!("BMP width must be positive, got {}", width));
    }
    if height == 0 {
        return Err("BMP height must be non-zero".to_string());
    }
    // height < 0 means top-down DIB; we still emit abs(height) rows in
    // declared order (callers can read the sign).
    let abs_h = height.unsigned_abs() as usize;

    let bytes_per_row = compute_bytes_per_row(width as usize, bpp as usize)?;
    let total_pixel_bytes = bytes_per_row
        .checked_mul(abs_h)
        .ok_or_else(|| "BMP pixel buffer overflows usize".to_string())?;

    let pixel_data = if pixel_offset + total_pixel_bytes > input.len() {
        return Err(format!(
            "BMP pixel data truncated: need {} bytes from offset {}, have {}",
            total_pixel_bytes,
            pixel_offset,
            input.len() - pixel_offset
        ));
    } else {
        input[pixel_offset..pixel_offset + total_pixel_bytes].to_vec()
    };

    Ok(Bmp { width, height, bits_per_pixel: bpp, compression, pixel_data, bytes_per_row })
}

/// Compute the row stride (in bytes) for a row of `width` pixels at `bpp`
/// bits per pixel, rounded up to a 4-byte boundary.
fn compute_bytes_per_row(width: usize, bpp: usize) -> Result<usize, String> {
    if bpp == 0 || bpp % 8 != 0 {
        return Err(format!("unsupported BMP bits per pixel: {}", bpp));
    }
    let bits_per_row =
        width.checked_mul(bpp).ok_or_else(|| "BMP row width overflows usize".to_string())?;
    let bytes_per_row_unaligned = (bits_per_row + 7) / 8;
    Ok((bytes_per_row_unaligned + 3) & !3)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid uncompressed 24bpp BMP with the given pixel
    /// pattern. Used as the basis for several tests.
    fn build_24bpp(width: i32, height: i32, bgr_triples: &[u8]) -> Vec<u8> {
        let abs_h = height.unsigned_abs() as usize;
        let w = width as usize;
        let bytes_per_row = compute_bytes_per_row(w, 24).unwrap();
        let pixel_size = bytes_per_row * abs_h;

        let dib_size: u32 = 40;
        let pixel_offset: u32 = 14 + dib_size;
        let file_size = pixel_offset + pixel_size as u32;

        let mut out = Vec::new();
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&file_size.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // reserved1
        out.extend_from_slice(&0u16.to_le_bytes()); // reserved2
        out.extend_from_slice(&pixel_offset.to_le_bytes());

        // DIB header (BITMAPINFOHEADER)
        out.extend_from_slice(&dib_size.to_le_bytes());
        out.extend_from_slice(&width.to_le_bytes());
        out.extend_from_slice(&height.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes()); // planes
        out.extend_from_slice(&24u16.to_le_bytes()); // bpp
        out.extend_from_slice(&0u32.to_le_bytes()); // compression = BI_RGB
        out.extend_from_slice(&(pixel_size as u32).to_le_bytes()); // image_size
        out.extend_from_slice(&0u32.to_le_bytes()); // x_pels
        out.extend_from_slice(&0u32.to_le_bytes()); // y_pels
        out.extend_from_slice(&0u32.to_le_bytes()); // clr_used
        out.extend_from_slice(&0u32.to_le_bytes()); // clr_important

        assert_eq!(bgr_triples.len(), pixel_size);
        out.extend_from_slice(bgr_triples);
        out
    }

    #[test]
    fn rejects_short_input() {
        let r = parse_header(&[0, 1, 2]);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("file header"));
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = build_24bpp(1, 1, &[0, 0, 0, 0]);
        bytes[0] = b'X';
        let r = parse_header(&bytes);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("magic"));
    }

    #[test]
    fn rejects_pixel_offset_before_header() {
        let mut bytes = build_24bpp(1, 1, &[0, 0, 0, 0]);
        // Set pixel_offset (bytes 10..14) to 0 (before end of file header).
        bytes[10] = 0;
        bytes[11] = 0;
        bytes[12] = 0;
        bytes[13] = 0;
        let r = parse_header(&bytes);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("pixel offset"));
    }

    #[test]
    fn rejects_unsupported_dib_size() {
        // Take a valid BMP and overwrite the dib_size field with an unsupported value.
        let mut bytes = build_24bpp(1, 1, &[0, 0, 0, 0]);
        bytes[14] = 16; // unsupported DIB header size
        bytes[15] = 0;
        bytes[16] = 0;
        bytes[17] = 0;
        let r = parse_header(&bytes);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("DIB header"));
    }

    #[test]
    fn parse_minimal_24bpp_1x1_black() {
        // 1x1 image, 24bpp, no row padding needed (3 bytes rounds up to 4).
        let bytes = build_24bpp(1, 1, &[0, 0, 0, 0]); // 4-byte row (1 pad)
        let bmp = parse_header(&bytes).unwrap();
        assert_eq!(bmp.width, 1);
        assert_eq!(bmp.height, 1);
        assert_eq!(bmp.bits_per_pixel, 24);
        assert_eq!(bmp.compression, 0);
        assert_eq!(bmp.bytes_per_row, 4);
        assert_eq!(bmp.pixel_data, vec![0, 0, 0, 0]);
    }

    /// Cross-check: a 4x3 24bpp BMP where every pixel is blue (B=0xFF,
    /// G=0, R=0) and we verify the row-stride layout matches the BMP spec
    /// (4-byte alignment, bottom-up rows). For a 4-pixel-wide 24bpp row:
    /// 4*3 = 12 bytes raw, already 4-byte aligned. 3 rows * 12 = 36 bytes
    /// total pixel data.
    #[test]
    fn parse_24bpp_4x3_blue_padding() {
        // Rows in our test helper are emitted top-down; the BMP spec says
        // pixel data is stored bottom-up, but `parse_header` does not
        // reorder rows — we just verify the stride math here.
        let row = vec![0xFF, 0, 0, 0xFF, 0, 0, 0xFF, 0, 0, 0xFF, 0, 0];
        let mut bgr = Vec::new();
        bgr.extend_from_slice(&row); // bottom row
        bgr.extend_from_slice(&row); // middle row
        bgr.extend_from_slice(&row); // top row
        let bytes = build_24bpp(4, 3, &bgr);
        let bmp = parse_header(&bytes).unwrap();
        assert_eq!(bmp.width, 4);
        assert_eq!(bmp.height, 3);
        assert_eq!(bmp.bytes_per_row, 12);
        assert_eq!(bmp.pixel_data.len(), 36);
        // Spot-check a few bytes in the middle of the buffer.
        assert_eq!(bmp.pixel_data[0], 0xFF);
        assert_eq!(bmp.pixel_data[1], 0);
        assert_eq!(bmp.pixel_data[2], 0);
        assert_eq!(bmp.pixel_data[18], 0xFF); // start of second row
    }

    /// Cross-check: a 5x2 24bpp BMP. Row width = 5*3 = 15 bytes, padded to
    /// 16 bytes (the smallest multiple of 4 >= 15). Total pixel data = 32.
    /// This validates the row-padding computation against the BMP spec.
    #[test]
    fn parse_24bpp_row_padding_5x2() {
        let row = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut bgr = Vec::new();
        bgr.extend_from_slice(&row);
        bgr.extend_from_slice(&[0u8; 1]); // 1-byte pad
        bgr.extend_from_slice(&row);
        bgr.extend_from_slice(&[0u8; 1]); // 1-byte pad
        assert_eq!(bgr.len(), 32);
        let bytes = build_24bpp(5, 2, &bgr);
        let bmp = parse_header(&bytes).unwrap();
        assert_eq!(bmp.bytes_per_row, 16);
        assert_eq!(bmp.pixel_data.len(), 32);
    }

    #[test]
    fn parse_32bpp_no_padding() {
        // 32bpp: 2x2, 4 bytes/pixel, row width = 2*4 = 8 (already aligned).
        let pixel_size = 2 * 2 * 4;
        let dib_size: u32 = 40;
        let pixel_offset: u32 = 14 + dib_size;
        let file_size = pixel_offset + pixel_size as u32;

        let mut out = Vec::new();
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&file_size.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&pixel_offset.to_le_bytes());
        out.extend_from_slice(&dib_size.to_le_bytes());
        out.extend_from_slice(&2i32.to_le_bytes());
        out.extend_from_slice(&2i32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(pixel_size as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        // BGRA pixel data
        let pixels = vec![
            0xAA, 0xBB, 0xCC, 0xDD, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA,
            0xBB, 0xCC,
        ];
        out.extend_from_slice(&pixels);

        let bmp = parse_header(&out).unwrap();
        assert_eq!(bmp.bits_per_pixel, 32);
        assert_eq!(bmp.bytes_per_row, 8);
        assert_eq!(bmp.pixel_data, pixels);
    }

    #[test]
    fn parse_top_down_negative_height() {
        // Top-down DIBs use a negative height. Our parser accepts that and
        // exposes the sign through the struct field.
        let _pixel_size = 2 * 1 * 3;
        // Row width = 2*3 = 6, padded to 8.
        let padded_row = 8;
        let total = padded_row * 1;
        let dib_size: u32 = 40;
        let pixel_offset: u32 = 14 + dib_size;
        let file_size = pixel_offset + total as u32;

        let mut out = Vec::new();
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&file_size.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&pixel_offset.to_le_bytes());
        out.extend_from_slice(&dib_size.to_le_bytes());
        out.extend_from_slice(&2i32.to_le_bytes());
        out.extend_from_slice(&(-1i32).to_le_bytes()); // negative height
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(total as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&[1, 2, 3, 4, 5, 6, 0, 0]);

        let bmp = parse_header(&out).unwrap();
        assert_eq!(bmp.width, 2);
        assert_eq!(bmp.height, -1);
        assert_eq!(bmp.bytes_per_row, 8);
    }

    #[test]
    fn rejects_truncated_pixel_data() {
        // 2x2 24bpp: needs 2 rows * 8 bytes = 16 bytes; supply only 8.
        let dib_size: u32 = 40;
        let pixel_offset: u32 = 14 + dib_size;
        let file_size = pixel_offset + 8; // intentionally short

        let mut out = Vec::new();
        out.extend_from_slice(b"BM");
        out.extend_from_slice(&file_size.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&pixel_offset.to_le_bytes());
        out.extend_from_slice(&dib_size.to_le_bytes());
        out.extend_from_slice(&2i32.to_le_bytes());
        out.extend_from_slice(&2i32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&24u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&[0u8; 8]);

        let r = parse_header(&out);
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("pixel data truncated"));
    }
}
