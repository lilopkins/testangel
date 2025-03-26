use testangel_engine::{engine, Evidence as Ev, EvidenceContent};

engine! {
    /// Work with evidence.
    #[engine(
        version = env!("CARGO_PKG_VERSION"),
    )]
    #[derive(Default)]
    struct Evidence;

    impl Evidence {
        #[instruction(
            id = "evidence-add-text",
            name = "Add Text-based Evidence",
            lua_name = "AddText",
            flags = InstructionFlags::PURE | InstructionFlags::INFALLIBLE | InstructionFlags::AUTOMATIC,
        )]
        /// Add text based evidence to the report.
        fn add_text(
            label: String,
            content: String,
        ) {
            evidence.push(Ev {
                label,
                content: EvidenceContent::Textual(content),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use testangel_engine::iwp;

    use super::*;

    #[test]
    fn test_add_text() {
        let mut engine = EVIDENCE_ENGINE.lock().unwrap();
        let (_output, evidence) = engine
            .run_instruction(
                iwp!("evidence-add-text", false, "label" => "Test Label", "content" => "Content"),
            )
            .expect("Failed to trigger instruction");
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0].label, "Test Label".to_owned());
        assert_eq!(
            evidence[0].content,
            EvidenceContent::Textual("Content".to_owned())
        );
    }
}
