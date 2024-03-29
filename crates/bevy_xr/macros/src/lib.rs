use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Comma, Eq};
use syn::{parse_macro_input, parse_quote, AttrStyle, DeriveInput, Expr, Type};
use syn::parse::{Parse, ParseStream};

mod kw {
    syn::custom_keyword!(action_type);
    syn::custom_keyword!(name);
    syn::custom_keyword!(pretty_name);
}

enum AttributeInput {
    Kind(Type, Span),
    Name(Expr, Span),
    PrettyName(Expr, Span),
}

impl Parse for AttributeInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(field) = input.parse::<kw::action_type>() {
            input.parse::<Eq>()?;
            Ok(Self::Kind(input.parse()?, field.span()))
        } else if let Ok(field) = input.parse::<kw::name>() {
            input.parse::<Eq>()?;
            Ok(Self::Name(input.parse()?, field.span()))
        } else if let Ok(field) = input.parse::<kw::pretty_name>() {
            input.parse::<Eq>()?;
            Ok(Self::PrettyName(input.parse()?, field.span()))
        } else {
            Err(input.error("expected 'action_type', 'name' or 'pretty_name'"))
        }
    }
}


#[proc_macro_derive(Action, attributes(action))]
pub fn derive_action(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let item = input.ident;
    let mut attributes = vec![];
    for attribute in input.attrs {
        if let AttrStyle::Inner(token) = attribute.style {
            return TokenStream::from(syn::Error::new(token.span, "This derive macro does not accept inner attributes").to_compile_error());
        };
        let parsed_attributes = match attribute.parse_args_with(Punctuated::<AttributeInput, Comma>::parse_terminated) {
            Ok(inner) => inner,
            Err(e) => return TokenStream::from(e.to_compile_error()),
        };
        for attr in parsed_attributes {
            attributes.push(attr);
        }
    }
    let mut kind: Option<Type> = None;
    let mut name: Option<Expr> = None;
    let mut pretty_name: Option<Expr> = None;
    for attribute in attributes {
        match attribute {
            AttributeInput::Kind(ty, span) => {
                if kind.is_some() {
                    return syn::Error::new(span, "attribute 'action_type' is defined more than once").to_compile_error().into();
                }

                kind = Some(ty);
            },
            AttributeInput::Name(expr, span) => {
                if name.is_some() {
                    return syn::Error::new(span, "attribute 'name' is defined more than once").to_compile_error().into();
                }

                name = Some(expr);
            },
            AttributeInput::PrettyName(expr, span) => {
                if pretty_name.is_some() {
                    return syn::Error::new(span, "attribute 'pretty_name' is defined more than once").to_compile_error().into();
                }

                pretty_name = Some(expr);
            },
        }
    };

    if kind.is_none() {
        panic!("action_type isn't specified")
    }
    if name.is_none() {
        name = Some(parse_quote! {
            std::stringify!(#item)
        });
    }
    if pretty_name.is_none() {
        pretty_name = name.clone();
    }
    let kind = kind.unwrap();
    let name = name.unwrap();
    let pretty_name = pretty_name.unwrap();

    let expanded = quote! {
        impl bevy_xr::actions::Action for #item {
            type ActionType = #kind;

            fn info() -> bevy_xr::actions::ActionInfo {
                bevy_xr::actions::ActionInfo {
                    pretty_name: #pretty_name,
                    name: #name,
                    action_type: <Self::ActionType as bevy_xr::actions::ActionTy>::TYPE,
                    type_id: std::any::TypeId::of::<Self>(),
                }
            }
        }
    };

    TokenStream::from(expanded)
}