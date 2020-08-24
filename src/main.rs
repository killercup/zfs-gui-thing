#![recursion_limit = "1024"]

use vgtk::ext::*;
use vgtk::lib::gio::{prelude::ApplicationExtManual, ActionExt, ApplicationFlags, SimpleAction};
use vgtk::lib::glib::object::Cast;
use vgtk::lib::gtk::*;
use vgtk::{gtk, Component, UpdateAction, VNode};

use anyhow::{Context, Result};
use vgtk_treeview::*;

mod datasets;
use datasets::Dataset;

fn main() {
    pretty_env_logger::init();
    let (app, scope) = vgtk::start::<Model>();

    let _worker = std::thread::spawn(move || {
        let mut datasets = Dataset::fetch_datasets().context("load datasets").unwrap();
        datasets.sort_by(|a, b| a.name.cmp(&b.name));
        log::debug!("got {} datasets", datasets.len());
        scope.send_message(Message::UpdateDatasets(datasets));
        let snapshots = Dataset::fetch_snapshots()
            .context("load snapshots")
            .unwrap();
        log::debug!("got {} snapshots", snapshots.len());
        scope.send_message(Message::InsertSnapshots(snapshots));
    });

    let args: Vec<String> = std::env::args().collect();
    let exit_status = app.run(&args);
    std::process::exit(exit_status);
}

#[derive(Clone, Debug)]
struct Model {
    datasets: TreeStore,
    show_snapshots: bool,
}

fn find_parent(store: &TreeStore, mut path: Vec<&str>) -> Option<TreeIter> {
    let mut parent = None;
    let _child_name = path.pop();
    let parent_path = path;
    if parent_path.is_empty() {
        return None;
    }
    let parent_name = parent_path.join("/");

    store.foreach(|model, _path, iter| {
        let name = model
            .get_value(iter, Dataset::NAME_COLUMN_INDEX)
            .get::<String>()
            .unwrap()
            .unwrap();
        if name == parent_name {
            parent = Some(iter.clone());
            true
        } else {
            false
        }
    });
    parent
}

impl Model {
    fn update_datasets(&mut self, datasets: &[Dataset]) -> Result<()> {
        self.datasets.clear();

        for dataset in datasets {
            let parent = find_parent(&self.datasets, dataset.name.split('/').collect());
            dataset.append_to_treestore(&self.datasets, parent.as_ref());
        }
        Ok(())
    }

    fn update_datasets_with_snapshots(&mut self, datasets: &[(String, Dataset)]) -> Result<()> {
        for (dataset, snapshot) in datasets {
            let parent = find_parent(&self.datasets, dataset.split('/').collect());
            snapshot.append_to_treestore(&self.datasets, parent.as_ref());
        }
        Ok(())
    }
}

impl Default for Model {
    fn default() -> Self {
        Model {
            datasets: TreeStore::new(&Dataset::to_glib_types()),
            show_snapshots: false,
        }
    }
}

#[derive(Clone, Debug)]
enum Message {
    Init,
    UpdateDatasets(Vec<Dataset>),
    InsertSnapshots(Vec<(String, Dataset)>),
    ShowSnapshots(bool),
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
            Message::Init => {
                log::info!("hello");
                UpdateAction::None
            }
            Message::UpdateDatasets(datasets) => {
                self.update_datasets(&datasets).unwrap();
                UpdateAction::Render
            }
            Message::InsertSnapshots(snapshots) => {
                self.update_datasets_with_snapshots(&snapshots).unwrap();
                UpdateAction::Render
            }
            Message::ShowSnapshots(flag) => {
                self.show_snapshots = flag;
                UpdateAction::Render
            }
        }
    }

    fn view(&self) -> VNode<Model> {
        let filter = TreeModelFilter::new(&self.datasets, None);
        let show = self.show_snapshots;
        filter.set_visible_func(move |model, iter| {
            let x = model
                .get_value(iter, Dataset::KIND_COLUMN_INDEX)
                .get::<datasets::Type>()
                .expect("type is correct")
                .unwrap();
            show || x != datasets::Type::Snapshot
        });
        let model: Option<TreeModel> = Some(filter.upcast());

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
                                    Message::Init
                                }>
                            </TreeView>
                        </ScrolledWindow>
                        <CheckButton label="Show snapshots" on toggled=|btn| { Message::ShowSnapshots(btn.get_active()) } />
                    </Box>
                </ApplicationWindow>
            </Application>
        }
    }
}
