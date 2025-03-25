use glib::subclass::prelude::*;
use relm4::gtk::glib;
use sourceview5::CompletionProposal;

mod imp;

glib::wrapper! {
    pub struct EngineInstructionCompletionProposal(ObjectSubclass<imp::EngineInstructionCompletionProposal>)
        @implements CompletionProposal;
}

impl EngineInstructionCompletionProposal {
    /// Create a new proposal.
    pub fn new(
        engine_lua_name: String,
        instruction_lua_name: String,
        documentation: String,
        parameters: Vec<String>,
        returns: Vec<String>,
    ) -> Self {
        let o: EngineInstructionCompletionProposal = glib::Object::builder().build();
        o.imp().engine_lua_name.replace(engine_lua_name);
        o.imp().instruction_lua_name.replace(instruction_lua_name);
        o.imp().documentation.replace(documentation);
        o.imp().parameters.replace(parameters);
        o.imp().returns.replace(returns);
        o
    }

    pub fn engine_lua_name(&self) -> String {
        let imp::EngineInstructionCompletionProposal {
            engine_lua_name, ..
        } = self.imp();
        engine_lua_name.borrow().clone()
    }

    pub fn instruction_lua_name(&self) -> String {
        let imp::EngineInstructionCompletionProposal {
            instruction_lua_name,
            ..
        } = self.imp();
        instruction_lua_name.borrow().clone()
    }

    pub fn documentation(&self) -> String {
        let imp::EngineInstructionCompletionProposal { documentation, .. } = self.imp();
        documentation.borrow().clone()
    }

    pub fn parameters(&self) -> Vec<String> {
        let imp::EngineInstructionCompletionProposal { parameters, .. } = self.imp();
        parameters.borrow().clone()
    }

    pub fn returns(&self) -> Vec<String> {
        let imp::EngineInstructionCompletionProposal { returns, .. } = self.imp();
        returns.borrow().clone()
    }
}
