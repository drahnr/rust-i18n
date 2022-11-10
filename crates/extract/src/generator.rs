use crate::extractor::Message;
use rust_i18n_support::{load_locales, Translations};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;

pub fn generate<'a, P: AsRef<Path>>(
    output: P,
    locale: &str,
    messages: impl IntoIterator<Item = &'a Message>,
) -> Result<()> {
    println!("Checking [{}] and generating untranslated texts...", locale);

    // TODO.en.yml
    let filename = format!("TODO.{}.yml", locale);
    // ~/work/my-project/locales
    let output_path = output.as_ref().to_owned();

    let ignore_file = |fname: &str| fname.ends_with(&filename);
    let old_translations = load_locales(&output_path, ignore_file);

    let mut new_translations: Translations = HashMap::new();
    let mut new_values: HashMap<String, String> = HashMap::new();

    for m in messages {
        let key = format!("{}.{}", locale, m.key);

        if !m.locations.is_empty() {
            for _l in &m.locations {
                // TODO: write file and line as YAML comment
            }
        }

        if old_translations.get(&key).is_some() {
            continue;
        }

        let value = m.key.split(".").last().unwrap_or_default();

        new_values
            .entry(m.key.clone())
            .or_insert_with(|| value.into());
    }

    new_translations.insert(locale.to_string(), serde_json::to_value(&new_values)?);
    write_file(&output, &filename, &new_translations)?;

    if new_values.is_empty() {
        println!("All thing done.\n");

        return Ok(());
    }

    eprintln!("Found {} new texts need to translate.", new_values.len());
    eprintln!("----------------------------------------");
    eprintln!("Writing to {}\n", filename);

    write_file(&output, &filename, &new_translations)?;

    // Finally, return error for let CI fail
    let err = std::io::Error::new(std::io::ErrorKind::Other, "");
    Err(err)
}

fn write_file<'a, P: AsRef<Path>>(
    output: &P,
    filename: &str,
    translations: &Translations,
) -> Result<()> {
    let output_file = std::path::Path::new(output.as_ref()).join(String::from(filename));
    let mut output = ::std::fs::File::create(&output_file)
        .unwrap_or_else(|_| panic!("Unable to create {} file", &output_file.display()));

    writeln!(output, "{}", serde_yaml::to_string(&translations).unwrap())
        .expect("Write YAML file error");

    Ok(())
}
