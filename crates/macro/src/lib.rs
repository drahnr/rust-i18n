//! Parse the content of `format_t!` arguments
//!
//! Required, to filter out `a.b.c` style paths
//! and avoid erroring for no good reason.

use fs_err as fs;
use proc_macro2::Ident;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::Token;
use syn::{parse::Parse, punctuated::Punctuated, Expr};

/// A single argument as passed to `format!`
///
/// Skips the initial literal string!
enum FormatArg {
    AliasEqIdent {
        alias: Ident,
        #[allow(dead_code)]
        eq: Token![=],
        ident: Ident,
    },
    AliasEqExpr {
        alias: Ident,
        #[allow(dead_code)]
        eq: Token![=],
        expr: Expr,
    },
    Ident {
        ident: Ident,
    },
}

use std::fmt;

impl fmt::Debug for FormatArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ident { ident } => {
                write!(f, "{ident}")?;
            }
            Self::AliasEqIdent { alias, ident, .. } => {
                write!(f, "{alias} = {ident}")?;
            }
            Self::AliasEqExpr { alias, .. } => {
                write!(f, "{alias} = <expr>")?;
            }
        }

        Ok(())
    }
}

impl syn::parse::Parse for FormatArg {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let ident = input.parse()?;

        let lookahead = input.lookahead1();
        let me = if lookahead.peek(Token![=]) {
            let eq = input.parse::<Token![=]>()?;
            let alias = ident;

            let expr = input.parse::<Expr>().map_err(|_e| {
                syn::Error::new(
                    input.span(),
                    "Expected `Expr` after = since it's not an ident",
                )
            })?;
            Self::AliasEqExpr { alias, eq, expr }
        } else {
            Self::Ident { ident }
        };
        Ok(me)
    }
}

impl ToTokens for FormatArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident { ident } => tokens.extend(ident.to_token_stream()),
            Self::AliasEqExpr { alias, eq, expr } => tokens.extend(quote! { #alias #eq #expr }),
            Self::AliasEqIdent { alias, eq, ident } => tokens.extend(quote! { #alias #eq #ident }),
        }
    }
}

/// All format arguments.
///
/// Including the str literal.
struct FormatArgs {
    fmt_str: syn::LitStr,
    #[allow(dead_code)]
    maybe_comma: Option<Token![,]>,
    maybe_args: Punctuated<FormatArg, Token![,]>,
}

impl Parse for FormatArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<syn::Lit>()?;
        let fmt_str = match lit {
            syn::Lit::Str(alias) => alias,
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "Expected a literal str for format arg but found...",
                ))
            }
        };
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![,]) {
            let comma = input.parse::<Token![,]>()?;

            let maybe_comma = Some(comma);
            let maybe_args = Punctuated::<FormatArg, Token![,]>::parse_terminated(&input)?;

            Ok(Self {
                fmt_str,
                maybe_comma,
                maybe_args,
            })
        } else {
            Ok(Self {
                fmt_str,
                maybe_comma: None,
                maybe_args: Punctuated::new(),
            })
        }
    }
}

impl ToTokens for FormatArgs {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let FormatArgs {
            ref fmt_str,
            maybe_comma,
            ref maybe_args,
        } = self;
        tokens.extend(fmt_str.to_token_stream());
        if let Some(comma) = maybe_comma {
            comma.to_tokens(tokens);
            if !maybe_args.is_empty() {
                tokens.extend(maybe_args.to_token_stream());
            }
        }
    }
}

impl fmt::Debug for FormatArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r##""{}""##, self.fmt_str.value())?;
        if let Some(_comma) = self.maybe_comma {
            f.write_str(",")?;
            for pair in self.maybe_args.pairs() {
                write!(f, "{:?},", pair.value())?;
            }
        }
        Ok(())
    }
}

fn format_inner(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let support = support_crate_path();
    let FormatArgs {
        fmt_str,
        maybe_comma: _,
        maybe_args,
    } = syn::parse2(input)?;

    // must be (a.b.c -> (language_2_letter_code -> translation_text)* )*

    let path = if let Ok(locale_dir) = std::env::var("I18N_LOCALES_SOURCE_DIR") {
        std::path::PathBuf::from(locale_dir)
    } else {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("locales")
    };
    path.display();
    eprintln!("Reading {}", path.display());
    let bytes = fs::read(&path).unwrap();
    let tp2trans_per_locale = rust_i18n_support::deserialize(&bytes[..]).unwrap();

    eprintln!("Read {:?}", &tp2trans_per_locale);
    // Will cause quite a bit of load during compilation for applications with many
    // invocations, but whatever...
    let tp = fmt_str.value();
    let tp = tp.as_str();
    let translations: &HashMap<String, String> = tp2trans_per_locale.get(tp).ok_or_else(|| {
        syn::Error::new(
            Span::call_site(),
            format!("No translation for \"{tp}\" in {tp2trans_per_locale:?}"),
        )
    })?;

    let language = translations.keys();
    let translation = translations.values().map(|v| v.trim().to_owned());
    let ts = quote!(
        match #support::locale() {
            #( #language => { ::std::format!( #translation, #maybe_args ) }, )*
            _ => { "<missing translation>".to_owned() }, // TODO FIXME, use a default language
        }
    );
    println!("{s}", s = ts.to_string());
    Ok(ts)
}

#[proc_macro]
pub fn format_t(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    format_inner(proc_macro2::TokenStream::from(input))
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn support_crate_path() -> syn::Path {
    use proc_macro_crate as pmc;
    let found_crate = pmc::crate_name("rust-i18n")
        .expect("rust-i18n must be present in `Cargo.toml`, but it's not");

    let ident = match found_crate {
        pmc::FoundCrate::Itself => Ident::new("crate", Span::call_site()),
        pmc::FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    };
    syn::Path::from(ident)
}

#[cfg(test)]
mod tests;
