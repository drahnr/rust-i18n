use rust_i18n_support::{is_debug, load_locales};
use std::collections::HashMap;

/// Init I18n translations.
///
/// This will load all translations by glob `**/*.yml` from the given path and prepare a file to be included in the compiled proc macro.
pub fn load(locale_dir: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
    let locales_path = locale_dir.as_ref();

    let translations = load_locales(&locales_path, |_| false);

    let serialized = serde::serialize(translations).unwrap();
    std::fs::write_all("foo-bar-baz", serialized,)?;
    
    Ok(())
}
