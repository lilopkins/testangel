use std::{cell::RefCell, sync::Arc};

use convert_case::Casing;
use glib::prelude::*;
use gtk::subclass::prelude::*;
use relm4::gtk::{self, gio::ListModel, glib};
use sourceview5::{prelude::TextBufferExt, subclass::prelude::*, CompletionProvider};
use testangel::ipc::EngineList;

use crate::ui::actions::completion_proposal_list::CompletionProposalListModel;

use super::item::EngineInstructionCompletionProposal;

#[derive(Default)]
pub struct CompletionProviderEngineInstructions {
    pub(super) engine_list: RefCell<Arc<EngineList>>,
}

#[glib::object_subclass]
impl ObjectSubclass for CompletionProviderEngineInstructions {
    const NAME: &'static str = "TestAngelCompletionProviderEngineInstructions";
    type Type = super::CompletionProviderEngineInstructions;
    type ParentType = glib::Object;
    type Interfaces = (CompletionProvider,);
}

impl ObjectImpl for CompletionProviderEngineInstructions {}

impl CompletionProviderImpl for CompletionProviderEngineInstructions {
    fn activate(
        &self,
        context: &sourceview5::CompletionContext,
        proposal: &sourceview5::CompletionProposal,
    ) {
        if let Ok(proposal) = proposal
            .clone()
            .downcast::<EngineInstructionCompletionProposal>()
        {
            if let Some((mut start, mut end)) = context.bounds() {
                let buffer = start.buffer();
                let instruction_lua_name = proposal.instruction_lua_name();
                let mut end_mark = None;

                // If the insertion cursor is within a word and the trailing
                // characters of the word match the suffix of the proposal, then
                // limit how much text we insert so that the word is completed
                // properly.
                if !end.ends_line() && !end.char().is_whitespace() && !end.ends_word() {
                    let mut word_end = end;
                    if word_end.forward_word_end() {
                        let text = end.slice(&word_end).to_string();

                        if instruction_lua_name.ends_with(&text) {
                            assert!(instruction_lua_name.len() >= text.len());
                            end_mark = Some(buffer.create_mark(None, &word_end, false));
                        }
                    }
                }

                buffer.begin_user_action();
                buffer.delete(&mut start, &mut end);
                buffer.insert(&mut start, &instruction_lua_name);
                buffer.end_user_action();

                if let Some(end_mark) = end_mark {
                    let new_end = buffer.iter_at_mark(&end_mark);
                    buffer.select_range(&new_end, &new_end);
                    buffer.delete_mark(&end_mark);
                }
            }
        }
    }

    fn display(
        &self,
        _context: &sourceview5::CompletionContext,
        proposal: &sourceview5::CompletionProposal,
        cell: &sourceview5::CompletionCell,
    ) {
        if let Ok(proposal) = proposal
            .clone()
            .downcast::<EngineInstructionCompletionProposal>()
        {
            match cell.column() {
                sourceview5::CompletionColumn::Icon => {
                    cell.set_icon_name(relm4_icons::icon_names::PUZZLE_PIECE);
                }
                sourceview5::CompletionColumn::Before => {
                    cell.set_text(Some(&proposal.engine_lua_name()));
                }
                sourceview5::CompletionColumn::TypedText => {
                    cell.set_text(Some(&proposal.instruction_lua_name()));
                }
                sourceview5::CompletionColumn::Details => {
                    cell.set_text(Some(&proposal.documentation()));
                }
                _ => (),
            }
        }
    }

    fn title(&self) -> Option<glib::GString> {
        None
    }

    fn priority(&self, _context: &sourceview5::CompletionContext) -> i32 {
        0
    }

    fn is_trigger(&self, _iter: &gtk::TextIter, c: char) -> bool {
        c == '.'
    }

    fn key_activates(
        &self,
        _context: &sourceview5::CompletionContext,
        _proposal: &sourceview5::CompletionProposal,
        _keyval: gtk::gdk::Key,
        _state: gtk::gdk::ModifierType,
    ) -> bool {
        false
    }

    fn refilter(&self, context: &sourceview5::CompletionContext, model: &gtk::gio::ListModel) {
        let word = context.word().to_string();
        if let Ok(model) = model.clone().downcast::<CompletionProposalListModel>() {
            model.retain(|item| {
                item.clone()
                    .downcast::<EngineInstructionCompletionProposal>()
                    .is_ok_and(|item| {
                        item.instruction_lua_name()
                            .to_ascii_lowercase()
                            .starts_with(&word.to_ascii_lowercase())
                    })
            });
        }
    }

    fn list_alternates(
        &self,
        _context: &sourceview5::CompletionContext,
        _proposal: &sourceview5::CompletionProposal,
    ) -> Vec<sourceview5::CompletionProposal> {
        vec![]
    }

    fn populate_future(
        &self,
        context: &sourceview5::CompletionContext,
    ) -> std::pin::Pin<
        Box<dyn std::prelude::rust_2024::Future<Output = Result<gtk::gio::ListModel, glib::Error>>>,
    > {
        let engines_box = self.engine_list.borrow();
        let engines_arc = (*engines_box).clone();

        let context = context.clone();
        Box::pin(async move {
            let engines = engines_arc;
            let list = CompletionProposalListModel::new();
            if let Some((mut start, mut end)) = context.bounds() {
                let word = context.word().to_string();
                let buffer = start.buffer();
                while !start.starts_line() {
                    start.backward_char();
                }
                while !end.ends_line() {
                    end.forward_char();
                }

                let line = buffer.slice(&start, &end, false).to_string();
                if let Some(engine_lua_name) = line.trim().split('.').next() {
                    for engine in &**engines {
                        if engine.lua_name == engine_lua_name {
                            for instruction in &engine.instructions {
                                if instruction
                                    .lua_name()
                                    .to_ascii_lowercase()
                                    .starts_with(&word.to_ascii_lowercase())
                                {
                                    list.append(EngineInstructionCompletionProposal::new(
                                        engine_lua_name.to_owned(),
                                        instruction.lua_name().clone(),
                                        instruction.description().clone(),
                                        instruction
                                            .parameters()
                                            .iter()
                                            .map(|p| {
                                                p.friendly_name().to_case(convert_case::Case::Snake)
                                            })
                                            .collect(),
                                        instruction
                                            .outputs()
                                            .iter()
                                            .map(|p| {
                                                p.friendly_name().to_case(convert_case::Case::Snake)
                                            })
                                            .collect(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            Ok(list.upcast::<ListModel>())
        })
    }
}
