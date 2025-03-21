use testangel_engine::{engine, Evidence as Ev, EvidenceContent};

engine! {
    /// Work with evidence.
    #[engine(
        version = env!("CARGO_PKG_VERSION"),
    )]
    struct Evidence;

    impl Evidence {
        #[instruction(id = "evidence-add-text", name = "Add Text-based Evidence", lua_name = "AddText")]
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
