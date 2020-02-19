extern crate proc_macro;
use proc_macro::TokenStream;

mod to_glib_types;
mod to_treeview_columns;

#[proc_macro_derive(ToGlibTypes)]
pub fn to_glib_types(input: TokenStream) -> TokenStream {
    to_glib_types::expand(input)
}

#[proc_macro_derive(ToTreeViewColumns, attributes(tree_view_column))]
pub fn to_treeview_columns(input: TokenStream) -> TokenStream {
    to_treeview_columns::expand(input)
}
