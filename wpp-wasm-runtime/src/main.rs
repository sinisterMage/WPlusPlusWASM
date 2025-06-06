use wasm_encoder::*;
use serde::Deserialize;
use std::fs;
mod map;
use map::{ElementMap, SemanticMap};

#[derive(Deserialize, Debug)]
#[serde(tag = "Kind")]
pub enum Node {
    Box {
        X: i32,
        Y: i32,
        Width: i32,
        Height: i32,
    },
    Group {
        Children: Vec<Node>,
    },
}

fn emit_node(
    node: &Node,
    func: &mut Function,
    map: &mut Vec<ElementMap>,
    offset: &mut u32,
    draw_rect_func: u32,
    gc_alloc_func: u32,
) {
    match node {
        Node::Box { X, Y, Width, Height } => {
            let start_offset = *offset;

            // Simulate object allocation
            func.instruction(&Instruction::I32Const(64)); *offset += 5;
            func.instruction(&Instruction::Call(gc_alloc_func)); *offset += 2;
            func.instruction(&Instruction::Drop); *offset += 1;

            // Draw box
            func.instruction(&Instruction::I32Const(*X)); *offset += 5;
            func.instruction(&Instruction::I32Const(*Y)); *offset += 5;
            func.instruction(&Instruction::I32Const(*Width)); *offset += 5;
            func.instruction(&Instruction::I32Const(*Height)); *offset += 5;
            func.instruction(&Instruction::Call(draw_rect_func)); *offset += 2;

            map.push(ElementMap {
                kind: "box".to_string(),
                wasm_offset: start_offset,
                pointer: None,
                source: Some(format!("ui.wpp:x={},y={}", X, Y)),
                props: Some(serde_json::json!({
                    "x": X,
                    "y": Y,
                    "width": Width,
                    "height": Height
                })),
            });
        }
        Node::Group { Children } => {
            let group_offset = *offset;

            for child in Children {
                emit_node(child, func, map, offset, draw_rect_func, gc_alloc_func);
            }

            map.push(ElementMap {
                kind: "group".to_string(),
                wasm_offset: group_offset,
                pointer: None,
                source: None,
                props: None,
            });
        }
    }
}

fn main() {
    let json = fs::read_to_string("../out.box.json").expect("❌ Failed to read JSON");
    let nodes: Vec<Node> = serde_json::from_str(&json).expect("❌ Invalid JSON format");

    let mut module = Module::new();

    // === Type Section ===
    let mut types = TypeSection::new();
    let draw_ui_type = types.len();
    types.function([], []);

    let draw_rect_type = types.len();
    types.function([ValType::I32, ValType::I32, ValType::I32, ValType::I32], []);

    let gc_alloc_type = types.len();
    types.function([ValType::I32], [ValType::I32]);

    module.section(&types);

    // === Import Section ===
    let mut imports = ImportSection::new();
    let draw_rect_func = 0;
    let gc_alloc_func = 1;

    imports.import("env", "drawRect", EntityType::Function(draw_rect_type as u32));
    imports.import("env", "gc_alloc", EntityType::Function(gc_alloc_type as u32));
    module.section(&imports);

    // === Function Section ===
    let mut functions = FunctionSection::new();
    let draw_ui_func_index = draw_rect_func + 2;
    functions.function(draw_ui_type as u32);
    module.section(&functions);

    // === Export Section ===
    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, draw_ui_func_index);
    module.section(&exports);

    // === Code Section ===
    let mut codes = CodeSection::new();
    let mut draw_ui_func = Function::new(vec![]);
    let mut semantic_map = vec![];

    let mut offset_counter: u32 = 0;

    for node in &nodes {
        emit_node(
            node,
            &mut draw_ui_func,
            &mut semantic_map,
            &mut offset_counter,
            draw_rect_func,
            gc_alloc_func,
        );
    }

    draw_ui_func.instruction(&Instruction::End);
    codes.function(&draw_ui_func);
    module.section(&codes);

    // === Write .wasm File ===
    fs::write("../ui.wasm", module.finish()).expect("❌ Failed to write WASM");
    println!("✅ Wrote ui.wasm from out.box.json");

    // === Write .map.json File ===
    let semantic = SemanticMap { elements: semantic_map };
    let map_json = serde_json::to_string_pretty(&semantic).unwrap();
    fs::write("../ui.wpp.map.json", map_json).expect("❌ Failed to write .map file");
    println!("🧠 Wrote ui.wpp.map.json for devtools");
}
