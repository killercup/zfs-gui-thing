#![recursion_limit = "1024"]

use vgtk::ext::*;
use vgtk::lib::gio::{ApplicationFlags, SimpleAction, ActionExt};
use vgtk::lib::gtk::*;
use vgtk::{gtk, run, Component, UpdateAction, VNode};

use anyhow::{Context, Result};

#[derive(Clone, Debug, Default)]
struct Model {
    datasets: Vec<Dataset>,
}

#[derive(Clone, Debug)]
enum Message {
    LoadDatasets,
    DatasetsLoaded(Vec<Dataset>),
    Exit,
}

impl Component for Model {
    type Message = Message;
    type Properties = ();

    fn update(&mut self, msg: Self::Message) -> UpdateAction<Self> {
        match msg {
            Message::Exit => {
                vgtk::quit();
                UpdateAction::None
            }
            Message::LoadDatasets => UpdateAction::defer(async move {
                Message::DatasetsLoaded(Dataset::fetch_all().context("load datasets").unwrap())
            }),
            Message::DatasetsLoaded(datasets) => {
                self.datasets = datasets;
                UpdateAction::Render
            }
        }
    }

    fn view(&self) -> VNode<Model> {
        gtk! {
            <Application::new_unwrap(Some("com.example.zfs-gui-thingy"), ApplicationFlags::empty())>
                <SimpleAction::new("quit", None) Application::accels=["<Ctrl>q"].as_ref() enabled=true on activate=|a, _| Message::Exit/>

                <ApplicationWindow default_width=800 default_height=480 border_width=20 on destroy=|_| Message::Exit>
                    <HeaderBar title="ZFS Datasets" show_close_button=true />
                    <Box orientation=Orientation::Vertical>
                        <ScrolledWindow Box::expand=true Box::fill=true on show=|_| Message::LoadDatasets>
                            <ListBox>
                                {
                                    self.datasets.iter().enumerate()
                                        .map(|(index, item)| item.render(index))
                                }
                            </ListBox>
                        </ScrolledWindow>
                    </Box>
                </ApplicationWindow>
            </Application>
        }
    }
}

#[derive(Clone, Default, Debug)]
struct Dataset {
    pub name: String,
    pub used: u64,
    pub compressratio: f64,
    pub refer: u64,
    pub avail: u64,
}

impl Dataset {
    fn fetch_all() -> Result<Vec<Self>> {
        let list = std::process::Command::new("zfs")
            .args(&[
                "list",
                "-o",
                "name,used,compressratio,refer,avail",
                "-r",
                "rpool",
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

    fn render(&self, _index: usize) -> VNode<Model> {
        gtk! {
            <ListBoxRow>
                <Box spacing=10 orientation=Orientation::Horizontal>
                    <Label label=self.name.clone() use_markup=true Box::fill=true />
                    <Label label={human_format::Formatter::new().with_scales(human_format::Scales::Binary()).format(self.used as f64)} use_markup=true Box::fill=true />
                </Box>
            </ListBoxRow>
        }
    }
}

fn main() {
    pretty_env_logger::init();
    std::process::exit(run::<Model>());
}
