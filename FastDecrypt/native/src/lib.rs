use crc::Crc;

/// # Safety
///
/// We have to trust the caller to supply valid ptr/sz pairs to the function.
#[no_mangle]
pub unsafe extern "C" fn decrypt(
    guid_ptr: *const u8,
    guid_len: usize,
    data_ptr: *const u8,
    data_len: usize,
    key_ptr: *const u8,
    key_len: usize,
    dst_ptr: *mut u8,
) {
    let guid = std::slice::from_raw_parts(guid_ptr, guid_len);
    let data = std::slice::from_raw_parts(data_ptr, data_len);
    let key = std::slice::from_raw_parts(key_ptr, key_len);
    let dst = std::slice::from_raw_parts_mut(dst_ptr, data_len + key_len);

    decrypt_internal(&guid, data, key, dst);
}

const X32: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

fn compute_crc(data: &[u8]) -> u32 {
    X32.checksum(data)
}

struct CVRRand {
    state: i64,
    crc: i64,
    frag_size: i64,
}

impl CVRRand {
    pub fn new(crc: u32, size: usize) -> Self {
        Self {
            state: 0x3fffffefffffff_i64,
            crc: crc.into(),
            frag_size: u32::max((size / 100) as u32, 1000).into(),
        }
    }
    pub fn next(&mut self) -> usize {
        self.state = self.state.wrapping_mul(self.crc).wrapping_add(self.state) % self.frag_size
            + self.frag_size;
        self.state as usize
    }
}

/// Steps:
/// - Seed PRNG with CRC32 of the GUID and data length
/// - Segment the data in chunks of random length
/// - Scramble segments, skipping the first one (UnityFS header)
/// - Reassemble the data with these segments
pub fn decrypt_internal(guid: &[u8], bytes: &[u8], key_frag: &[u8], dst: &mut [u8]) {
    let total_size = bytes.len() + key_frag.len();

    // Seed PRNG
    let mut random = CVRRand::new(compute_crc(guid), total_size);

    // Segment data
    let mut segments: [MaybeUninit<Range<usize>>; 100] =
        unsafe { MaybeUninit::uninit().assume_init() };
    let mut i = 0;
    let mut offset = 0;
    while offset < total_size {
        let len = random.next();
        let end = (offset + len).min(total_size);
        segments[i] = MaybeUninit::new(offset..end);
        i += 1;
        offset = end;
    }
    let segments: &mut [Range<usize>; 100] = &mut unsafe { mem::transmute(segments) };
    let segments = &mut segments[..i];

    // Scramble
    let length = segments.len();
    for i in 1..length {
        let index = random.next() % (length - 1) + 1;
        segments.swap(i, index);
    }

    // Reassemble
    let mut offset = 0;
    for segment in segments.iter() {
        let length = segment.len();
        match (offset > bytes.len(), offset + length > bytes.len()) {
            (false, false) => {
                dst[segment.start..segment.end].copy_from_slice(&bytes[offset..offset + length]);
            }
            (false, true) => {
                let remainder = bytes.len() - offset;
                let temp = segment.start + remainder;
                dst[segment.start..temp].copy_from_slice(&bytes[offset..]);
                dst[temp..segment.end].copy_from_slice(&key_frag[..length - remainder]);
            }
            (true, _) => {
                dst[segment.start..segment.end].copy_from_slice(
                    &key_frag[offset - bytes.len()..offset + length - bytes.len()],
                );
            }
        }
        offset += length;
    }
}

#[test]
fn test_compute_crc() {
    assert_eq!(
        compute_crc(b"2c99f767-53b9-463c-aa99-791b04cd9003"),
        510747253
    );
    assert_eq!(
        compute_crc(b"8611ee9e-0c57-48d2-af32-7f980b0895db"),
        395872363
    );
    assert_eq!(
        compute_crc(b"6586c486-4731-4fae-a2d2-de415cd8bcd6"),
        4284363129
    );
    assert_eq!(
        compute_crc(b"6b86cced-e17c-4f57-8bdf-812615773ce6"),
        2976536806
    );
    assert_eq!(
        compute_crc(b"32ceb35d-24fa-469f-8aa4-23851ac68f84"),
        693157058
    );
    assert_eq!(
        compute_crc(b"17c267db-18c4-4900-bb73-ad323f082640"),
        2690702873
    );
    assert_eq!(
        compute_crc(b"5dc14ba2-7164-40e9-8b52-f55cf3129a24"),
        4047935656
    );
    assert_eq!(
        compute_crc(b"67e08c5c-d918-478e-ad8d-58e884fa53b4"),
        4085790074
    );
    assert_eq!(
        compute_crc(b"9d1d8585-9c0b-40d9-8721-76f21cc745f2"),
        2255572015
    );
}

#[test]
fn test_random() {
    let pairs: &[(u32, usize, &[usize])] = &[
        (
            510747253,
            2498515,
            &[
                1856, 38009, 40561, 33484, 29916, 38479, 42136, 29459, 37426, 49824, 32576, 26129,
                45936, 47509,
            ],
        ),
        (
            395872363,
            2786283,
            &[
                41700, 48260, 48970, 51128, 32886, 31124, 46096, 40666, 33580, 55366, 41372, 55190,
                33538, 36716,
            ],
        ),
        (
            4284363129,
            2759924,
            &[
                32766, 35213, 50837, 52719, 52656, 28388, 34509, 35052, 28684, 37696, 44844, 55046,
                48340, 47186,
            ],
        ),
        (
            2976536806,
            6347242,
            &[
                66489, 95311, 106793, 85663, 116057, 72031, 119673, 97295, 97625, 116255, 83209,
                87151, 125049, 94975,
            ],
        ),
        (
            693157058,
            690036,
            &[
                9717, 11403, 7977, 11943, 9237, 7983, 13797, 9423, 10257, 12663, 10917, 9603,
                10677, 11343,
            ],
        ),
        (
            2690702873,
            1227329,
            &[
                13213, 17016, 13056, 19971, 13800, 17928, 13323, 20574, 19023, 13038, 13587, 24204,
                13707, 13581,
            ],
        ),
        (
            4047935656,
            2974248,
            &[
                44043, 49539, 53073, 40377, 47409, 55827, 56337, 58191, 56883, 55977, 39387, 40311,
                54867, 45849,
            ],
        ),
        (
            4085790074,
            6442418,
            &[
                85765, 97567, 93445, 72351, 81749, 69951, 89629, 113871, 106485, 83223, 100813,
                122247, 82205, 103887,
            ],
        ),
        (
            2255572015,
            18981052,
            &[
                266060, 205540, 304850, 205900, 379280, 362050, 348140, 282490, 222770, 318760,
                271550, 249190, 266060, 205540,
            ],
        ),
    ];
    for &(crc, size, want) in pairs {
        let mut random = CVRRand::new(crc, size);
        let got = (0..want.len()).map(|_| random.next()).collect::<Vec<_>>();
        assert_eq!(&got, want);
    }
}

#[test]
fn test_decrypt() {
    use std::collections::hash_map::DefaultHasher;
    use std::fs;
    use std::hash::{Hash, Hasher};
    use std::time::Instant;

    let guids = &[
        "2c99f767-53b9-463c-aa99-791b04cd9003",
        "8611ee9e-0c57-48d2-af32-7f980b0895db",
        "6586c486-4731-4fae-a2d2-de415cd8bcd6",
        "6b86cced-e17c-4f57-8bdf-812615773ce6",
        "32ceb35d-24fa-469f-8aa4-23851ac68f84",
        "17c267db-18c4-4900-bb73-ad323f082640",
        "5dc14ba2-7164-40e9-8b52-f55cf3129a24",
        "67e08c5c-d918-478e-ad8d-58e884fa53b4",
        "9d1d8585-9c0b-40d9-8721-76f21cc745f2",
    ];
    for guid in guids {
        let enc = fs::read(format!("tests/{guid}.enc")).unwrap();
        let key = fs::read(format!("tests/{guid}.key")).unwrap();
        let mut dec = Vec::<u8>::new();
        dec.resize(enc.len() + key.len(), 0x42);
        let now = Instant::now();
        decrypt_internal(guid.as_bytes(), &enc, &key, &mut dec);
        let elapsed = now.elapsed();
        println!(
            "guid: {guid}, elapsed: {elapsed:?}, {} MiB/s",
            dec.len() as f32 / 1024.0 / 1024.0 / elapsed.as_secs_f32()
        );

        let mut hasher = DefaultHasher::new();
        dec.hash(&mut hasher);
        let got = hasher.finish();

        let mut hasher = DefaultHasher::new();
        let want = fs::read(format!("tests/{guid}.dec")).unwrap();
        want.hash(&mut hasher);
        let expected = hasher.finish();

        assert_eq!(got, expected);
    }
}
