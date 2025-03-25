use std::cell::RefCell;

use glib::subclass::prelude::*;
use relm4::gtk::glib;
use sourceview5::{subclass::prelude::CompletionProposalImpl, CompletionProposal};

#[derive(Debug, Default)]
pub struct EngineInstructionCompletionProposal {
    pub(super) engine_lua_name: RefCell<String>,
    pub(super) instruction_lua_name: RefCell<String>,
    pub(super) documentation: RefCell<String>,
    pub(super) parameters: RefCell<Vec<String>>,
    pub(super) returns: RefCell<Vec<String>>,
}

#[glib::object_subclass]
impl ObjectSubclass for EngineInstructionCompletionProposal {
    const NAME: &'static str = "TestAngelEngineInstructionCompletionProposal";
    type Type = super::EngineInstructionCompletionProposal;
    type ParentType = glib::Object;
    type Interfaces = (CompletionProposal,);
}

impl ObjectImpl for EngineInstructionCompletionProposal {}

impl CompletionProposalImpl for EngineInstructionCompletionProposal {}
