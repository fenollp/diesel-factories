extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use regex::Regex;
use syn::punctuated::Pair;
use syn::Ident;
use syn::{parse_macro_input, Attribute, DeriveInput};
use syn::{Data, Fields, FieldsNamed};

#[proc_macro_derive(Factory, attributes(factory_model, factory_default))]
pub fn derive_factory(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let model_name = model_name(&input.attrs);
    let factory_name = input.ident.clone();
    let fields = struct_fields(input);

    let methods = fields
        .named
        .into_pairs()
        .map(|pair| match pair {
            Pair::Punctuated(field, _) => field,
            Pair::End(field) => field,
        })
        .map(|field| {
            let name = field.ident.unwrap_or_else(|| panic!("Field without name"));
            let ty = field.ty;
            quote! {
                pub fn #name<T: Into<#ty>>(mut self, value: T) -> Self {
                    self.#name = value.into();
                    self
                }
            }
        })
        .collect::<Vec<_>>();

    let tokens = quote! {
        impl diesel_factories::DefaultFactory<#factory_name> for #model_name {}

        impl #factory_name {
            #(#methods)*
        }
    };

    tokens.into()
}

fn model_name(attrs: &Vec<Attribute>) -> Ident {
    let factory_model_attr = attrs.into_iter().find(|attr| {
        attr.path
            .segments
            .iter()
            .any(|segment| &segment.ident.to_string() == "factory_model")
    });

    let factory_model_attr = match factory_model_attr {
        Some(x) => x,
        None => panic!(
            "#[derive(Factory)] requires you to also set the attribute #[factory_model(...)]"
        ),
    };

    let attr = factory_model_attr.tts.to_string();

    let re = Regex::new(r"\( (?P<name>.*?) \)").unwrap();
    let caps = re.captures(&attr).expect(
        "The `factory_model` attributes must be on the form `#[factory_model(SomeStruct)]`",
    );

    ident(&caps["name"])
}

fn ident(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}

fn struct_fields(input: DeriveInput) -> FieldsNamed {
    let err = || panic!("Factory can only be derived on structs with named fields");

    match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) => fields,
            _ => err(),
        },
        _ => err(),
    }
}