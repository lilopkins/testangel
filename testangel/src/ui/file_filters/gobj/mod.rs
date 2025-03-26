use relm4::gtk::{FileFilter, gio, glib, prelude::*, subclass::prelude::*};

mod imp;

// Public part of the FileFilterListModel type.
glib::wrapper! {
    pub struct FileFilterListModel(ObjectSubclass<imp::FileFilterListModel>)
        @implements gio::ListModel;
}

// Constructor for new instances. This simply calls glib::Object::new()
impl FileFilterListModel {
    pub fn new() -> FileFilterListModel {
        glib::Object::new()
    }

    pub fn append(&self, obj: FileFilter) {
        let imp = self.imp();
        let index = {
            // Borrow the data only once and ensure the borrow guard is dropped
            // before we emit the items_changed signal because the view
            // could call get_item / get_n_item from the signal handler to update its state
            let mut data = imp.inner.borrow_mut();
            data.push(obj);
            data.len() - 1
        };
        // Emits a signal that 1 item was added, 0 removed at the position index
        self.items_changed(u32::try_from(index).unwrap(), 0, 1);
    }

    pub fn remove(&self, index: u32) {
        let imp = self.imp();
        imp.inner.borrow_mut().remove(index as usize);
        // Emits a signal that 1 item was removed, 0 added at the position index
        self.items_changed(index, 1, 0);
    }
}

impl Default for FileFilterListModel {
    fn default() -> Self {
        Self::new()
    }
}
