use wasm_encoder::*;
use crate::parser::{Node, Expr};
use crate::map::{ElementMap, SemanticMap};
use serde_json::json;

const DRAW_RECT_FUNC: u32 = 0;
const GC_ALLOC_FUNC: u32 = 1;
const DRAW_TEXT_FUNC: u32 = 2;
const ADD_ROOT_FUNC: u32 = 3;
const TYPE_BOX: i32 = 1;
const TYPE_TEXT: i32 = 2;
const TYPE_GROUP: i32 = 3;


pub fn compile_to_wasm(ast: &[Node]) -> (Vec<u8>, String) {
    let mut module = Module::new();

    // === Type Section ===
    let mut types = TypeSection::new();
    let draw_ui_type = types.len(); types.function([], []);
let draw_rect_type = types.len(); types.function([ValType::I32; 4], []);
let gc_alloc_type = types.len(); types.function([ValType::I32; 2], [ValType::I32]); // fixed!
let draw_text_type = types.len(); types.function([ValType::I32; 4], []);
let add_root_type = types.len(); types.function([ValType::I32], []);
let gc_tick_type = types.len(); types.function([], []);
module.section(&types);

let mut imports = ImportSection::new();
imports.import("env", "memory", EntityType::Memory(MemoryType {
    minimum: 1,
    maximum: None,
    memory64: false, 
    shared: false,
}));
imports.import("env", "drawRect", EntityType::Function(draw_rect_type as u32));
imports.import("env", "gc_alloc", EntityType::Function(gc_alloc_type as u32));
imports.import("env", "drawText", EntityType::Function(draw_text_type as u32));
imports.import("env", "add_root", EntityType::Function(add_root_type as u32));
imports.import("env", "gc_tick", EntityType::Function(gc_tick_type as u32)); // new hook!
module.section(&imports);


    // === Function Section ===
    let mut functions = FunctionSection::new();
    let imported_funcs = 5; // drawRect, gc_alloc, drawText, add_root, gc_tick
    let draw_ui_func_index = imported_funcs as u32; // ✅ This is the first user-defined function
    functions.function(draw_ui_type as u32);
    functions.function(gc_tick_type as u32); // This defines the type of gc_tick
    module.section(&functions);

    // === Export Section ===
    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, draw_ui_func_index);
    let gc_tick_func_index = draw_ui_func_index + 1; // assuming it's the next function
exports.export("gc_tick", ExportKind::Func, gc_tick_func_index);

    module.section(&exports);

    // === Code Section ===
    let mut codes = CodeSection::new();
    let mut draw_ui = Function::new(vec![(1, ValType::I32)]); // local 0 = temp ptr
    let mut map = vec![];
    let mut offset = 0u32;

    println!("📦 Compiling AST:\n{:#?}", ast);
    for node in ast {
    let stack = compile_node(node, &mut draw_ui, &mut map, &mut offset);
    if stack > 0 {
        draw_ui.instruction(&Instruction::Drop);
    }
}

    draw_ui.instruction(&Instruction::End);
codes.function(&draw_ui);
let mut gc_tick_func = Function::new(vec![]);

// Push 2 i32s onto the stack for gc_alloc
gc_tick_func.instruction(&Instruction::I32Const(8)); // example size
gc_tick_func.instruction(&Instruction::I32Const(TYPE_BOX)); // or TYPE_GROUP etc

gc_tick_func.instruction(&Instruction::Call(GC_ALLOC_FUNC));
gc_tick_func.instruction(&Instruction::Drop); // we don't use the pointer

gc_tick_func.instruction(&Instruction::End);
codes.function(&gc_tick_func);




module.section(&codes);


    let wasm = module.finish();
    let semantic = SemanticMap { elements: map };
    let map_json = serde_json::to_string_pretty(&semantic).unwrap();

    (wasm, map_json)
}
fn compile_expr(expr: &Expr, func: &mut Function) {
    match expr {
        Expr::Literal(n) => {
            func.instruction(&Instruction::I32Const(*n));
        }
        Expr::Binary { left, op, right } => {
            compile_expr(left, func);
            compile_expr(right, func);
            match op.as_str() {
                ">" => func.instruction(&Instruction::I32GtS),
                "<" => func.instruction(&Instruction::I32LtS),
                "==" => func.instruction(&Instruction::I32Eq),
                _ => panic!("Unsupported binary op: {}", op),
            };
        }
    }
}

fn compile_expr_and_discard(expr: &Expr, func: &mut Function) {
    compile_expr(expr, func);
    func.instruction(&Instruction::Drop);
}

fn compile_node(
    node: &Node,
    func: &mut Function,
    map: &mut Vec<ElementMap>,
    offset_counter: &mut u32,
) -> i32 {
    match node {
        Node::Group { direction, gap, align, justify, padding, children } => {
    let group_offset = *offset_counter;

    // === GC Allocation for Group ===
    func.instruction(&Instruction::I32Const(8)); // dummy size
    func.instruction(&Instruction::I32Const(TYPE_GROUP));
    func.instruction(&Instruction::Call(GC_ALLOC_FUNC));
    func.instruction(&Instruction::LocalTee(0));
    func.instruction(&Instruction::LocalGet(0));
    func.instruction(&Instruction::Call(ADD_ROOT_FUNC));
    func.instruction(&Instruction::Drop); // drop return of add_root

    // === Layout Pre-Pass ===
    let is_horizontal = direction == "horizontal";
    let mut total_main_size = 0;
    let mut child_dims = vec![]; // (main_size, cross_size)

    for child in children {
        match child {
            Node::Box { width, height, .. } => {
                let main = if is_horizontal { *width } else { *height };
                let cross = if is_horizontal { *height } else { *width };
                total_main_size += main + gap;
                child_dims.push((main, cross));
            }
            Node::Text { value, .. } => {
                let width = value.len() as i32 * 8;
                let height = 16;
                let main = if is_horizontal { width } else { height };
                let cross = if is_horizontal { height } else { width };
                total_main_size += main + gap;
                child_dims.push((main, cross));
            }
            _ => {
                total_main_size += 50 + gap;
                child_dims.push((50, 50));
            }
        }
    }

    if !children.is_empty() {
        total_main_size -= gap;
    }

    // === Justify & Align Calculations ===
    let container_size = 300; // TODO: make dynamic later

    let mut cursor_main = match justify.as_str() {
        "start" => *padding,
        "center" => (container_size - total_main_size) / 2,
        "end" => container_size - total_main_size - *padding,
        "space-between" => *padding,
        _ => *padding,
    };

    let space_between = if justify == "space-between" && children.len() > 1 {
        (container_size - total_main_size + gap * ((children.len() - 1) as i32)) / ((children.len() - 1) as i32)
    } else {
        *gap
    };

    // === Child Rendering Pass ===
    for (i, child) in children.iter().enumerate() {
        let (main_size, cross_size) = child_dims[i];

        let (x, y) = if is_horizontal {
            let x = cursor_main;
            let y = match align.as_str() {
                "start" => *padding,
                "center" => (container_size - cross_size) / 2,
                "end" => container_size - cross_size - *padding,
                _ => *padding,
            };
            (x, y)
        } else {
            let x = match align.as_str() {
                "start" => *padding,
                "center" => (container_size - cross_size) / 2,
                "end" => container_size - cross_size - *padding,
                _ => *padding,
            };
            let y = cursor_main;
            (x, y)
        };

        let rewritten = match child {
            Node::Box { width, height, .. } => Node::Box {
                x,
                y,
                width: *width,
                height: *height,
            },
            Node::Text { value, .. } => Node::Text {
                x,
                y,
                value: value.clone(),
            },
            _ => child.clone(), // fallback
        };

        let stack = compile_node(&rewritten, func, map, offset_counter);
        if stack > 0 {
            func.instruction(&Instruction::Drop);
        }

        cursor_main += main_size + space_between;
    }

    map.push(ElementMap {
        kind: "group".to_string(),
        wasm_offset: group_offset,
        pointer: None,
        source: None,
        props: Some(json!({
            "direction": direction,
            "gap": gap,
            "padding": padding,
            "align": align,
            "justify": justify
        })),
    });

    0
}





        Node::Box { x, y, width, height } => {
            let off = *offset_counter;

            func.instruction(&Instruction::I32Const(0)); *offset_counter += 5; // dummy size
            func.instruction(&Instruction::I32Const(TYPE_BOX)); *offset_counter += 5;
            func.instruction(&Instruction::Call(GC_ALLOC_FUNC)); *offset_counter += 2;
            func.instruction(&Instruction::LocalTee(0)); *offset_counter += 2;

            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::Call(ADD_ROOT_FUNC)); *offset_counter += 2;
            func.instruction(&Instruction::Drop);

            func.instruction(&Instruction::I32Const(*x)); *offset_counter += 5;
            func.instruction(&Instruction::I32Const(*y)); *offset_counter += 5;
            func.instruction(&Instruction::I32Const(*width)); *offset_counter += 5;
            func.instruction(&Instruction::I32Const(*height)); *offset_counter += 5;
            func.instruction(&Instruction::Call(DRAW_RECT_FUNC)); *offset_counter += 2;

            map.push(ElementMap {
                kind: "box".to_string(),
                wasm_offset: off,
                pointer: None,
                source: Some(format!("wpp:x={},y={}", x, y)),
                props: Some(json!({ "x": x, "y": y, "width": width, "height": height })),
            });

            0 // pushes ptr (if reused)
        }

        Node::Text { x, y, value } => {
            let off = *offset_counter;
            let len = value.len() as i32;

            func.instruction(&Instruction::I32Const(len)); *offset_counter += 5;
            func.instruction(&Instruction::I32Const(TYPE_TEXT)); *offset_counter += 5;
            func.instruction(&Instruction::Call(GC_ALLOC_FUNC)); *offset_counter += 2;
            func.instruction(&Instruction::LocalTee(0)); *offset_counter += 2;

            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::Call(ADD_ROOT_FUNC)); *offset_counter += 2;
            func.instruction(&Instruction::Drop);

            for (i, byte) in value.bytes().enumerate() {
                func.instruction(&Instruction::LocalGet(0));
                func.instruction(&Instruction::I32Const(i as i32));
                func.instruction(&Instruction::I32Add);
                func.instruction(&Instruction::I32Const(byte as i32));
                func.instruction(&Instruction::I32Store8(MemArg {
                    align: 0,
                    offset: 0,
                    memory_index: 0,
                }));
                *offset_counter += 8;
            }

            func.instruction(&Instruction::I32Const(*x)); *offset_counter += 5;
            func.instruction(&Instruction::I32Const(*y)); *offset_counter += 5;
            func.instruction(&Instruction::LocalGet(0)); *offset_counter += 2;
            func.instruction(&Instruction::I32Const(len)); *offset_counter += 5;
            func.instruction(&Instruction::Call(DRAW_TEXT_FUNC)); *offset_counter += 2;

            map.push(ElementMap {
                kind: "text".to_string(),
                wasm_offset: off,
                pointer: None,
                source: Some(format!("wpp:text='{}'", value)),
                props: Some(json!({ "x": x, "y": y, "value": value })),
            });

            0 // pushes ptr (but we drop it later)
        }

        Node::Expr(e) => {
            compile_expr_and_discard(e, func);
            0
        }

        Node::If { condition, then_body, else_body } => {
    compile_expr(condition, func);
    func.instruction(&Instruction::If(BlockType::Empty));

    let mut max_stack = 0;

    for stmt in then_body {
        max_stack = max_stack.max(compile_node(stmt, func, map, offset_counter));
    }

    if let Some(else_branch) = else_body {
        func.instruction(&Instruction::Else);
        for stmt in else_branch {
            max_stack = max_stack.max(compile_node(stmt, func, map, offset_counter));
        }
    }

    func.instruction(&Instruction::End);

    if max_stack > 0 {
        func.instruction(&Instruction::Drop);
    }

    0
}


        _ => {
    println!("⚠️ Unhandled node variant: {:?}", node);
    0 // ❌ don’t emit Drop if we didn't push anything!
}

    }
}

