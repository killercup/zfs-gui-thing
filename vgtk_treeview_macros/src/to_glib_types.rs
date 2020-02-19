use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mapped_types = map_types(&input.data);

    let expanded = quote! {
        impl #impl_generics vgtk_treeview::ToGlibTypes for #name #ty_generics #where_clause {
            fn to_glib_types() -> Vec<vgtk::lib::glib::types::Type> {
                #mapped_types
            }
        }
    };

    TokenStream::from(expanded)
}

fn map_types(data: &Data) -> impl ToTokens {
    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let items = fields.iter().map(|field| {
                let ty = &field.ty;
                quote_spanned! { field.span() => <#ty as vgtk::lib::glib::types::StaticType>::static_type() }
            });
            quote!(std::vec![ #( #items ),* ])
        }
        _ => unimplemented!("Can only derive GLib types for structs for now"),
    }
}
