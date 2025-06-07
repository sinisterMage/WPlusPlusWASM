use wasm_encoder::*;
use crate::parser::{Node, Expr};
use crate::map::{ElementMap, SemanticMap};
use serde_json::json;



pub fn compile_to_wasm(ast: &[Node]) -> (Vec<u8>, String) {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    let draw_ui_type = types.len();
    types.function([], []);
    let draw_rect_type = types.len();
    types.function([ValType::I32, ValType::I32, ValType::I32, ValType::I32], []);
    let gc_alloc_type = types.len();
    types.function([ValType::I32], [ValType::I32]);
    module.section(&types);

    let mut imports = ImportSection::new();
    let draw_rect_func = 0;
    let gc_alloc_func = 1;
    imports.import("env", "drawRect", EntityType::Function(draw_rect_type as u32));
    imports.import("env", "gc_alloc", EntityType::Function(gc_alloc_type as u32));
    module.section(&imports);

    let mut functions = FunctionSection::new();
    let draw_ui_func_index = draw_rect_func + 2;
    functions.function(draw_ui_type as u32);
    module.section(&functions);

    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, draw_ui_func_index);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut draw_ui = Function::new(vec![]);
    let mut map = vec![];
    let mut offset = 0u32;

    for node in ast {
        compile_node(node, &mut draw_ui, &mut map, &mut offset);
    }

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
    ">" => {
        func.instruction(&Instruction::I32GtS);
    }
    "<" => {
        func.instruction(&Instruction::I32LtS);
    }
    "==" => {
        func.instruction(&Instruction::I32Eq);
    }
    _ => panic!("Unsupported binary op: {}", op),
}

}

    }
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

            func.instruction(&Instruction::I32Const(64)); *offset_counter += 5;
            func.instruction(&Instruction::Call(1));     *offset_counter += 2;
            func.instruction(&Instruction::Drop);        *offset_counter += 1;

            func.instruction(&Instruction::I32Const(*x));      *offset_counter += 5;
            func.instruction(&Instruction::I32Const(*y));      *offset_counter += 5;
            func.instruction(&Instruction::I32Const(*width));  *offset_counter += 5;
            func.instruction(&Instruction::I32Const(*height)); *offset_counter += 5;
            func.instruction(&Instruction::Call(0));           *offset_counter += 2;

            map.push(ElementMap {
                kind: "box".to_string(),
                wasm_offset: off,
                pointer: None,
                source: Some(format!("wpp:x={},y={}", x, y)),
                props: Some(json!({ "x": x, "y": y, "width": width, "height": height })),
            });
        }

        Node::Print(msg) => {
            // Optional: Print stub or ignore in WASM mode
            eprintln!("🖨️ print('{}') is not supported in WASM yet", msg);
        }
        Node::If { condition, then_body, else_body } => {
    compile_expr(condition, func);

    func.instruction(&Instruction::If(BlockType::Empty));

    for stmt in then_body {
        compile_node(stmt, func, map, offset_counter);
    }

    if let Some(else_block) = else_body {
        func.instruction(&Instruction::Else);
        for stmt in else_block {
            compile_node(stmt, func, map, offset_counter);
        }
    }

    func.instruction(&Instruction::End);
}


    }
    
}
