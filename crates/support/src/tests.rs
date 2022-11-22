use super::*;
use crate::{prepare, trans_map_voodoo};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

fn test_locale_dir() -> PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../foo/locales/foo-bar-baz")
}

#[test]
fn full_roundtrip() {
    let dir = test_locale_dir();
    prepare(&dir).unwrap();

    dbg!(super::locales_yaml_files_to_translation_map(&dir).unwrap());
}

#[test]
fn ser_de_roundtrip() {
    let tp2per_locale_translations = HashMap::new();
    let bytes = crate::serialize(tp2per_locale_translations.clone()).unwrap();
    let reconstructed = crate::deserialize(&bytes[..]).unwrap();
    assert_eq!(tp2per_locale_translations, reconstructed);
}

#[test]
fn yaml_parsing_works() {
    let yaml_content = r###"---
en:
  The table below describes some of those behaviours.: "w00t"
  "Use YAML for mapping localized text, and support mutiple YAML files merging.": ""
  a.very.nested.message: whatever
"###;
    let mut trans_map = HashMap::new();
    extract_yaml_content(yaml_content, &mut trans_map);
    dbg!(trans_map);
}

#[test]
fn trans_map_voodoo_works() {
    let mut json_intermediate = HashMap::new();
    json_intermediate.insert(
        "en".to_owned(),
        json!({
                "The table below describes some of those behaviours.": "w00t",
                "Use YAML for mapping localized text, and support mutiple YAML files merging.": "",
                "a.very.nested.message": "whatever",
        }),
    );
    let mut trans_map = dbg!(trans_map_voodoo(json_intermediate));
    assert_eq!(
        trans_map
            .entry("The table below describes some of those behaviours.".to_owned())
            .or_default()
            .get("en"),
        Some(&"w00t".to_owned())
    );
}
