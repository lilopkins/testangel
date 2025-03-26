use std::sync::Arc;

use glib::Object;
use gtk::glib;
use gtk::subclass::prelude::*;
use relm4::gtk;
use relm4::gtk::glib::property::PropertySet;
use sourceview5::CompletionProvider;
use testangel::ipc::EngineList;

mod imp;
mod item;

glib::wrapper! {
    pub struct CompletionProviderEngineInstructions(ObjectSubclass<imp::CompletionProviderEngineInstructions>)
        @implements CompletionProvider;
}

impl CompletionProviderEngineInstructions {
    /// Create a new [`CompletionProvider`] that suggests engine instructions.
    pub fn new(engine_list: Arc<EngineList>) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().engine_list.set(engine_list);
        obj
    }
}
