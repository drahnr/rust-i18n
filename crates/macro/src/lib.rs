use quote::quote;
use rust_i18n_support::load_locales;
use syn::{punctuated::Punctuated, token::Token, Expr};
use std::collections::HashMap;

// Avoid repeated path reads
const TRANSLATIONS: &[u8] = include_bytes!("foo-bar-baz");

enum FormatArg {
    Ident(Ident),
    Expr(Expr),
    IdentEqExpr{ alias: Ident, eq: Token![=], value: Expr },
}

struct FormatArgs {
    fmt_str: syn::Literal,
    maybe_comma: Option<Token![,]>,
    maybe_args: Punctuated<FormatArg, Token![,]>,
}

fn format_inner(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let support = support_crate_path();
    let FormatArgs { fmt_str, maybe_comma: _, maybe_args } = syn::parse2(input)?;

    // must be (orig_text -> (language -> translation_text)* )*
    let translations = rust_i18n_support::deserialize(TRANSLATIONS).expect("You must have a build.rs that does the fixins");
    
    // Will cause quite a bit of load during compilation for applications with many
    // invocations
    let translations: HashMap<(String, String), String> = translations.get(args.fmt_str).ok_or_else(|| syn::Error::new(Span::call_site(), "No translation for string")) {

    let language = translations.keys().map(|(orig: _, locale)| locale);
    let translation = translations.value();
    Ok(quote!(
        match #support::locale() {
           #(
                #language => ::std::format!(#translation, #maybe_args,),
            )*
        }
    ))
}


fn format_inner(input: proc_macro2::TokenStream, ) -> syn::Result<proc_macro2::TokenStream> {
    let support = support_crate_path();
    let FormatArgs { fmt_str, maybe_comma: _, maybe_args } = syn::parse2(input)?;

    // must be (orig_text -> (language -> translation_text)* )*
    let orig2translations = deserialize(TRANSLATIONS);
    
    // Will cause quite a bit of load during compilation for applications with many
    // invocations
    let translations: HashMap<&'static str, &'static str> = translations.get(args.fmt_str).ok_or_else(|| syn::Error::new(Span::call_site(), "No translation for string")) {

    let language = translations.keys();
    let translation = translations.value();
    Ok(quote!(
        match #support::locale() {
           #(#language => ::std::eprintln!(#translation, #maybe_args,)),*
        }
    ))
}

#[proc_macro]
pub fn format_t(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    format_inner(proc_macro2::TokenStream::from(input)).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn support_crate_path() -> syn::Path {
    use proc_macro_crate as pmc;
    let found_crate = pmc::crate_name("rust-i18n").expect("rust-i18n must be present in `Cargo.toml`, but it's not");

    syn::Path::from(match found_crate {
        pmc::FoundCrate::Itself => quote!( crate ),
        pmc::FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident )
        }
    })
}
