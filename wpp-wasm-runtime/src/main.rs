use wasm_encoder::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct BoxNode {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn main() {
    // Load layout from JSON
    let json = std::fs::read_to_string("../out.box.json").expect("Failed to read JSON file");
    let boxes: Vec<BoxNode> = serde_json::from_str(&json).expect("Invalid JSON");

    let mut module = Module::new();

    // === Type Section ===
    let mut types = TypeSection::new();
    let draw_ui_type = types.len();
    types.function([], []); // draw_ui: () -> void

    let draw_rect_type = types.len();
    types.function([ValType::I32, ValType::I32, ValType::I32, ValType::I32], []); // drawRect(x, y, w, h)

    let gc_alloc_type = types.len();
    types.function([ValType::I32], [ValType::I32]); // gc_alloc(size) -> ptr

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
    let draw_ui_func_index = draw_rect_func + 2; // after 2 imports
    functions.function(draw_ui_type as u32);
    module.section(&functions);

    // === Export Section ===
    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, draw_ui_func_index);
    module.section(&exports);

    // === Code Section ===
    let mut codes = CodeSection::new();
    let mut draw_ui_func = Function::new(vec![]); // no locals

    for b in boxes {
        // GC allocation (simulate allocating an object)
        draw_ui_func.instruction(&Instruction::I32Const(64)); // size in bytes
        draw_ui_func.instruction(&Instruction::Call(gc_alloc_func)); // gc_alloc(size)
        draw_ui_func.instruction(&Instruction::Drop); // discard pointer for now

        // Draw rectangle
        draw_ui_func.instruction(&Instruction::I32Const(b.x));
        draw_ui_func.instruction(&Instruction::I32Const(b.y));
        draw_ui_func.instruction(&Instruction::I32Const(b.width));
        draw_ui_func.instruction(&Instruction::I32Const(b.height));
        draw_ui_func.instruction(&Instruction::Call(draw_rect_func));
    }

    draw_ui_func.instruction(&Instruction::End);
    codes.function(&draw_ui_func);
    module.section(&codes);

    // === Write WASM File ===
    std::fs::write("../ui.wasm", module.finish()).expect("Failed to write ui.wasm");
    println!("✅ Wrote ui.wasm from out.box.json with GC support");
}
