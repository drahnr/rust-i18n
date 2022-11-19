fn main() -> Result<(), Box<dyn std::error::Error>> {
    rust_i18n_support::prepare("locales")?;
    Ok(())
}
