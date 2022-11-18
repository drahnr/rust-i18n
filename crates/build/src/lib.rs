use rust_i18n_support::load_locales;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

/// Init I18n translations.
///
/// This will load all translations by glob `**/*.yml` from the given path and prepare a file to be included in the compiled proc macro.
pub fn prepare(locale_dir: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
    let locales_path = locale_dir.as_ref();

    let translations = load_locales(&locales_path, |_| false);

    let serialized = rust_i18n_support::serialize(translations).unwrap();
    let mut f = fs::OpenOptions::new().create(true).write(true).truncate(true).open(locales_path.join("foo-bar-baz"))?;
    f.write_all(serialized.as_slice())?;
    Ok(())
}
