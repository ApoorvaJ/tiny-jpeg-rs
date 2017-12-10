extern crate byteorder;

use byteorder::{BigEndian, WriteBytesExt};
use std::io;
use std::io::Write;
use std::fs::File;
use std::path::Path;

const QT_SIZE: usize = 64;

struct State {
    // Huffman data
    ehuffsize: [[u8; 257]; 4],
    ehuffcode: [[u16; 256]; 4],
    ht_bits: [&'static [u8]; 4],
    ht_vals: [&'static [u8]; 4],
    // Quantization tables
    qt_luma: [u8; QT_SIZE],
    qt_chroma: [u8; QT_SIZE],
}

const DEFAULT_QT_LUMA_FROM_SPEC: [u8; QT_SIZE] = [
    16, 11, 10, 16, 24, 40, 51, 61,
    12, 12, 14, 19, 26, 58, 60, 55,
    14, 13, 16, 24, 40, 57, 69, 56,
    14, 17, 22, 29, 51, 87, 80, 62,
    18, 22, 37, 56, 68, 109, 103, 77,
    24, 35, 55, 64, 81, 104, 113, 92,
    49, 64, 78, 87, 103, 121, 120, 101,
    72, 92, 95, 98, 112, 100, 103, 99,
];

const DETAULT_QT_CHROMA_FROM_PAPER: [u8; QT_SIZE] = [
    16, 12, 14, 14, 18, 24, 49, 72,
    11, 10, 16, 24, 40, 51, 61, 12,
    13, 17, 22, 35, 64, 92, 14, 16,
    22, 37, 55, 78, 95, 19, 24, 29,
    56, 64, 87, 98, 26, 40, 51, 68,
    81, 103, 112, 58, 57, 87, 109, 104,
    121, 100, 60, 69, 80, 103, 113, 120,
    103, 55, 56, 62, 77, 92, 101, 99,
];

const DEFAULT_HT_LUMA_DC_LEN: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
const DEFAULT_HT_LUMA_AC_LEN: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d];
const DEFAULT_HT_CHROMA_DC_LEN: [u8; 16] = [0, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
const DEFAULT_HT_CHROMA_AC_LEN: [u8; 16] = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 0x77];

const DEFAULT_HT_LUMA_DC: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const DEFAULT_HT_LUMA_AC: [u8; 162] = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
    0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0,
    0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
    0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
    0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
    0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5,
    0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
    0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xF9, 0xFA,
];
const DEFAULT_HT_CHROMA_DC: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
const DEFAULT_HT_CHROMA_AC: [u8; 162] = [
    0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
    0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xA1, 0xB1, 0xC1, 0x09, 0x23, 0x33, 0x52, 0xF0,
    0x15, 0x62, 0x72, 0xD1, 0x0A, 0x16, 0x24, 0x34, 0xE1, 0x25, 0xF1, 0x17, 0x18, 0x19, 0x1A, 0x26,
    0x27, 0x28, 0x29, 0x2A, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
    0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
    0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5,
    0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3,
    0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA,
    0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xF9,
    0xFA,
];

const ZIG_ZAG: [usize; 64] = [
    0, 1, 5, 6, 14, 15, 27, 28,
    2, 4, 7, 13, 16, 26, 29, 42,
    3, 8, 12, 17, 25, 30, 41, 43,
    9, 11, 18, 24, 31, 40, 44, 53,
    10, 19, 23, 32, 39, 45, 52, 54,
    20, 22, 33, 38, 46, 51, 55, 60,
    21, 34, 37, 47, 50, 56, 59, 61,
    35, 36, 48, 49, 57, 58, 62, 63,
];

fn append_dqt(out: &mut Vec<u8>, matrix: &[u8], id: u8) {
    out.write_u16::<BigEndian>(0xffdb).unwrap();
    out.write_u16::<BigEndian>(0x0043).unwrap(); // 2(len) + 1(id) + 64(matrix) = 67 = 0x43
    debug_assert!(id < 4);
    out.push(id);
    out.extend_from_slice(matrix);
}

fn append_dht(out: &mut Vec<u8>, matrix_len: &[u8], matrix_val: &[u8], ht_class: i32, id: u8) {
    // DHT
    out.write_u16::<BigEndian>(0xffc4).unwrap();

    // 2(len) + 1(Tc|th) + 16 (num lengths) + ?? (num values)
    let mut num_values = 0usize;
    for i in 0..16 {
        num_values += matrix_len[i] as usize;
    }
    debug_assert!(num_values <= 0xffff);
    let len: u16 = 2 + 1 + 16 + num_values as u16;
    out.write_u16::<BigEndian>(len).unwrap();

    // tc_th
    debug_assert!(id < 4);
    let tc_th: u8 = ((ht_class as u8) << 4) | id;
    out.push(tc_th);

    out.extend_from_slice(matrix_len);
    out.extend_from_slice(matrix_val);
}

fn huff_get_code_lengths(huffsize: &mut [u8], bits: &[u8]) {
    let mut k = 0;
    for i in 0..16 {
        for _ in 0..bits[i] {
            huffsize[k] = (i + 1) as u8;
            k += 1;
        }
        huffsize[k] = 0;
    }
}

fn huff_get_codes(codes: &mut [u16], huffsize: &[u8], count: usize) {
    let mut code = 0u16;
    let mut k = 0usize;
    let mut sz: u8 = huffsize[0];

    loop {
        loop {
            debug_assert!(k < count);
            codes[k] = code;
            k += 1;
            code += 1;
            if huffsize[k] != sz {
                break;
            }
        }
        if huffsize[k] == 0 {
            return;
        }
        loop {
            code = code << 1;
            sz += 1;
            if huffsize[k] == sz {
                break;
            }
        }
    }
}

fn huff_get_extended(
    out_ehuffsize: &mut [u8],
    out_ehuffcode: &mut [u16],
    huffval: &[u8],
    huffsize: &[u8],
    huffcode: &[u16],
    count: usize,
) {
    let mut k = 0usize;
    loop {
        let val = huffval[k] as usize;
        out_ehuffcode[val] = huffcode[k];
        out_ehuffsize[val] = huffsize[k];
        k += 1;
        if k >= count {
            break;
        }
    }
}

// Returns: (bits, num_bits)
fn calculate_variable_length_int(mut val: i32) -> (u16, u16) {
    let mut abs_val = val;
    if val < 0 {
        abs_val = -abs_val;
        val -= 1;
    }

    let mut num_bits = 1;
    loop {
        abs_val = abs_val >> 1;
        if abs_val == 0 {
            break;
        }
        num_bits += 1;
    }

    let bits = (val & ((1 << num_bits) - 1)) as u16;

    (bits, num_bits)
}

fn append_bits(
    out: &mut Vec<u8>,
    bitbuffer: &mut u32,
    location: &mut u32,
    num_bits: u16,
    bits: u16,
) {
    /*
         v-- location
        [                     ]   <-- bit buffer
       32                     0

       This call pushes to the bitbuffer and saves the location. Data is pushed
       from most significant to less significant.
       When we can write a full byte, we write a byte and shift. */

    // Push the stack.
    let nloc = *location + num_bits as u32;
    *bitbuffer |= ((bits as u32) << (32 - nloc)) as u32;
    *location = nloc;
    while *location >= 8 {
        // Grab the most significant byte.
        let c = ((*bitbuffer) >> 24) as u8;
        // Write it to file.
        out.push(c);
        if c == 0xff {
            // Special case: tell JPEG this is not a marker.
            out.push(0);
        }
        // Pop the stack.
        *bitbuffer <<= 8;
        *location -= 8;
    }
}

fn fdct(data: &mut [f32]) {
    let (mut tmp0,
         mut tmp1,
         mut tmp2,
         mut tmp3,
         mut tmp4,
         mut tmp5,
         mut tmp6,
         mut tmp7,
         mut tmp10,
         mut tmp11,
         mut tmp12,
         mut tmp13);
    let (mut z1, mut z2, mut z3, mut z4, mut z5, mut z11, mut z13);

    /* Pass 1: process rows. */

    let mut i = 0;
    for _ in (0..8).rev() {
        tmp0 = data[i + 0] + data[i + 7];
        tmp7 = data[i + 0] - data[i + 7];
        tmp1 = data[i + 1] + data[i + 6];
        tmp6 = data[i + 1] - data[i + 6];
        tmp2 = data[i + 2] + data[i + 5];
        tmp5 = data[i + 2] - data[i + 5];
        tmp3 = data[i + 3] + data[i + 4];
        tmp4 = data[i + 3] - data[i + 4];

        // Even part
        tmp10 = tmp0 + tmp3; // phase 2
        tmp13 = tmp0 - tmp3;
        tmp11 = tmp1 + tmp2;
        tmp12 = tmp1 - tmp2;

        data[i + 0] = tmp10 + tmp11; // phase 3
        data[i + 4] = tmp10 - tmp11;

        z1 = (tmp12 + tmp13) * 0.707106781; // c4
        data[i + 2] = tmp13 + z1; // phase 5
        data[i + 6] = tmp13 - z1;

        // Odd part
        tmp10 = tmp4 + tmp5; // phase 2
        tmp11 = tmp5 + tmp6;
        tmp12 = tmp6 + tmp7;

        // The rotator is modified from fig 4-8 to avoid extra negations
        z5 = (tmp10 - tmp12) * 0.382683433; // c6
        z2 = 0.541196100 * tmp10 + z5; // c2-c6
        z4 = 1.306562965 * tmp12 + z5; // c2+c6
        z3 = tmp11 * 0.707106781; // c4

        z11 = tmp7 + z3; // phase 5
        z13 = tmp7 - z3;

        data[i + 5] = z13 + z2; // phase 6
        data[i + 3] = z13 - z2;
        data[i + 1] = z11 + z4;
        data[i + 7] = z11 - z4;

        i += 8; // Advance pointer to next row
    }

    // Pass 2: process columns.

    i = 0;
    for _ in (0..8).rev() {
        tmp0 = data[i + 8 * 0] + data[i + 8 * 7];
        tmp7 = data[i + 8 * 0] - data[i + 8 * 7];
        tmp1 = data[i + 8 * 1] + data[i + 8 * 6];
        tmp6 = data[i + 8 * 1] - data[i + 8 * 6];
        tmp2 = data[i + 8 * 2] + data[i + 8 * 5];
        tmp5 = data[i + 8 * 2] - data[i + 8 * 5];
        tmp3 = data[i + 8 * 3] + data[i + 8 * 4];
        tmp4 = data[i + 8 * 3] - data[i + 8 * 4];

        // Even part
        tmp10 = tmp0 + tmp3; /* phase 2 */
        tmp13 = tmp0 - tmp3;
        tmp11 = tmp1 + tmp2;
        tmp12 = tmp1 - tmp2;

        data[i + 8 * 0] = tmp10 + tmp11; /* phase 3 */
        data[i + 8 * 4] = tmp10 - tmp11;

        z1 = (tmp12 + tmp13) * 0.707106781; /* c4 */
        data[i + 8 * 2] = tmp13 + z1; /* phase 5 */
        data[i + 8 * 6] = tmp13 - z1;

        // Odd part

        tmp10 = tmp4 + tmp5; /* phase 2 */
        tmp11 = tmp5 + tmp6;
        tmp12 = tmp6 + tmp7;

        // The rotator is modified from fig 4-8 to avoid extra negations.
        z5 = (tmp10 - tmp12) * 0.382683433; /* c6 */
        z2 = 0.541196100 * tmp10 + z5; /* c2-c6 */
        z4 = 1.306562965 * tmp12 + z5; /* c2+c6 */
        z3 = tmp11 * 0.707106781; /* c4 */

        z11 = tmp7 + z3; /* phase 5 */
        z13 = tmp7 - z3;

        data[i + 8 * 5] = z13 + z2; /* phase 6 */
        data[i + 8 * 3] = z13 - z2;
        data[i + 8 * 1] = z11 + z4;
        data[i + 8 * 7] = z11 - z4;

        i += 1; /* advance pointer to next column */
    }
}

/* DCT implementation by Thomas G. Lane.
   Obtained through NVIDIA
    http://developer.download.nvidia.com/SDK/9.5/Samples/vidimaging_samples.html#gpgpu_dct

   This implementation is based on Arai, Agui, and Nakajima's algorithm for
   scaled DCT.  Their original paper (Trans. IEICE E-71(11):1095) is in
   Japanese, but the algorithm is described in the Pennebaker & Mitchell JPEG
   textbook (see REFERENCES section in file README).  The following code is
   based directly on figure 4-8 in P&M. */

fn encode_and_append_mcu(
    out: &mut Vec<u8>,
    mcu: &[f32],
    qt: &[f32],
    huff_dc_len: &[u8],
    huff_dc_code: &[u16],
    huff_ac_len: &[u8],
    huff_ac_code: &[u16],
    pred: &mut i32,
    bitbuffer: &mut u32,
    location: &mut u32,
) {
    let mut du = [0i32; 64];
    let mut dct_mcu = [0f32; 64];
    dct_mcu.copy_from_slice(mcu);
    fdct(&mut dct_mcu);
    for i in 0..64 {
        let mut fval = dct_mcu[i];
        fval *= qt[i];
        fval = (fval + 1024.0 + 0.5).floor();
        fval -= 1024.0;
        let val = fval as i32;
        du[ZIG_ZAG[i]] = val;
    }

    // Encode DC coefficient.
    let diff = du[0] - *pred;
    *pred = du[0];
    if diff != 0 {
        let (bits, num_bits) = calculate_variable_length_int(diff);
        // Write number of bits with Huffman coding
        append_bits(
            out,
            bitbuffer,
            location,
            huff_dc_len[num_bits as usize] as u16,
            huff_dc_code[num_bits as usize],
        );
        append_bits(out, bitbuffer, location, num_bits, bits);
    } else {
        append_bits(
            out,
            bitbuffer,
            location,
            huff_dc_len[0] as u16,
            huff_dc_code[0],
        );
    }
    // ==== Encode AC coefficients ====

    let mut last_non_zero_i = 0;
    // Find the last non-zero element.
    for i in (0..64).rev() {
        if du[i] != 0 {
            last_non_zero_i = i;
            break;
        }
    }

    let mut i = 1;
    while i <= last_non_zero_i {
        // If zero, increase count. If >=15, encode (FF,00)
        let mut zero_count = 0;
        while du[i] == 0 {
            zero_count += 1;
            i += 1;
            if zero_count == 16 {
                // encode (ff,00) == 0xf0
                append_bits(
                    out,
                    bitbuffer,
                    location,
                    huff_ac_len[0xf0] as u16,
                    huff_ac_code[0xf0],
                );
                zero_count = 0;
            }
        }
        let (bits, num_bits) = calculate_variable_length_int(du[i]);

        debug_assert!(zero_count < 0x10);
        debug_assert!(num_bits <= 10);

        let sym1 = (((zero_count as u16) << 4) | num_bits) as usize;

        debug_assert!(huff_ac_len[sym1] != 0);

        // Write symbol 1  --- (RUNLENGTH, SIZE)
        append_bits(
            out,
            bitbuffer,
            location,
            huff_ac_len[sym1] as u16,
            huff_ac_code[sym1],
        );
        // Write symbol 2  --- (AMPLITUDE)
        append_bits(out, bitbuffer, location, num_bits, bits);
        i += 1;
    }

    if last_non_zero_i != 63 {
        // write EOB HUFF(00,00)
        append_bits(
            out,
            bitbuffer,
            location,
            huff_ac_len[0] as u16,
            huff_ac_code[0],
        );
    }
}

fn huff_expand(mem: &mut State) {
    // How many codes in total for each of LUMA_(DC|AC) and CHROMA_(DC|AC)
    let mut spec_tables_len = [0usize; 4];
    for i in 0..4 {
        for k in 0..16 {
            spec_tables_len[i] += mem.ht_bits[i][k] as usize;
        }
    }

    // Fill out the extended tables..
    let mut huffsize = [[0u8; 257]; 4];
    let mut huffcode = [[0u16; 256]; 4];
    for i in 0..4 {
        debug_assert!(256 >= spec_tables_len[i]);
        huff_get_code_lengths(&mut huffsize[i], mem.ht_bits[i]);
        huff_get_codes(&mut huffcode[i], &mut huffsize[i], spec_tables_len[i])
    }
    for i in 0..4 {
        huff_get_extended(
            &mut mem.ehuffsize[i],
            &mut mem.ehuffcode[i],
            &mem.ht_vals[i],
            &huffsize[i],
            &huffcode[i],
            spec_tables_len[i],
        );
    }
}

fn encode_main(mem: &State, w: i32, h: i32, num_components: i32, data: &[u8]) -> Vec<u8> {
    assert!(num_components == 3 || num_components == 4);
    assert!(w <= 0xffff && h <= 0xffff);

    let mut pqt_chroma = [0f32; 64];
    let mut pqt_luma = [0f32; 64];

    /* For float AA&N IDCT method, divisors are equal to quantization
       coefficients scaled by scalefactor[row]*scalefactor[col], where
         scalefactor[0] = 1
         scalefactor[k] = cos(k*PI/16) * sqrt(2)    for k=1..7
       We apply a further scale factor of 8.
       What's actually stored is 1/divisor so that the inner loop can
       use a multiplication rather than a division. */
    const AAN_SCALES: [f32; 8] = [
        1.0,
        1.387039845,
        1.306562965,
        1.175875602,
        1.0,
        0.785694958,
        0.541196100,
        0.275899379,
    ];

    // Build (de)quantization tables
    for y in 0..8 {
        for x in 0..8 {
            let i = y * 8 + x;
            let luma = mem.qt_luma[ZIG_ZAG[i]] as f32;
            let chroma = mem.qt_chroma[ZIG_ZAG[i]] as f32;
            pqt_luma[i] = 1.0 / (8.0 * AAN_SCALES[x] * AAN_SCALES[y] * luma);
            pqt_chroma[i] = 1.0 / (8.0 * AAN_SCALES[x] * AAN_SCALES[y] * chroma);
        }
    }

    let mut out: Vec<u8> = vec![];

    // Write header
    {
        // SOI
        out.write_u16::<BigEndian>(0xffd8).unwrap();
        // APP0
        out.write_u16::<BigEndian>(0xffe0).unwrap();
        // JFIF length
        out.write_u16::<BigEndian>(20 - 4).unwrap();
        // JFIF ID
        out.extend_from_slice(b"JFIF\0");
        // Version
        out.write_u16::<BigEndian>(0x0102).unwrap();
        // Dots-per-inch
        out.push(0x01);
        // X Density - 96 DPI
        out.write_u16::<BigEndian>(0x0060).unwrap();
        // Y Density - 96 DPI
        out.write_u16::<BigEndian>(0x0060).unwrap();
        // X thumb, Y thumb
        out.push(0);
        out.push(0);
    }

    // Write comment
    {
        let c = b"Created by Tiny JPEG Encoder";
        // Comment
        out.write_u16::<BigEndian>(0xfffe).unwrap();
        // Comment length
        let len = c.len() as u16 + 2;
        out.write_u16::<BigEndian>(len).unwrap();
        // Comment string
        out.extend_from_slice(c);
    }

    // Write quantization tables
    append_dqt(&mut out, &mem.qt_luma, 0);
    append_dqt(&mut out, &mem.qt_chroma, 1);

    // Write the frame marker
    {
        // SOF
        out.write_u16::<BigEndian>(0xffc0).unwrap();
        // Len
        out.write_u16::<BigEndian>(8 + 3 * 3).unwrap();
        // Precision
        out.push(8);
        // Height
        debug_assert!(h <= 0xffff);
        out.write_u16::<BigEndian>(h as u16).unwrap();
        // Width
        debug_assert!(w <= 0xffff);
        out.write_u16::<BigEndian>(w as u16).unwrap();
        // Number of components
        out.push(3);
        // Component spec
        let tables = [0, 1, 1];
        for i in 0..3 {
            out.push(i + 1); // No particular reason. Just 1, 2, 3.
            out.push(0x11);
            out.push(tables[i as usize]);
        }
    }

    // TODO: Use enums TJEI_LUMA/CHROMA_DC/AC and TJEI_DC/AC
    append_dht(&mut out, &mem.ht_bits[0], &mem.ht_vals[0], 0, 0);
    append_dht(&mut out, &mem.ht_bits[1], &mem.ht_vals[1], 1, 0);
    append_dht(&mut out, &mem.ht_bits[2], &mem.ht_vals[2], 0, 1);
    append_dht(&mut out, &mem.ht_bits[3], &mem.ht_vals[3], 1, 1);

    // Write start of scan
    {
        // SOS
        out.write_u16::<BigEndian>(0xffda).unwrap();
        // Length = 6 + (frame component spec * 3) = 12
        out.write_u16::<BigEndian>(12).unwrap();
        // Number of components
        out.push(3);

        let tables = [0x00, 0x11, 0x11];
        for i in 0..3 {
            /* Component ID -  Must be equal to component_id from frame header
               above. */
            out.push(i + 1);
            // dc_ac
            out.push(tables[i as usize]);

        }
        // First
        out.push(0);
        // Last
        out.push(63);
        // ah_al
        out.push(0);
    }

    /* Write compressed data
       --------------------- */

    let mut du_y = [0f32; 64];
    let mut du_b = [0f32; 64];
    let mut du_r = [0f32; 64];
    // Set diff to 0
    let mut pred_y = 0;
    let mut pred_b = 0;
    let mut pred_r = 0;
    // Bit stack
    let mut bitbuffer = 0u32;
    let mut location = 0u32;

    let mut y = 0;

    while y < h {
        let mut x = 0;
        while x < w {

            // Block loop: ====
            for off_y in 0..8 {
                for off_x in 0..8 {
                    let block_idx = off_y * 8 + off_x;
                    let mut src_idx = (((y + off_y) * w) + (x + off_x)) * num_components;
                    let col = x + off_x;
                    let row = y + off_y;

                    if row >= h {
                        src_idx -= (w * (row - h + 1)) * num_components;
                    }

                    if col >= w {
                        src_idx -= (col - w + 1) * num_components;
                    }
                    debug_assert!(src_idx < w * h * num_components);

                    let r = data[src_idx as usize + 0] as f32;
                    let g = data[src_idx as usize + 1] as f32;
                    let b = data[src_idx as usize + 2] as f32;

                    let luma: f32 = 0.299 * r + 0.587 * g + 0.114 * b - 128.0;
                    let cb: f32 = -0.1687 * r - 0.3313 * g + 0.5 * b;
                    let cr: f32 = 0.5 * r - 0.4187 * g - 0.0813 * b;

                    let block_idx = block_idx as usize;
                    du_y[block_idx] = luma;
                    du_b[block_idx] = cb;
                    du_r[block_idx] = cr;
                }
            }
            // ===============

            encode_and_append_mcu(
                &mut out,
                &du_y,
                &pqt_luma,
                &mem.ehuffsize[0],
                &mem.ehuffcode[0],
                &mem.ehuffsize[1],
                &mem.ehuffcode[1],
                &mut pred_y,
                &mut bitbuffer,
                &mut location,
            );
            encode_and_append_mcu(
                &mut out,
                &du_b,
                &pqt_chroma,
                &mem.ehuffsize[2],
                &mem.ehuffcode[2],
                &mem.ehuffsize[3],
                &mem.ehuffcode[3],
                &mut pred_b,
                &mut bitbuffer,
                &mut location,
            );
            encode_and_append_mcu(
                &mut out,
                &du_r,
                &pqt_chroma,
                &mem.ehuffsize[2],
                &mem.ehuffcode[2],
                &mem.ehuffsize[3],
                &mem.ehuffcode[3],
                &mut pred_r,
                &mut bitbuffer,
                &mut location,
            );

            x += 8;
        }
        y += 8;
    }

    /* Finish the image
       ---------------- */
    if location > 0 && location < 8 {
        let num_bits = (8 - location) as u16;
        append_bits(&mut out, &mut bitbuffer, &mut location, num_bits, 0);
    }
    // EOI
    out.write_u16::<BigEndian>(0xffd9).unwrap();

    out
}

pub enum Quality {
    Medium,
    High,
    Highest,
}

/// Takes bitmap data and writes a JPEG-encoded image to disk at the highest
/// quality.
pub fn encode_to_file(
    dest: &Path,
    w: i32,
    h: i32,
    num_components: i32,
    data: &[u8],
) -> Result<(), io::Error> {
    encode_to_file_at_quality(dest, Quality::Highest, w, h, num_components, data)
}

/// Takes bitmap data and writes a JPEG-encoded image to disk at the specified
/// quality.
pub fn encode_to_file_at_quality(
    dest: &Path,
    quality: Quality,
    w: i32,
    h: i32,
    num_components: i32,
    data: &[u8],
) -> Result<(), io::Error> {
    let mut f = File::create(dest)?;
    let encoded_bytes = encode_to_buffer(quality, w, h, num_components, data);
    f.write_all(&encoded_bytes)?;
    Ok(())
}

/// Returns a JPEG-encoded buffer, given bitmap data
pub fn encode_to_buffer(
    quality: Quality,
    w: i32,
    h: i32,
    num_components: i32,
    data: &[u8],
) -> Vec<u8> {

    let qt_factor: u8 = match quality {
        Quality::High => 10,
        _ => 1,
    };

    let mut mem = State {
        ehuffsize: [[0u8; 257]; 4],
        ehuffcode: [[0u16; 256]; 4],
        ht_bits: [
            &DEFAULT_HT_LUMA_DC_LEN,
            &DEFAULT_HT_LUMA_AC_LEN,
            &DEFAULT_HT_CHROMA_DC_LEN,
            &DEFAULT_HT_CHROMA_AC_LEN,
        ],
        ht_vals: [
            &DEFAULT_HT_LUMA_DC,
            &DEFAULT_HT_LUMA_AC,
            &DEFAULT_HT_CHROMA_DC,
            &DEFAULT_HT_CHROMA_AC,
        ],
        qt_luma: [1; QT_SIZE],
        qt_chroma: [1; QT_SIZE],
    };

    match quality {
        Quality::Highest => {}
        Quality::High | Quality::Medium => {
            for i in 0..QT_SIZE {
                if DEFAULT_QT_LUMA_FROM_SPEC[i] != 0 {
                    mem.qt_luma[i] = DEFAULT_QT_LUMA_FROM_SPEC[i] / qt_factor;
                }
                if DETAULT_QT_CHROMA_FROM_PAPER[i] != 0 {
                    mem.qt_chroma[i] = DETAULT_QT_CHROMA_FROM_PAPER[i] / qt_factor;
                }
            }
        }
    }

    huff_expand(&mut mem);
    encode_main(&mem, w, h, num_components, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn white_texture() {
        let dest = Path::new("./out.jpg");
        const W: i32 = 4000;
        const H: i32 = 2000;
        const C: i32 = 4;
        let data = vec![255u8; (W * H * C) as usize];
        assert!(encode_to_file(dest, W, H, C, &data).is_ok());
    }
}
