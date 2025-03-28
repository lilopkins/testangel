use testangel_engine::engine;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowTermination {
    #[error("The user terminated the flow.")]
    UserTerminated,
    #[error("An automation step terminated the flow.")]
    StepTerminated,
}

engine! {
    /// Interact with the user, ask them questions and tell them things.
    #[engine(
        lua_name = "Interaction",
        version = env!("CARGO_PKG_VERSION"),
    )]
    #[derive(Default)]
    struct UserInteraction;

    impl UserInteraction {
        #[instruction(
            id = "user-interaction-wait",
            name = "Wait for OK",
            lua_name = "WaitForOK",
            flags = InstructionFlags::INFALLIBLE,
        )]
        /// Display a message dialog and don't continue running the test flow
        /// until the user presses 'OK'.
        fn wait(
            message: String,
        ) {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_buttons(rfd::MessageButtons::Ok)
                .set_title("Information")
                .set_description(message)
                .show();
        }

        #[instruction(
            id = "user-interaction-ask",
            name = "Yes/No Question",
            lua_name = "AskYesNo",
            flags = InstructionFlags::INFALLIBLE,
        )]
        /// Returns a boolean if the input text matches a regular expression.
        fn ask(
            message: String,
        ) -> #[output(id = "response", name = "Response")] bool {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_buttons(rfd::MessageButtons::YesNo)
                .set_title("Question")
                .set_description(message)
                .show() == rfd::MessageDialogResult::Yes
        }

        #[instruction(
            id = "user-interaction-ask-continue",
            name = "Ask to Continue Flow",
            lua_name = "AskToContinue",
            flags = InstructionFlags::NONE,
        )]
        /// Ask the user if they want to continue the automation flow.
        fn ask_continue(
            message: String,
        ) {
            if rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_buttons(rfd::MessageButtons::YesNo)
                .set_title("Continue flow?")
                .set_description(message)
                .show() == rfd::MessageDialogResult::No
            {
                log(TA_LOG_INFO, "Flow terminating due to user input");
                Err::<(), FlowTermination>(FlowTermination::UserTerminated)?;
            }
        }

        #[instruction(
            id = "user-interaction-terminate-flow",
            name = "Terminate Flow",
            lua_name = "TerminateFlow",
            flags = InstructionFlags::NONE,
        )]
        /// Let the user know that the flow has been stopped for a reason.
        fn terminate_flow(
            message: String,
        ) {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_buttons(rfd::MessageButtons::Ok)
                .set_title("Flow Terminating")
                .set_description(message)
                .show();

            log(TA_LOG_INFO, "Flow terminating due to triggered instruction");
            Err::<(), FlowTermination>(FlowTermination::StepTerminated)?;
        }
    }
}
