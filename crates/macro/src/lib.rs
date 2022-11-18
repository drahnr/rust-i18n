//! Parse the content of `format_t!` arguments
//! 
//! Required, to filter out `a.b.c` style paths
//! and avoid erroring for no good reason.

use quote::{quote, ToTokens};
use rust_i18n_support::load_locales;
use syn::{punctuated::Punctuated, Expr, parse::Parse};
use std::collections::HashMap;
use proc_macro2::Ident;
use proc_macro2::Span;
use syn::punctuated::*;
use syn::Token;

enum FormatArg {
    Ident(Ident),
    Expr(syn::Expr),
    IdentEqExpr{ alias: Ident, eq: Token![=], value: Expr },
}

impl Parse for FormatArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            let alias = input.parse::<syn::Ident>()?;
            if lookahead.peek(Token![=]) {
                let eq = input.parse::<Token![=]>()?;
                let value = input.parse::<syn::Expr>()?;
                Ok(Self::IdentEqExpr{ alias, eq, value })
            } else {
                Ok(Self::Ident(alias))
            }
        } else {
            input.parse::<syn::Expr>().map(Self::Expr)
        }
    }
}

impl ToTokens for FormatArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(ident) => tokens.extend(ident.to_token_stream()),
            Self::Expr(expr) => tokens.extend(expr.to_token_stream()),
            Self::IdentEqExpr{ alias, eq: _, value} => tokens.extend(quote!{ #alias = #value }),
        }
    }
}

struct FormatArgs {
    fmt_str: syn::LitStr,
    maybe_comma: Option<Token![,]>,
    maybe_args: Punctuated<FormatArg, Token![,]>,
}

use syn::Lit;

impl Parse for FormatArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<syn::Lit>()?;
        let fmt_str = match lit {
            syn::Lit::Str(alias) => alias,
            other => return Err(syn::Error::new(other.span(), "Expected a literal str for format arg but found..."))
        };
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![,]) {
            let maybe_comma = Some(input.parse::<Token![,]>()?);
            let maybe_args = Punctuated::<FormatArg, Token![,]>::parse_terminated(&input)?;
            Ok(Self{ fmt_str, maybe_comma, maybe_args })
        } else {
            Ok(Self { fmt_str, maybe_comma: None, maybe_args: Punctuated::new()})
        }
    }
}

impl ToTokens for FormatArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let FormatArgs {
            fmt_str : ref fmt_str,
            maybe_comma : ref maybe_comma,
            maybe_args : ref maybe_args,
        } = self;
        tokens.extend(fmt_str.to_token_stream());
        if !maybe_args.is_empty() {
          tokens.extend(maybe_args.to_token_stream());  
        }
    }
}

fn format_inner(input: proc_macro2::TokenStream, ) -> syn::Result<proc_macro2::TokenStream> {
    let support = support_crate_path();
    let FormatArgs { fmt_str, maybe_comma: _, maybe_args } = syn::parse2(input)?;

    // must be (a.b.c -> (language -> translation_text)* )*

    let orig2translations = 
        rust_i18n_support::load_locales(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("locales"),
            |_| false,
        );
    
    // Will cause quite a bit of load during compilation for applications with many
    // invocations, but whatever...
    let translations: &HashMap<String, String> = orig2translations.get(fmt_str.value().as_str()).ok_or_else(|| syn::Error::new(Span::call_site(), "No translation for string"))?;

    let language = translations.keys();
    let translation = translations.values();
    let ts = quote!(
        match #support::locale() {
            #( #language => ::std::format!( #translation, #maybe_args,)),*
        }
    );
    Ok(ts)
}

#[proc_macro]
pub fn format_t(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    format_inner(proc_macro2::TokenStream::from(input)).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn support_crate_path() -> syn::Path {
    use proc_macro_crate as pmc;
    let found_crate = pmc::crate_name("rust-i18n").expect("rust-i18n must be present in `Cargo.toml`, but it's not");

    let ident = match found_crate {
        pmc::FoundCrate::Itself => Ident::new("crate", Span::call_site()),
        pmc::FoundCrate::Name(name) => {
            Ident::new(&name, Span::call_site())
        }
    };
    syn::Path::from(ident)
}
