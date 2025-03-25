use std::cell::{Cell, RefCell};

use glib::subclass::prelude::*;
use relm4::gtk::glib;
use sourceview5::{subclass::prelude::CompletionProposalImpl, CompletionProposal};

use crate::ui::actions::completion_proposal_list::ProposalSource;

#[derive(Debug, Default)]
pub struct EngineCompletionProposal {
    pub(super) engine_lua_name: RefCell<String>,
    pub(super) documentation: RefCell<String>,
    pub(super) source: Cell<ProposalSource>,
}

#[glib::object_subclass]
impl ObjectSubclass for EngineCompletionProposal {
    const NAME: &'static str = "TestAngelEngineCompletionProposal";
    type Type = super::EngineCompletionProposal;
    type ParentType = glib::Object;
    type Interfaces = (CompletionProposal,);
}

impl ObjectImpl for EngineCompletionProposal {}

impl CompletionProposalImpl for EngineCompletionProposal {}
