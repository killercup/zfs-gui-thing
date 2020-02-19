use std::marker::PhantomData;
use vgtk::lib::{
    glib::types::Type,
    gtk::{TreeStore, TreeViewColumn},
};
pub use vgtk_treeview_macros::*;

pub trait ToGlibTypes {
    fn to_glib_types() -> Vec<Type>;
}

pub trait ToTreeViewColumns {
    fn to_treeview_columns() -> Vec<TreeViewColumn>;
}

#[derive(Clone, Debug)]
struct VTreeView<T: ToTreeViewColumns + ToGlibTypes> {
    row_type: PhantomData<T>,
    data: TreeStore,
}

impl<T: ToTreeViewColumns + ToGlibTypes> Default for VTreeView<T> {
    fn default() -> Self {
        VTreeView {
            row_type: PhantomData,
            data: TreeStore::new(&T::to_glib_types()),
        }
    }
}
