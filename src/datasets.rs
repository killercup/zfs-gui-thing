use vgtk::lib::glib::{types::Type, object::ObjectExt};
use vgtk::lib::gtk::*;
use anyhow::{Context, Result};

#[derive(Clone, Default, Debug)]
pub struct Dataset {
    pub name: String,
    pub used: u64,
    pub compressratio: f64,
    pub refer: u64,
    pub avail: u64,
}

impl Dataset {
    pub fn to_glib_types() -> &'static [Type] {
        &[
            Type::String,
            Type::U64, 
            Type::F64,
            Type::U64,
            Type::U64,
        ]
    }

    pub fn to_treeview_columns() -> Vec<TreeViewColumn> {
        vec![
            {
                let x = TreeViewColumn::new();
                let cell = CellRendererText::new();
                x.pack_start(&cell, true);
                x.add_attribute(&cell, "text", 0);

                x.set_title("name");
                x.set_expand(true);
                x.set_sort_column_id(0);
                x
            },
            {
                let x = TreeViewColumn::new();
                let cell = CellRendererText::new();
                x.pack_start(&cell, true);
                x.add_attribute(&cell, "text", 1);

                x.set_title("used");
                x.set_sort_column_id(1);

                fn fmt(
                    _col: &TreeViewColumn, cell: &CellRenderer, model: &TreeModel, iter: &TreeIter
                ) {
                    cell.set_property("text", &human_format(model.get_value(iter, 1).get::<u64>().unwrap().unwrap())).unwrap();
                }

                TreeViewColumnExt::set_cell_data_func(&x, &cell, Some(std::boxed::Box::new(fmt)));
                x
            },
            {
                let x = TreeViewColumn::new();
                let cell = CellRendererText::new();
                x.pack_start(&cell, true);
                x.add_attribute(&cell, "text", 2);

                x.set_title("compressratio");
                x.set_sort_column_id(2);
                x
            },
            {
                let x = TreeViewColumn::new();
                let cell = CellRendererText::new();
                x.pack_start(&cell, true);
                x.add_attribute(&cell, "text", 3);

                x.set_title("refer");
                x.set_sort_column_id(3);

                fn fmt(
                    _col: &TreeViewColumn, cell: &CellRenderer, model: &TreeModel, iter: &TreeIter
                ) {
                    cell.set_property("text", &human_format(model.get_value(iter, 3).get::<u64>().unwrap().unwrap())).unwrap();
                }

                TreeViewColumnExt::set_cell_data_func(&x, &cell, Some(std::boxed::Box::new(fmt)));
                x
            },
            {
                let x = TreeViewColumn::new();
                let cell = CellRendererText::new();
                x.pack_start(&cell, true);
                x.add_attribute(&cell, "text", 4);

                x.set_title("avail");
                x.set_sort_column_id(4);

                fn fmt(
                    _col: &TreeViewColumn, cell: &CellRenderer, model: &TreeModel, iter: &TreeIter
                ) {
                    cell.set_property("text", &human_format(model.get_value(iter, 4).get::<u64>().unwrap().unwrap())).unwrap();
                }

                TreeViewColumnExt::set_cell_data_func(&x, &cell, Some(std::boxed::Box::new(fmt)));
                x
            },
        ]
    }

    pub fn fetch_all() -> Result<Vec<Self>> {
        let list = std::process::Command::new("zfs")
            .args(&[
                "list",
                "-o",
                "name,used,compressratio,refer,avail",
                "-r",
                "-H",
                "-p",
            ])
            .output()
            .context("failed to run `zfs list`")?;
        anyhow::ensure!(list.status.success(), "`zfs list` unsuccessful");
        let list = String::from_utf8(list.stdout).context("parse `zfs list` as UTF-8")?;

        list.trim()
            .lines()
            .map(|line| {
                let mut columns = line.split('\t');
                let name = columns.next().context("no name column")?.to_string();
                let used = columns.next().context("no used column")?.parse()?;
                let compressratio = columns
                    .next()
                    .context("no compressratio column")?
                    .parse()?;
                let refer = columns.next().context("no refer column")?.parse()?;
                let avail = columns.next().context("no avail column")?.parse()?;
                Ok(Dataset {
                    name,
                    used,
                    compressratio,
                    refer,
                    avail,
                })
            })
            .collect()
    }

    pub fn append_to(&self, tree: &TreeStore) -> Result<()> {
        use vgtk::lib::{gtk::prelude::TreeStoreExtManual, glib::{Value}};
        tree.insert_with_values(
            None,
            None,
            &[0, 1, 2, 3, 4],
            &[
                &Value::from(&self.name),
                &Value::from(&self.used),
                &Value::from(&self.compressratio),
                &Value::from(&self.refer),
                &Value::from(&self.avail),
            ]
        );
        Ok(())
    }
}

use humansize::{FileSize, file_size_opts as options};

fn human_format(x: impl FileSize + ToString) -> String {
    x.file_size(options::BINARY).unwrap_or_else(|_| x.to_string())
}
