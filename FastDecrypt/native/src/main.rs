fn main() {
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
        libdec::decrypt_internal(guid.as_bytes(), &enc, &key, &mut dec);
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
