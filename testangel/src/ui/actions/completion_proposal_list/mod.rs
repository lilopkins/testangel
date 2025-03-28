use std::cmp::Ordering;

use relm4::gtk::{gio, glib, prelude::*, subclass::prelude::*};
use sourceview5::CompletionProposal;

mod imp;

// Public part of the FileFilterListModel type.
glib::wrapper! {
    pub struct CompletionProposalListModel(ObjectSubclass<imp::CompletionProposalListModel>)
        @implements gio::ListModel;
}

// Constructor for new instances. This simply calls glib::Object::new()
impl CompletionProposalListModel {
    pub fn new() -> CompletionProposalListModel {
        glib::Object::new()
    }

    pub fn append(&self, obj: impl IsA<CompletionProposal>) {
        let imp = self.imp();
        let index = {
            // Borrow the data only once and ensure the borrow guard is dropped
            // before we emit the items_changed signal because the view
            // could call get_item / get_n_item from the signal handler to update its state
            let mut data = imp.inner.borrow_mut();
            data.push(obj.upcast());
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

    pub fn retain<F>(&self, retain_fn: F)
    where
        F: Fn(&CompletionProposal) -> bool,
    {
        let mut list = self.imp().inner.borrow().clone();
        let mut remove_indexes = vec![];
        for (idx, item) in list.iter().enumerate() {
            if !retain_fn(item) {
                remove_indexes.push(idx);
            }
        }
        list.retain(retain_fn);
        self.imp().inner.replace(list);
        for (offset, index) in remove_indexes.iter().enumerate() {
            self.items_changed(u32::try_from(*index - offset).unwrap(), 1, 0);
        }
    }

    pub fn sort<F>(&self, compare: F)
    where
        F: FnMut(&CompletionProposal, &CompletionProposal) -> Ordering,
    {
        let mut list = self.imp().inner.borrow_mut();
        list.sort_by(compare);
    }
}

impl Default for CompletionProposalListModel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProposalSource {
    #[default]
    Exact,
    Fuzzy {
        score: i64,
    },
}

impl ProposalSource {
    fn numerical_value(&self) -> i64 {
        match self {
            Self::Exact => i64::MIN,
            Self::Fuzzy { score } => *score,
        }
    }
}

impl PartialOrd for ProposalSource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ProposalSource {
    fn cmp(&self, other: &Self) -> Ordering {
        self.numerical_value().cmp(&other.numerical_value())
    }
}
