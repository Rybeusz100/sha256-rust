use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// initialize array of round constants
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn process_chunk(chunk: &mut [u8], hash: &mut [u32]) {
    let mut w: [u32; 64] = [0; 64];

    // copy into first 16 32-bit words
    for (i, word) in w.iter_mut().enumerate().take(16) {
        let index = i * 4;
        *word = u32::from_be_bytes(chunk[index..index + 4].try_into().unwrap());
    }

    // extend the first 16 words into remaining 48 words
    for i in 16..64 {
        let s0 = (w[i - 15].rotate_right(7)) ^ (w[i - 15].rotate_right(18)) ^ (w[i - 15] >> 3);
        let s1 = (w[i - 2].rotate_right(17)) ^ (w[i - 2].rotate_right(19)) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }

    let mut a = hash[0];
    let mut b = hash[1];
    let mut c = hash[2];
    let mut d = hash[3];
    let mut e = hash[4];
    let mut f = hash[5];
    let mut g = hash[6];
    let mut h = hash[7];

    for i in 0..64 {
        let s1 = (e.rotate_right(6)) ^ (e.rotate_right(11)) ^ (e.rotate_right(25));
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let s0 = (a.rotate_right(2)) ^ (a.rotate_right(13)) ^ (a.rotate_right(22));
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    hash[0] = hash[0].wrapping_add(a);
    hash[1] = hash[1].wrapping_add(b);
    hash[2] = hash[2].wrapping_add(c);
    hash[3] = hash[3].wrapping_add(d);
    hash[4] = hash[4].wrapping_add(e);
    hash[5] = hash[5].wrapping_add(f);
    hash[6] = hash[6].wrapping_add(g);
    hash[7] = hash[7].wrapping_add(h);
}

pub fn hash_file(path: &Path, buf_size_kb: usize) -> Result<String, std::io::Error> {
    // initialize hash values
    let mut hash: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    // open file and check file size
    let mut file = File::open(path)?;
    let metadata = path.metadata()?;
    let file_size_bytes: u64 = metadata.len();
    let file_size_bits: u64 = file_size_bytes * 8;
    let buf_size_bytes = buf_size_kb * 1024;

    // allocate main buffer for reading files
    let mut main_buffer: Vec<u8> = vec![0; buf_size_bytes];

    // calculate how many times will the main buffer be filled
    let buf_reads = file_size_bytes / buf_size_bytes as u64 + 1;
    let chunks_in_buf = buf_size_bytes / 64;

    // process buf_reads - 1 parts of file
    for _buf_read in 0..buf_reads - 1 {
        // read into buffer
        file.read_exact(&mut main_buffer)?;

        // process 512-bit chunks
        let mut chunk_start = 0;
        for _ in 0..chunks_in_buf {
            let chunk = &mut main_buffer[chunk_start..chunk_start + 64];
            chunk_start += 64;
            process_chunk(chunk, &mut hash);
        }
    }

    // process data without last 512-bit chunk
    let last_chunk_start =
        file_size_bytes - ((buf_reads - 1) * buf_size_bytes as u64) - (file_size_bytes % 64);
    file.read_exact(&mut main_buffer[0..last_chunk_start as usize])?;
    // process 512-bit chunks
    let mut chunk_start = 0;
    let chunks_in_buf = last_chunk_start / 64;
    for _ in 0..chunks_in_buf {
        let chunk = &mut main_buffer[chunk_start..chunk_start + 64];
        chunk_start += 64;
        process_chunk(chunk, &mut hash);
    }

    // calculate padding
    let zeros_to_append = 512 - (file_size_bits + 1 + 64) % 512;
    let bytes_to_append = ((zeros_to_append - 7) / 8) as usize;
    let file_size_byte_array: [u8; 8] = u64::to_be_bytes(file_size_bits);

    // apply padding
    let data_usize = file.read(&mut main_buffer)?;
    let mut index = data_usize;

    main_buffer[index] = 0b10000000;
    index += 1;

    for _ in 0..bytes_to_append {
        main_buffer[index] = 0;
        index += 1;
    }

    for byte in file_size_byte_array {
        main_buffer[index] = byte;
        index += 1;
    }

    // check if a new chunk has been made
    let chunks_in_buf = if data_usize + 1 + bytes_to_append + 8 > 64 {
        2
    } else {
        1
    };

    let mut chunk_start = 0;
    for _ in 0..chunks_in_buf {
        let chunk = &mut main_buffer[chunk_start..chunk_start + 64];
        chunk_start += 64;
        process_chunk(chunk, &mut hash);
    }

    // return hash
    let hash_string = format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]
    );
    Ok(hash_string)
}
