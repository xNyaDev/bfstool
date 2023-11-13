#[test]
#[cfg(feature = "extra_tests")]
fn test_bzf2001() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    use std::io::{BufReader, BufWriter, Cursor, Read};

    use bfstool::keys::Keys;

    use pretty_assertions::assert_eq;

    let mut file = File::open("extra_test_data/Keys.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let keys = toml::from_str::<Keys>(&contents)?;
    let key = keys.bzf2001.expect("Missing decryption key").key;

    let input = File::open("extra_test_data/bzf2001/language.bzf")?;
    let input = BufReader::new(input);

    let decrypted_data = Vec::new();
    let mut decrypted_data = BufWriter::new(Cursor::new(decrypted_data));

    bfstool::crypt::bzf2001::decrypt(input, &mut decrypted_data, key)?;

    let expected_hash = blake3::Hash::from_hex(
        b"d2b05a207f24ba1ac69ea1a786135f771d674d6192f26f16fcce723c2af1b5c8",
    )?;

    decrypted_data.flush()?;
    let decrypted_data = decrypted_data.into_inner()?;

    let hash = blake3::hash(decrypted_data.get_ref().as_slice());

    assert_eq!(expected_hash, hash);

    let encrypted_data = Vec::new();
    let mut encrypted_data = BufWriter::new(Cursor::new(encrypted_data));

    bfstool::crypt::bzf2001::encrypt(decrypted_data, &mut encrypted_data, key)?;

    let mut original_data = Vec::new();
    let mut input = File::open("extra_test_data/bzf2001/language.bzf")?;
    input.read_to_end(&mut original_data)?;

    encrypted_data.flush()?;
    let encrypted_data = encrypted_data.into_inner()?;

    assert_eq!(original_data, encrypted_data.into_inner());

    Ok(())
}
