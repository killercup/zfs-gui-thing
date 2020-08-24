use anyhow::{Context, Result};
use vgtk::lib::glib::{object::ObjectExt, prelude::*};
use vgtk::lib::gtk::*;
use vgtk_treeview::*;

#[derive(Clone, Default, Debug, ToGlibTypes, ToTreeViewColumns)]
pub struct Dataset {
    #[tree_view_column(name = "Dataset", presenter = "path_segments")]
    pub name: String,
    #[tree_view_column(name = "Used", presenter = "human_format")]
    pub used: u64,
    #[tree_view_column(name = "Compression ratio")]
    pub compressratio: f64,
    #[tree_view_column(name = "Refers to", presenter = "human_format")]
    pub refer: u64,
    #[tree_view_column(name = "Available", presenter = "human_format")]
    pub avail: u64,
    #[tree_view_column(name = "Type")]
    pub kind: Type,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib_macros::GEnum)]
#[repr(u32)]
#[genum(type_name = "TestAnimalType")]
pub enum Type {
    Dataset,
    Snapshot,
}

impl Default for Type {
    fn default() -> Type {
        Type::Dataset
    }
}

impl Dataset {
    pub fn fetch_datasets() -> Result<Vec<Self>> {
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
                    kind: Type::Dataset,
                })
            })
            .collect()
    }

    pub fn fetch_snapshots() -> Result<Vec<(String, Self)>> {
        let list = std::process::Command::new("zfs")
            .args(&[
                "list",
                "-t",
                "snapshot",
                "-o",
                "name,used,compressratio,refer",
                "-r",
                "-H",
                "-p",
            ])
            .output()
            .context("run `zfs list -t snapshot`")?;
        anyhow::ensure!(list.status.success(), "`zfs list -t snapshot` unsuccessful");
        let list =
            String::from_utf8(list.stdout).context("parse `zfs list -t snapshot` as UTF-8")?;

        list.trim()
            .lines()
            .map(|line| {
                let mut columns = line.split('\t');
                let name = columns
                    .next()
                    .context("name")?
                    .splitn(2, '@')
                    .collect::<Vec<_>>();
                anyhow::ensure!(
                    name.len() == 2,
                    "expected name to be like x@y, was {:?}",
                    name
                );
                let (dataset_name, snapshot_name) = (name[0].to_string(), name[1].to_string());
                let used = columns.next().context("used")?.parse().context("used")?;
                let compressratio = columns
                    .next()
                    .context("compressratio")?
                    .parse()
                    .context("compressratio")?;
                let refer = columns.next().context("refer")?.parse().context("refer")?;
                let avail = 0; // n/a for snapshots
                Ok((
                    dataset_name,
                    Dataset {
                        name: snapshot_name,
                        used,
                        compressratio,
                        refer,
                        avail,
                        kind: Type::Snapshot,
                    },
                ))
            })
            .collect()
    }
}

use humansize::{file_size_opts as options, FileSize};

fn human_format(x: impl FileSize + ToString) -> String {
    x.file_size(options::BINARY)
        .unwrap_or_else(|_| x.to_string())
}

fn path_segments(x: String) -> String {
    x.split('/').last().map(|x| x.to_string()).unwrap_or(x)
}
