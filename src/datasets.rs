use anyhow::{Context, Result};
use vgtk::lib::glib::object::ObjectExt;
use vgtk::lib::gtk::*;
use vgtk_treeview::*;

#[derive(Clone, Default, Debug, ToGlibTypes, ToTreeViewColumns)]
pub struct Dataset {
    #[tree_view_column(name = "Dataset")]
    pub name: String,
    #[tree_view_column(name = "Used", presenter = "human_format")]
    pub used: u64,
    #[tree_view_column(name = "Compression ratio")]
    pub compressratio: f64,
    #[tree_view_column(name = "Refers to", presenter = "human_format")]
    pub refer: u64,
    #[tree_view_column(name = "Available", presenter = "human_format")]
    pub avail: u64,
}

impl Dataset {
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
                let compressratio = columns.next().context("no compressratio column")?.parse()?;
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
        use vgtk::lib::{glib::Value, gtk::prelude::TreeStoreExtManual};
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
            ],
        );
        Ok(())
    }
}

use humansize::{file_size_opts as options, FileSize};

fn human_format(x: impl FileSize + ToString) -> String {
    x.file_size(options::BINARY)
        .unwrap_or_else(|_| x.to_string())
}
