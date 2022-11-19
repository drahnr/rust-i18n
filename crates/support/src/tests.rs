use crate::prepare;
use std::collections::HashMap;
use std::path::PathBuf;

fn test_locale_dir() -> PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../foo/locales/foo-bar-baz")
}

#[test]
fn full_roundtrip() {
    let dir = test_locale_dir();
    prepare(&dir).unwrap();

    dbg!(super::load_locales(&dir).unwrap());
}

#[test]
fn ser_de_roundtrip() {
    let tp2per_locale_translations = HashMap::new();
    let bytes = crate::serialize(tp2per_locale_translations.clone()).unwrap();
    let reconstructed = crate::deserialize(&bytes[..]).unwrap();
    assert_eq!(tp2per_locale_translations, reconstructed);
}
