use syn::LitStr;

use super::*;

macro_rules! gen_fmtarg_test {
    (pass: $x:expr) => {{
        let fmt_args: FormatArg = syn::parse2(quote::quote! {
            $x
        })
        .expect("FormatArg must parse. qed");
        dbg!(fmt_args)
    }};
}

macro_rules! gen_fmtargs_test {
    (pass: $fmt:literal $(,$x:expr)* $(,)?) => {
        {
            let fmt_args:FormatArgs = syn::parse2(quote::quote!{
                $fmt $(, $x)*
            }).expect("FormatArgs must parse. qed");
            fmt_args
        }
    };
}

macro_rules! roundtrip {
    ($ty:ty; $setup:expr ) => {
        let seed: $ty = $setup;
        println!("{:?}", &seed);
        let seed_str = format!("{:?}", &seed);
        let ts = dbg!(seed.into_token_stream());
        let reconstructed: $ty = syn::parse2::<$ty>(ts).expect("Must parse, was ok before");

        assert_eq!(seed_str, format!("{:?}", reconstructed));
    };
}

#[test]
fn roundtrip_fmtarg_ident_eq_ident() {
    let sweed = FormatArg::AliasEqIdent {
        alias: Ident::new("mynameis", Span::mixed_site()),
        eq: Token![=](Span::call_site()),
        ident: Ident::new("alois", Span::mixed_site()),
    };
    roundtrip!(FormatArg; 
        sweed);
}

#[test]
fn roundtrip_fmtarg_ident_eq_expr() {
    let sweed = FormatArg::AliasEqExpr {
        alias: Ident::new("mynameis", Span::mixed_site()),
        eq: Token![=](Span::call_site()),
        expr: syn::parse2::<syn::Expr>(quote! { { int_a + int_b } }).unwrap(),
    };
    roundtrip!(FormatArg; 
        sweed);
}

#[test]
fn roundtrip_fmtargs() {
    let sweed = FormatArgs {
        fmt_str: LitStr::new("foo bar bay", Span::call_site()),
        maybe_comma: Some(Token![,](Span::call_site())),
        maybe_args: {
            let mut p = Punctuated::new();
            p.push(FormatArg::AliasEqExpr {
                alias: Ident::new("mynameis007", Span::mixed_site()),
                eq: Token![=](Span::call_site()),
                expr: syn::parse2::<syn::Expr>(quote! { { int_a + int_b } }).unwrap(),
            });
            p.push(FormatArg::AliasEqIdent {
                alias: Ident::new("mynameis7", Span::mixed_site()),
                eq: Token![=](Span::call_site()),
                ident: Ident::new("value", Span::mixed_site()),
            });
            p
        },
    };

    roundtrip!(FormatArgs; 
        sweed);
}

#[test]
fn coll_arg_y() {
    gen_fmtarg_test!(pass: a);
    gen_fmtarg_test!(
        pass: b = {
            let x = foo?;
            x
        }
    );
    gen_fmtarg_test!(pass: x = y);
}

#[test]
fn coll_args() {
    gen_fmtargs_test!(pass: "f.b.q");
    gen_fmtargs_test!(pass: "f.b.q",);
    gen_fmtargs_test!(pass: "f.b.q", x);
    gen_fmtargs_test!(pass: "f.b.q", x,);
    gen_fmtargs_test!(pass: "a.b.c", foo=b, bar);
    gen_fmtargs_test!(pass: "x.y.z", b = { let x = foo?; x }, foo = bar, poo);
}
