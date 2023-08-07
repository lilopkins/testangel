use serde::{Deserialize, Serialize};

/// A piece of evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    /// The label for this piece of evidence.
    pub label: String,
    /// The contents of the evidence.
    pub content: EvidenceContent,
}

/// Content of some evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum EvidenceContent {
    /// Textual content
    Textual(String),
    /// A PNG encoded image encoded as a base64 string.
    ImageAsPngBase64(String),
}
