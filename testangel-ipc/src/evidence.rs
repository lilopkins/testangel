use serde::{Deserialize, Serialize};
#[cfg(feature="schemas")] use schemars::JsonSchema;

/// A piece of evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct Evidence {
    /// The label for this piece of evidence.
    pub label: String,
    /// The contents of the evidence.
    pub content: EvidenceContent,
}

/// Content of some evidence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[serde(tag = "type", content = "content")]
pub enum EvidenceContent {
    /// Textual content
    Textual(String),
    /// A PNG encoded image encoded as a base64 string.
    ImageAsPngBase64(String),
}
