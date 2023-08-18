use std::{env, fmt, fs, path::PathBuf};

use iced::widget::{
    column, row, Button, Column, Container, PickList, Rule, Scrollable, Text, TextInput,
};
use testangel::types::Action;
use testangel_ipc::prelude::ParameterKind;

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum ActionEditorMessage {
    SaveAction,
    SaveAsAction,
    CloseAction,

    NameChanged(String),
    GroupChanged(String),
    DescriptionChanged(String),

    ParameterCreate,
    ParameterNameChange(usize, String),
    ParameterTypeChange(usize, ParameterKind),
    ParameterMoveUp(usize),
    ParameterMoveDown(usize),
    ParameterDelete(usize),
}

#[derive(Clone, Debug)]
pub enum ActionEditorMessageOut {
    CloseActionEditor,
}

pub enum SaveOrOpenActionError {
    IoError(std::io::Error),
    ParsingError(ron::error::SpannedError),
    SerializingError(ron::Error),
    ActionNotVersionCompatible,
}

impl fmt::Display for SaveOrOpenActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O Error: {e}"),
            Self::ParsingError(e) => write!(f, "Parsing Error: {e}"),
            Self::SerializingError(e) => write!(f, "Serializing error: {e}"),
            Self::ActionNotVersionCompatible => write!(
                f,
                "This action is not compatible with this version of TestAngel."
            ),
        }
    }
}

#[derive(Default)]
pub struct ActionEditor {
    currently_open: Option<Action>,
    current_path: Option<PathBuf>,
    needs_saving: bool,
}

impl ActionEditor {
    /// Create a new action and open it
    pub(crate) fn new_action(&mut self) {
        self.offer_to_save_default_error_handling();
        self.currently_open = Some(Action::default());
        self.current_path = None;
        self.needs_saving = true;
    }

    /// Open an action
    pub(crate) fn open_action(&mut self, file: PathBuf) -> Result<(), SaveOrOpenActionError> {
        self.offer_to_save_default_error_handling();
        let action: Action = ron::from_str(
            &fs::read_to_string(&file).map_err(|e| SaveOrOpenActionError::IoError(e))?,
        )
        .map_err(|e| SaveOrOpenActionError::ParsingError(e))?;
        if action.version() != 1 {
            return Err(SaveOrOpenActionError::ActionNotVersionCompatible);
        }
        self.currently_open = Some(action);
        self.current_path = Some(file);
        self.needs_saving = false;
        Ok(())
    }

    /// Offer to save if it is needed
    fn offer_to_save(&mut self) -> Result<(), SaveOrOpenActionError> {
        if self.needs_saving {
            if rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_title("Do you want to save this action?")
                .set_description("This action has been modified. Do you want to save it?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
            {
                self.save_action(false)?;
            }
        }
        Ok(())
    }

    /// Offer to save if it is needed with default error handling
    fn offer_to_save_default_error_handling(&mut self) {
        if let Err(e) = self.offer_to_save() {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Failed to save action")
                .set_description(&format!("{e}"))
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        }
    }

    /// Save the currently opened action
    fn save_action(&mut self, always_prompt_where: bool) -> Result<(), SaveOrOpenActionError> {
        self.needs_saving = false;
        if always_prompt_where || self.current_path.is_none() {
            // Populate save path
            if let Some(file) = rfd::FileDialog::new()
                .add_filter("TestAngel Actions", &["taaction"])
                .set_title("Save Action")
                .set_directory(
                    env::current_dir()
                        .expect("Failed to read current directory")
                        .join("actions"),
                )
                .save_file()
            {
                self.current_path = Some(file);
            } else {
                return Ok(());
            }
        }

        // Save
        let save_path = self.current_path.as_ref().unwrap();
        let data = ron::to_string(self.currently_open.as_ref().unwrap())
            .map_err(|e| SaveOrOpenActionError::SerializingError(e))?;
        fs::write(save_path, data).map_err(|e| SaveOrOpenActionError::IoError(e))?;
        self.needs_saving = false;
        Ok(())
    }

    /// Close the currently opened action
    fn close_action(&mut self) {
        self.offer_to_save_default_error_handling();
        self.currently_open = None;
        self.current_path = None;
        self.needs_saving = false;
    }

    /// Hint that the open action has been modified.
    fn modified(&mut self) {
        self.needs_saving = true;
    }

    /// Generate the UI for the action parameters
    fn ui_parameters(&self) -> iced::Element<'_, ActionEditorMessage> {
        let mut col = Column::new().spacing(4);
        let action = self.currently_open.as_ref().unwrap();

        for (idx, (name, kind)) in action.parameters.iter().enumerate() {
            col = col.push(
                row![
                    Button::new("×").on_press(ActionEditorMessage::ParameterDelete(idx)),
                    Button::new("ʌ").on_press_maybe(if idx == 0 {
                        None
                    } else {
                        Some(ActionEditorMessage::ParameterMoveUp(idx))
                    }),
                    Button::new("v").on_press_maybe(if (idx + 1) == action.parameters.len() {
                        None
                    } else {
                        Some(ActionEditorMessage::ParameterMoveDown(idx))
                    }),
                    TextInput::new("Parameter Name", name)
                        .on_input(move |s| ActionEditorMessage::ParameterNameChange(idx, s)),
                    PickList::new(
                        &[
                            ParameterKind::String,
                            ParameterKind::Integer,
                            ParameterKind::Decimal,
                        ][..],
                        Some(kind.clone()),
                        move |k| ActionEditorMessage::ParameterTypeChange(idx, k)
                    )
                    .placeholder("Parameter Kind"),
                ]
                .spacing(4)
                .align_items(iced::Alignment::Center),
            );
        }

        col.into()
    }

    fn ui_steps(&self) -> iced::Element<'_, ActionEditorMessage> {
        let mut col = Column::new();

        col.into()
    }

    fn ui_outputs(&self) -> iced::Element<'_, ActionEditorMessage> {
        let mut col = Column::new();

        col.into()
    }
}

impl UiComponent for ActionEditor {
    type Message = ActionEditorMessage;
    type MessageOut = ActionEditorMessageOut;

    fn title(&self) -> Option<&str> {
        Some("Action Editor")
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let action = self
            .currently_open
            .as_ref()
            .expect("the action editor can't be open with no open action");

        Scrollable::new(
            Container::new(
                column![
                    // Toolbar
                    row![
                        Button::new("Save").on_press(ActionEditorMessage::SaveAction),
                        Button::new("Save as").on_press(ActionEditorMessage::SaveAsAction),
                        Button::new("Close Action").on_press(ActionEditorMessage::CloseAction),
                    ]
                    .spacing(8),
                    // Metadata
                    TextInput::new("Action Name", &action.friendly_name)
                        .on_input(ActionEditorMessage::NameChanged),
                    TextInput::new("Action Group", &action.group)
                        .on_input(ActionEditorMessage::GroupChanged),
                    TextInput::new("Description", &action.description)
                        .on_input(ActionEditorMessage::DescriptionChanged),
                    Rule::horizontal(2),
                    // Parameters
                    Text::new("Action Parameters"),
                    self.ui_parameters(),
                    Button::new("+ New parameter").on_press(ActionEditorMessage::ParameterCreate),
                    Rule::horizontal(2),
                    // Instructions
                    Text::new("Steps"),
                    self.ui_steps(),
                    Button::new("+ Add step"),
                    Rule::horizontal(2),
                    // Outputs
                    Text::new("Action Outputs"),
                    self.ui_outputs(),
                    Button::new("+ New output"),
                ]
                .spacing(8),
            )
            .padding(16),
        )
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Option<Self::MessageOut> {
        match message {
            ActionEditorMessage::SaveAction => {
                if let Err(e) = self.save_action(false) {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Failed to save action")
                        .set_description(&format!("{e}"))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
            }
            ActionEditorMessage::SaveAsAction => {
                if let Err(e) = self.save_action(true) {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Failed to save action")
                        .set_description(&format!("{e}"))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
            }
            ActionEditorMessage::CloseAction => {
                self.close_action();
                return Some(ActionEditorMessageOut::CloseActionEditor);
            }

            ActionEditorMessage::NameChanged(new_name) => {
                self.currently_open.as_mut().unwrap().friendly_name = new_name;
                self.modified();
            }
            ActionEditorMessage::GroupChanged(new_group) => {
                self.currently_open.as_mut().unwrap().group = new_group;
                self.modified();
            }
            ActionEditorMessage::DescriptionChanged(new_description) => {
                self.currently_open.as_mut().unwrap().description = new_description;
                self.modified();
            }
            ActionEditorMessage::ParameterTypeChange(idx, new_type) => {
                let (name, _) = &self.currently_open.as_mut().unwrap().parameters[idx];
                self.currently_open.as_mut().unwrap().parameters[idx] = (name.clone(), new_type);
                self.modified();
            }
            ActionEditorMessage::ParameterNameChange(idx, new_name) => {
                let (_, kind) = &self.currently_open.as_mut().unwrap().parameters[idx];
                self.currently_open.as_mut().unwrap().parameters[idx] = (new_name, kind.clone());
                self.modified();
            }
            ActionEditorMessage::ParameterCreate => {
                self.currently_open
                    .as_mut()
                    .unwrap()
                    .parameters
                    .push((String::new(), ParameterKind::String));
                self.modified();
            }
            ActionEditorMessage::ParameterMoveUp(idx) => {
                let params = &mut self.currently_open.as_mut().unwrap().parameters;
                let val = params.remove(idx);
                params.insert((idx - 1).max(0), val);
                // TODO Renumber
                self.modified();
            }
            ActionEditorMessage::ParameterMoveDown(idx) => {
                let params = &mut self.currently_open.as_mut().unwrap().parameters;
                let val = params.remove(idx);
                params.insert((idx + 1).min(params.len()), val);
                // TODO Renumber
                self.modified();
            }
            ActionEditorMessage::ParameterDelete(idx) => {
                self.currently_open.as_mut().unwrap().parameters.remove(idx);
                // TODO Renumber
                self.modified();
            }
        };
        None
    }
}
