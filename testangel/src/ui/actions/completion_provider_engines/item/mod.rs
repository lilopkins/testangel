use glib::subclass::prelude::*;
use relm4::gtk::glib::{self, property::PropertySet};
use sourceview5::CompletionProposal;
use testangel::ipc::Engine;

mod imp;

glib::wrapper! {
    pub struct EngineCompletionProposal(ObjectSubclass<imp::EngineCompletionProposal>)
        @implements CompletionProposal;
}

impl EngineCompletionProposal {
    /// Create a new proposal.
    pub fn new(engine: &Engine) -> Self {
        let o: EngineCompletionProposal = glib::Object::builder().build();
        o.imp().engine_lua_name.set(engine.lua_name.clone());
        o.imp().documentation.set(engine.description.clone());
        o
    }

    /// Generate the lua name for the engine for this proposal.
    pub fn engine_lua_name(&self) -> String {
        let imp::EngineCompletionProposal {
            engine_lua_name, ..
        } = self.imp();
        engine_lua_name.borrow().clone()
    }

    /// Generate the documentation for the engine for this proposal.
    pub fn documentation(&self) -> String {
        let imp::EngineCompletionProposal { documentation, .. } = self.imp();
        documentation.borrow().clone()
    }
}
