use darling::FromField;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Field};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let columns = collect_columns(&input.data);
    let append_to = build_append_to(&input.data);

    let expanded = quote! {
        impl #impl_generics vgtk_treeview::ToTreeViewColumns for #name #ty_generics #where_clause {
            fn to_treeview_columns() -> Vec<vgtk::lib::gtk::TreeViewColumn> {
                #columns
            }

            fn append_to_treestore(&self, tree: &TreeStore, parent: Option<&TreeIter>) {
                #append_to
            }
        }
    };

    TokenStream::from(expanded)
}

fn collect_columns(data: &Data) -> impl ToTokens {
    match data {
        Data::Struct(DataStruct { fields, .. }) => {
            let items = fields.iter().enumerate().map(|(idx, f)| {
                build_column(f, idx)
                    .map(|x| x.to_token_stream())
                    .unwrap_or_else(|e| e.to_compile_error())
            });
            quote!(std::vec![ #( { #items } ),* ])
        }
        _ => unimplemented!("Can only derive GLib types for structs for now"),
    }
}

#[derive(FromField, Default)]
#[darling(default, attributes(tree_view_column))]
struct ColumnParams {
    renderer: Option<syn::Path>,
    name: Option<String>,
    expand: bool,
    presenter: Option<syn::Path>,
}

fn build_column(field: &Field, idx: usize) -> Result<impl ToTokens, syn::Error> {
    let params = match ColumnParams::from_field(field) {
        Ok(val) => val,
        Err(err) => {
            return Ok(err.write_errors());
        }
    };

    let idx = idx as i32;
    let field_ty = &field.ty;
    let renderer = &params
        .renderer
        .unwrap_or(syn::parse2(quote!(vgtk::lib::gtk::CellRendererText))?);
    let name = &params
        .name
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());
    let expand = params.expand;

    let data_funk = if let Some(presenter) = params.presenter {
        quote! {
            fn fmt(
                _col: &TreeViewColumn,
                cell: &CellRenderer,
                model: &TreeModel,
                iter: &TreeIter,
            ) {
                let data = model.get_value(iter, #idx).get::<#field_ty>().unwrap().unwrap();
                cell.set_property(
                    "text",
                    &#presenter(data),
                )
                .unwrap();
            }

            TreeViewColumnExt::set_cell_data_func(&x, &cell, Some(std::boxed::Box::new(fmt)));
        }
    } else {
        quote!()
    };

    Ok(quote! {
        let x = vgtk::lib::gtk::TreeViewColumn::new();
        let cell = #renderer::new();
        x.pack_start(&cell, true);
        x.add_attribute(&cell, "text", #idx);

        x.set_title(#name);
        x.set_expand(#expand);
        x.set_sort_column_id(#idx);

        #data_funk

        x
    })
}

fn build_append_to(data: &Data) -> impl ToTokens {
    let fields = match data {
        Data::Struct(DataStruct { fields, .. }) => fields,
        _ => unimplemented!("Can only derive `ToTreeViewColumns` for structs for now"),
    };

    let column_indices = fields
        .iter()
        .enumerate()
        .map(|(idx, f)| quote_spanned!(f.span() => { #idx as u32 }));
    let column_mapping = quote! {
        &[ #( #column_indices ),* ]
    };

    let field_names = fields.iter().map(|f| {
        let name = &f.ident;
        quote_spanned!(f.span() => { &Value::from(&self.#name) })
    });
    let field_mapping = quote! {
        &[ #( #field_names ),* ]
    };

    quote! {
        use vgtk::lib::{glib::Value, gtk::prelude::TreeStoreExtManual};

        tree.insert_with_values(
            parent,
            None,
            #column_mapping,
            #field_mapping,
        );
    }
}
