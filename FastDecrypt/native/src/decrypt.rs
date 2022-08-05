use crc::Crc;

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
    #[inline(always)]
    fn next(&mut self) -> usize {
        unsafe {
            self.state = self
                .state
                .wrapping_mul(self.crc)
                .wrapping_add(self.state)
                .checked_rem(self.frag_size)
                .unwrap_unchecked()
                .wrapping_add(self.frag_size);
            core::mem::transmute(self.state)
        }
    }
}

/// Steps:
/// - Seed PRNG with CRC32 of the GUID and clamp it between 1/100 and 2/100 of the data size
/// - Segment the data in, at most, 100 chunks of random length
/// - Scramble segments, skipping the first one (UnityFS header)
/// - Reassemble the data with these segments
pub fn decrypt_internal(guid: &[u8], bytes: &[u8], key_frag: &[u8], dst: &mut [u8]) {
    let total_size = bytes.len() + key_frag.len();

    // Seed PRNG
    let mut random = CVRRand::new(compute_crc(guid), total_size);

    // Segment data
    #[derive(Clone, Copy)]
    struct Segment {
        offset: usize,
        end: usize,
    }
    let mut segments: [Segment; 100] = [Segment { offset: 0, end: 0 }; 100];
    let mut i = 0;
    let mut offset = 0;
    while offset < total_size {
        let len = random.next();
        let end = (offset + len).min(total_size);
        unsafe { *segments.get_unchecked_mut(i) = Segment { offset, end } };
        i += 1;
        offset = end;
    }
    let segments = unsafe { segments.get_unchecked_mut(..i) };

    // Scramble
    let length = segments.len();
    for i in 1..length {
        let index = unsafe {
            random
                .next()
                .checked_rem(length.wrapping_sub(1))
                .unwrap_unchecked()
                .wrapping_add(1)
        };
        unsafe {
            let tmp = *segments.get_unchecked(index);
            *segments.get_unchecked_mut(index) = *segments.get_unchecked(i);
            *segments.get_unchecked_mut(i) = tmp
        };
    }

    // Reassemble
    let mut offset = 0;
    for segment in segments.iter() {
        let length = segment.end - segment.offset;
        match (offset > bytes.len(), offset + length > bytes.len()) {
            (false, false) => unsafe {
                core::ptr::copy_nonoverlapping(
                    bytes.get_unchecked(offset..offset + length).as_ptr(),
                    dst.get_unchecked_mut(segment.offset..segment.end)
                        .as_mut_ptr(),
                    length,
                );
            },
            (false, true) => {
                let remainder = bytes.len() - offset;
                let temp = segment.offset + remainder;
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        bytes.get_unchecked(offset..).as_ptr(),
                        dst.get_unchecked_mut(segment.offset..temp).as_mut_ptr(),
                        remainder,
                    );
                    core::ptr::copy_nonoverlapping(
                        key_frag.get_unchecked(..length - remainder).as_ptr(),
                        dst.get_unchecked_mut(temp..segment.end).as_mut_ptr(),
                        length - remainder,
                    );
                }
            }
            (true, _) => unsafe {
                let src =
                    key_frag.get_unchecked(offset - bytes.len()..offset + length - bytes.len());
                let dst = dst.get_unchecked_mut(segment.offset..segment.end);
                core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), length);
            },
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
        want.iter().for_each(|&want| {
            assert_eq!(want, random.next());
        });
    }
}
