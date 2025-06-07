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

pub fn compile_to_wasm(ast: &[Node]) -> (Vec<u8>, String) {
    let mut module = Module::new();

    // === Type Section ===
    let mut types = TypeSection::new();
    let draw_ui_type = types.len(); types.function([], []);
    let draw_rect_type = types.len(); types.function([ValType::I32; 4], []);
    let gc_alloc_type = types.len(); types.function([ValType::I32], [ValType::I32]);
    let draw_text_type = types.len(); types.function([ValType::I32; 4], []);
    let add_root_type = types.len(); types.function([ValType::I32], []);
    module.section(&types);

    // === Import Section ===
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
    module.section(&imports);

    // === Function Section ===
    let mut functions = FunctionSection::new();
    let draw_ui_func_index = ADD_ROOT_FUNC + 1;
    functions.function(draw_ui_type as u32);
    module.section(&functions);

    // === Export Section ===
    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, draw_ui_func_index);
    module.section(&exports);

    // === Code Section ===
    let mut codes = CodeSection::new();
    let mut draw_ui = Function::new(vec![(1, ValType::I32)]); // local 0 = temp ptr
    let mut map = vec![];
    let mut offset = 0u32;

    println!("📦 Compiling AST:\n{:#?}", ast);
    for node in ast {
        println!("🔧 Compiling node: {:?}", node);
        compile_node(node, &mut draw_ui, &mut map, &mut offset);
    }
    draw_ui.instruction(&Instruction::Drop); // final stack safety
    draw_ui.instruction(&Instruction::End);
    codes.function(&draw_ui);
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
) {
    match node {
        Node::Group(children) => {
    let group_offset = *offset_counter;

    for child in children {
        compile_node(child, func, map, offset_counter);
        func.instruction(&Instruction::Drop); // 🧹 drop result of each child
    }

    map.push(ElementMap {
        kind: "group".to_string(),
        wasm_offset: group_offset,
        pointer: None,
        source: None,
        props: None,
    });
}


        Node::Box { x, y, width, height } => {
            let off = *offset_counter;

            func.instruction(&Instruction::I32Const(TYPE_BOX));
            *offset_counter += 5;
            func.instruction(&Instruction::Call(GC_ALLOC_FUNC));
            *offset_counter += 2;
            func.instruction(&Instruction::LocalTee(0));
            *offset_counter += 2;
            func.instruction(&Instruction::LocalGet(0));
            func.instruction(&Instruction::Call(ADD_ROOT_FUNC));
            *offset_counter += 2;

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
        }

        Node::Print(msg) => {
            eprintln!("🖨️ print('{}') is not supported in WASM yet", msg);
        }

        Node::If { condition, then_body, else_body } => {
    compile_expr(condition, func);
    func.instruction(&Instruction::If(BlockType::Empty));

    for stmt in then_body {
        compile_node(stmt, func, map, offset_counter);
        func.instruction(&Instruction::Drop); // 🧹 ensure clean stack
    }

    if let Some(else_block) = else_body {
        func.instruction(&Instruction::Else);
        for stmt in else_block {
            compile_node(stmt, func, map, offset_counter);
            func.instruction(&Instruction::Drop); // 🧹 ensure clean stack
        }
    }

    func.instruction(&Instruction::End);
}



        Node::Expr(e) => {
            compile_expr_and_discard(e, func);
        }

        _ => {
            println!("⚠️ Unhandled node variant: {:?}", node);
            func.instruction(&Instruction::Drop);
        }
    }
}
