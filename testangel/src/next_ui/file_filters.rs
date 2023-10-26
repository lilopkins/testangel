use std::cell::RefCell;

use relm4::gtk::{self, FileFilter, glib, gio, subclass::prelude::*, prelude::*};

use super::lang;

/// Get a [`FileFilter`] tuned to all files.
pub fn all() -> FileFilter {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(&lang::lookup("filetype-all")));
    filter.add_pattern("*");
    filter
}

/// Get a [`FileFilter`] tuned to flows.
pub fn flows() -> FileFilter {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(&lang::lookup("filetype-flow")));
    filter.add_suffix("taflow");
    filter
}

/// Get a [`FileFilter`] tuned to PDFs.
pub fn pdfs() -> FileFilter {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(&lang::lookup("filetype-pdf")));
    filter.add_suffix("pdf");
    filter.add_mime_type("application/pdf");
    filter
}

/// Create a [`FileFilterListModel`] containing the provided list of [`FileFilter`]s.
pub fn filter_list(filters: Vec<FileFilter>) -> FileFilterListModel {
    let model = FileFilterListModel::new();
    for filter in filters {
        model.append(&filter);
    }
    model
}

// Public part of the FileFilterListModel type.
glib::wrapper! {
    pub struct FileFilterListModel(ObjectSubclass<InnerFileFilterListModel>) @implements gio::ListModel;
}

// Constructor for new instances. This simply calls glib::Object::new()
impl FileFilterListModel {
    pub fn new() -> FileFilterListModel {
        glib::Object::new()
    }

    pub fn append(&self, obj: &FileFilter) {
        let imp = self.imp();
        let index = {
            // Borrow the data only once and ensure the borrow guard is dropped
            // before we emit the items_changed signal because the view
            // could call get_item / get_n_item from the signal handler to update its state
            let mut data = imp.0.borrow_mut();
            data.push(obj.clone());
            data.len() - 1
        };
        // Emits a signal that 1 item was added, 0 removed at the position index
        self.items_changed(index as u32, 0, 1);
    }

    pub fn remove(&self, index: u32) {
        let imp = self.imp();
        imp.0.borrow_mut().remove(index as usize);
        // Emits a signal that 1 item was removed, 0 added at the position index
        self.items_changed(index, 1, 0);
    }
}

impl Default for FileFilterListModel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default)]
pub struct InnerFileFilterListModel(pub(super) RefCell<Vec<FileFilter>>);

/// Basic declaration of our type for the GObject type system
#[glib::object_subclass]
impl ObjectSubclass for InnerFileFilterListModel {
    const NAME: &'static str = "Model";
    type Type = FileFilterListModel;
    type Interfaces = (gio::ListModel,);
}

impl ObjectImpl for InnerFileFilterListModel {}

impl ListModelImpl for InnerFileFilterListModel {
    fn item_type(&self) -> glib::Type {
        FileFilter::static_type()
    }

    fn n_items(&self) -> u32 {
        self.0.borrow().len() as u32
    }

    fn item(&self, position: u32) -> Option<glib::Object> {
        self.0
            .borrow()
            .get(position as usize)
            .map(|o| o.clone().upcast::<glib::Object>())
    }
}
