use rust_i18n::format_t;

mod info;

rust_i18n::i18n!("locales");

pub fn f(key: &str) -> String {
    tormat_t!(key)
}
