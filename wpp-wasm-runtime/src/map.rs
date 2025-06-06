use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct ElementMap {
    pub kind: String,              // e.g. "box", "button"
    pub wasm_offset: u32,          // Approximate instruction offset
    pub pointer: Option<i32>,      // GC pointer if known
    pub source: Option<String>,    // e.g. "ui.wpp:1"
    pub props: Option<Value>,      // Extra fields like x/y/width/height/text
}

#[derive(Serialize)]
pub struct SemanticMap {
    pub elements: Vec<ElementMap>,
}
