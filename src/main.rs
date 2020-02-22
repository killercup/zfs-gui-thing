#![recursion_limit = "1024"]

use vgtk::ext::*;
use vgtk::lib::gio::{ActionExt, ApplicationFlags, SimpleAction};
use vgtk::lib::glib::object::Cast;
use vgtk::lib::gtk::*;
use vgtk::{gtk, run, Component, UpdateAction, VNode};

use anyhow::{Context, Result};
use async_std::task;
use vgtk_treeview::*;

mod datasets;
use datasets::Dataset;

#[derive(Clone, Debug)]
struct Model {
    datasets: TreeStore,
}

impl Model {
    fn update_datasets(&mut self, datasets: &[Dataset]) -> Result<()> {
        self.datasets.clear();
        for dataset in datasets {
            dataset.append_to_treestore(&self.datasets, None);
        }
        Ok(())
    }

    fn update_datasets_with_snapshots(&mut self, datasets: &[(String, Dataset)]) -> Result<()> {
        for (dataset, snapshot) in datasets {
            let name_column = 0;
            let mut parent = None;
            self.datasets.foreach(|model, _path, iter| {
                let name = model
                    .get_value(iter, name_column)
                    .get::<String>()
                    .unwrap()
                    .unwrap();
                if &name == dataset {
                    parent = Some(iter.clone());
                    true
                } else {
                    false
                }
            });
            snapshot.append_to_treestore(&self.datasets, parent.as_ref());
        }
        Ok(())
    }
}

impl Default for Model {
    fn default() -> Self {
        Model {
            datasets: TreeStore::new(&Dataset::to_glib_types()),
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    LoadDatasets,
    DatasetsLoaded((Vec<Dataset>, Vec<(String, Dataset)>)),
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
            Message::LoadDatasets => UpdateAction::defer(async {
                let list = task::spawn(async {
                    (
                        Dataset::fetch_datasets().context("load datasets").unwrap(),
                        Dataset::fetch_snapshots()
                            .context("load snapshots")
                            .unwrap(),
                    )
                })
                .await;
                Message::DatasetsLoaded(list)
            }),
            Message::DatasetsLoaded((datasets, snapshots)) => {
                self.update_datasets(&datasets).unwrap();
                self.update_datasets_with_snapshots(&snapshots).unwrap();
                UpdateAction::Render
            }
        }
    }

    fn view(&self) -> VNode<Model> {
        let model: Option<TreeModel> = Some(self.datasets.clone().upcast());

        gtk! {
            <Application::new_unwrap(Some("com.example.zfs-gui-thingy"), ApplicationFlags::empty())>
                <SimpleAction::new("quit", None) Application::accels=["<Ctrl>q"].as_ref() enabled=true on activate=|a, _| Message::Exit/>

                <ApplicationWindow default_width=800 default_height=480 border_width=20 on destroy=|_| Message::Exit>
                    <HeaderBar title="ZFS Datasets" show_close_button=true />
                    <Box orientation=Orientation::Vertical>
                        <ScrolledWindow Box::expand=true Box::fill=true>
                            <TreeView::new()
                                model=model
                                headers_clickable=true
                                enable_search=true
                                tooltip_column=0
                                on show=|tree_view| {
                                    for column in Dataset::to_treeview_columns() {
                                        tree_view.append_column(&column);
                                    }
                                    Message::LoadDatasets
                                }>
                            </TreeView>
                        </ScrolledWindow>
                    </Box>
                </ApplicationWindow>
            </Application>
        }
    }
}

fn main() {
    pretty_env_logger::init();
    std::process::exit(run::<Model>());
}
