mod parser;
mod transpile;
mod map;
mod gc;
mod memory;

use std::fs;
use transpile::compile_to_wasm;
use parser::{Node, parse_wpp};
use gc::gc_collect;

/// Optional GC export to allow JS to trigger cleanup.
#[no_mangle]
pub extern "C" fn gc_tick() {
    gc_collect();
    println!("🧹 GC tick called from WASM runtime");
}

fn main() {
    // Step 1: Read W++ source file
    let source = fs::read_to_string("ui.wpp").expect("❌ Failed to read ui.wpp");

    // Step 2: Parse W++ source into AST
    let ast: Vec<Node> = parse_wpp(&source);
    println!("✅ Parsed W++ source with {} root nodes", ast.len());

    // Step 3: Transpile AST to WASM + semantic map
    let (wasm_bytes, semantic_map_json) = compile_to_wasm(&ast);

    // Step 4: Write output files
    fs::write("ui.wasm", wasm_bytes).expect("❌ Failed to write ui.wasm");
    fs::write("ui.wpp.map.json", semantic_map_json).expect("❌ Failed to write semantic map");

    println!("✅ Compilation complete: ui.wasm & ui.wpp.map.json");

    // Step 5: Optional GC collection pass after compile
    gc_collect();
    println!("🧹 GC run complete");
}
