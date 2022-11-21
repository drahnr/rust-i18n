use fs::File;
use fs_err as fs;
use glob::glob;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Write;

pub type Locale = String;
pub type Value = serde_json::Value;
pub type Translations = HashMap<Locale, Value>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Failed to serialize / deserialize")]
    SerDe,
    #[error(transparent)]
    Postcard(#[from] postcard::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Init I18n translations from `build.rs`.
///
/// This will load all translations by glob `**/*.yml` from the
/// given path and prepare a file to be included in the compiled proc macro.
pub fn load_from_dirs(locale_dir: impl AsRef<std::path::Path>) -> Result<()> {
    let locale_path = locale_dir.as_ref();
    let translations = locales_yaml_files_to_translation_map(locale_path)?;
    let translations = serialize(translations)?;

    fs::write("foo-bar-baz", translations)?;

    Ok(())
}

/// Path to an translation item
///
/// f the for `a.b.c` analogous to `json` addressing.
pub type TranslationPath = String;

/// Optimize for proc-macro parsing ease,
/// that's called 1 vs n times more often!
pub type TranslationMap = HashMap<TranslationPath, HashMap<Locale, String>>;

pub fn deserialize(bytes: &[u8]) -> Result<TranslationMap> {
    let tmap: TranslationMap = postcard::from_bytes(bytes)?;
    Ok(tmap)
}

pub fn serialize(text2translations: TranslationMap) -> Result<Vec<u8>> {
    let bytes = postcard::to_allocvec(&text2translations)?;
    Ok(bytes)
}

/// Merge JSON Values, merge b into a
///
/// Overrides values of `a` with values of `b`
/// and recurses into all objects.
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

// Load locales into flatten key,value HashMap
pub fn locales_yaml_files_to_translation_map(locales_dir: &std::path::Path) -> Result<TranslationMap> {
    let mut trans_map = Translations::new();

    let path_pattern = format!("{}/**/*.yml", locales_dir.display());

    println!("cargo:i18n-locale={}", &path_pattern);

    let paths = glob(&path_pattern).expect("Failed to read glob pattern");
    for maybe_path in paths {
        let path = if let Ok(path) = maybe_path {
            path
        } else {
            continue;
        };
        println!("cargo:i18n-load={}", &path.display());

        let file = File::open(path).expect("Failed to open the YAML file");
        let mut reader = std::io::BufReader::new(file);
        let mut content = String::new();

        reader
            .read_to_string(&mut content)
            .expect("Read YAML file failed.");

        // All translation items per language
        let trs: Translations =
            serde_yaml::from_str(&content).expect("Invalid YAML format, parse error");

        eprintln!("cargo:warning: foo: -- {:?}", &trs);

        trs.into_iter().for_each(|(tp, translations)| {
            trans_map
                .entry(tp)
                .and_modify(|translations_old| merge_value(translations_old, &translations))
                .or_insert(translations);
        });
    }

    let mut tp2trans_per_locale = TranslationMap::new();
    trans_map.iter().for_each(|(locale, trs)| {
        let new_vars = extract_vars(locale.as_str(), &trs);
        let new_vars_iter = new_vars.into_iter().filter_map(|(k,v)| {
            k.strip_prefix(&(locale.to_owned() + ".")).map(move |k| (k.to_string(),v))
        });
        for (tp, translation) in new_vars_iter {
            tp2trans_per_locale
                .entry(tp)
                .or_default()
                .insert(locale.to_owned(), translation);
        }
    });

    Ok(tp2trans_per_locale)
}

/// Find the value based on it's path aka prefix `a.b.c`
/// 
/// Returns a `prefix`:`value` set.
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

/// Init I18n translations.
///
/// This will load all translations by glob `**/*.yml` from the given path and prepare a file to be included in the compiled proc macro.
pub fn prepare(locale_dir: impl AsRef<std::path::Path>) -> Result<()> {
    let locales_dir = locale_dir.as_ref();

    let translations = locales_yaml_files_to_translation_map(&locales_dir)?;

    let serialized = self::serialize(translations)?;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(locales_dir.join("foo-bar-baz"))?;
    f.write_all(serialized.as_slice())?;
    Ok(())
}

#[cfg(test)]
mod tests;
