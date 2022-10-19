use glob::glob;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

pub type Locale = String;
pub type Value = serde_json::Value;
pub type Translations = HashMap<Locale, Value>;

/// Init I18n translations from `build.rs`.
///
/// This will load all translations by glob `**/*.yml` from the
/// given path and prepare a file to be included in the compiled proc macro.
pub fn load_from_dirs(locale_path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
    let locales_path = current_dir.join(locale_path);

    let translations = load_locales(&locales_path.display().to_string(), |_| false);
    let translations = serialize(translations)?;
    
    std::fs::write("foo-bar-baz", translations)?;
    
    Ok(())
}

/// Optimize for proc-macro parsing ease, that's called 1 vs n times more often!
pub type TranslationMap = HashMap<String, HashMap<Locale, String>>;
 
pub fn deserialize(bytes: &[u8]) -> Result<TranslationMap, ()> {
    todo!()
}


pub fn serialize(text2translations: TranslationMap) -> Result<Vec<u8>,()> {
    todo!()
}

/// Merge JSON Values, merge b into a
pub fn merge_value(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_value(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

// Load locales into flatten key, value HashMap
pub fn load_locales<F: Fn(&str) -> bool>(
    locales_path: &std::path::Path,
    ignore_if: F,
) -> TranslationMap {
    let mut trans_map: Translations = HashMap::new();

    let path_pattern = format!("{}/**/*.yml", locales_path.display());

    println!("cargo:i18n-locale={}", &path_pattern);

    for entry in glob(&path_pattern).expect("Failed to read glob pattern") {
        let entry = entry.unwrap();
        println!("cargo:i18n-load={}", &entry.display());

        if ignore_if(&entry.display().to_string()) {
            continue;
        }

        let file = File::open(entry).expect("Failed to open the YAML file");
        let mut reader = std::io::BufReader::new(file);
        let mut content = String::new();

        reader
            .read_to_string(&mut content)
            .expect("Read YAML file failed.");

        let trs: Translations =
            serde_yaml::from_str(&content).expect("Invalid YAML format, parse error");

        trs.into_iter().for_each(|(locale, translations)| {
            trans_map
                .entry(locale)
                .and_modify(|translations_old| merge_value(translations_old, &translations))
                .or_insert(new_value);
        });
    }

    let mut locale_vars = HashMap::new();
    trans_map.iter().for_each(|(locale, trs)| {
        let new_vars = extract_vars(locale.as_str(), &trs);
        locale_vars.entry(orig_text).or_default().and_modify(|x| { x.extend(new_vars); });;
    });

    locale_vars
}

pub fn extract_vars(prefix: &str, trs: &Value) -> HashMap<String, String> {
    let mut v = HashMap::<String, String>::new();
    let prefix = prefix.to_string();

    match &trs {
        serde_json::Value::String(s) => {
            v.insert(prefix, s.to_string());
        }
        serde_json::Value::Object(o) => {
            for (k, vv) in o {
                let key = format!("{}.{}", prefix, k);
                v.extend(extract_vars(key.as_str(), vv));
            }
        }
        serde_json::Value::Null => {
            v.insert(prefix, "".into());
        }
        serde_json::Value::Bool(s) => {
            v.insert(prefix, format!("{}", s));
        }
        serde_json::Value::Number(s) => {
            v.insert(prefix, format!("{}", s));
        }
        serde_json::Value::Array(_) => {
            v.insert(prefix, "".into());
        }
    }

    v
}
