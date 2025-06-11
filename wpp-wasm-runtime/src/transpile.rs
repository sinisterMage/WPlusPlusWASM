use wasm_encoder::*;
use crate::parser::{Node, Expr};
use crate::map::{ElementMap, SemanticMap};
use serde_json::json;
use std::collections::HashMap;


const DRAW_RECT_FUNC: u32 = 0;
const GC_ALLOC_FUNC: u32 = 1;
const DRAW_TEXT_FUNC: u32 = 2;
const ADD_ROOT_FUNC: u32 = 3;
const MARK_USED_FUNC: u32 = 4;
const TYPE_BOX: i32 = 1;
const TYPE_TEXT: i32 = 2;
const TYPE_GROUP: i32 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FunctionSignature {
    name: String,
    param_types: Vec<String>,
}
#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    pub node: Node,
    pub wasm_index: u32,
    pub returns: usize, // <- how many values this function returns
}




pub fn compile_to_wasm(ast: &[Node]) -> (Vec<u8>, String) {
    let mut module = Module::new();
    let mut stack_counter: i32 = 0; // ⬅️ Add this at the start of draw_ui compilation

    // === Type Section ===
    let mut types = TypeSection::new();
    let draw_ui_type = types.len(); types.function([], []);
    let draw_rect_type = types.len(); types.function([ValType::I32; 4], []);
    let gc_alloc_type = types.len(); types.function([ValType::I32; 2], [ValType::I32]);
    let draw_text_type = types.len(); types.function([ValType::I32; 4], []);
    let add_root_type = types.len(); types.function([ValType::I32], []);
    let mark_used_type = types.len(); types.function([ValType::I32], []);
    let gc_tick_type = types.len(); types.function([], []);

    // === Function Signature Collection ===
    let mut function_signatures: HashMap<FunctionSignature, FunctionMetadata> = HashMap::new();
    let mut function_indices = HashMap::<FunctionSignature, u32>::new();
    let mut function_type_indices = HashMap::<FunctionSignature, u32>::new();

    for node in ast {
        if let Node::Function { name, params, .. } = node {
            let sig = FunctionSignature {
                name: name.clone(),
                param_types: params.iter().map(|(_, ty)| ty.clone()).collect(),
            };

            if function_signatures.contains_key(&sig) {
                panic!("❌ Duplicate function signature: {:?}", sig);
            }

            function_signatures.insert(sig, FunctionMetadata {
    returns: 0, // HACK: assume single int return (patch later)
    wasm_index: 0, // dummy, will be set later
    node: node.clone(), // store node for later
});

        }
    }

    // === Index prep ===
    let imported_funcs = 6; // ✅ six imported functions (not counting memory)
let gc_tick_func_index = imported_funcs as u32; // = 6
let draw_ui_func_index = gc_tick_func_index + 1; // = 7
let user_func_start_index = draw_ui_func_index + 1; // = 8


    // === Add user-defined function types and assign indices ===
    for sig in function_signatures.keys() {
        let wasm_param_types: Vec<ValType> = sig.param_types.iter().map(|ty| match ty.as_str() {
            "int" => ValType::I32,
            "string" => ValType::I32,
            _ => panic!("Unsupported type: {}", ty),
        }).collect();

        let type_index = types.len();
        types.function(wasm_param_types.clone(), Vec::new());
        function_type_indices.insert(sig.clone(), type_index as u32);

        let current_index = user_func_start_index + function_indices.len() as u32;
        function_indices.insert(sig.clone(), current_index);
    }

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
    imports.import("env", "mark_used", EntityType::Function(mark_used_type as u32));
    imports.import("env", "gc_tick", EntityType::Function(gc_tick_type as u32));
    module.section(&imports);

    // === Function Section ===
    let mut functions = FunctionSection::new();
    functions.function(draw_ui_type as u32);
    functions.function(gc_tick_type as u32);
    for sig in function_signatures.keys() {
        let type_index = *function_type_indices.get(sig).unwrap();
        functions.function(type_index);
    }
    module.section(&functions);

    // === Export Section ===
    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, draw_ui_func_index);
    exports.export("gc_tick", ExportKind::Func, gc_tick_func_index);
    module.section(&exports);

    // === Code Section ===
    let mut codes = CodeSection::new();

   // === draw_ui body ===
let mut wasm_locals: Vec<(u32, ValType)> = vec![(1, ValType::I32)];
let mut local_map: HashMap<String, u32> = HashMap::new();
let mut instructions = vec![];
let mut map = vec![];
let mut offset = 0u32;

println!("📦 Compiling AST:\n{:#?}", ast);

for node in ast {
    if !matches!(node, Node::Function { .. }) {
        println!("🔵 [compile_to_wasm] Compiling top-level node: {:?}", node);

       let mut stack: i32;
{
    stack = crate::transpile::compile_node(
        node,
        &mut instructions,
        &mut map,
        &mut offset,
        &mut local_map,
        &mut wasm_locals,
        &function_indices,
        &mut stack_counter,
    );
}


        println!("🟢 [compile_to_wasm] Stack returned = {}", stack);

        // 🔨 Always drop after Expr(...) just in case
        if let Node::Expr(Expr::Call { name, args }) = node {
    let arg_types: Vec<String> = args.iter()
        .map(|arg| infer_type(arg, &local_map))
        .collect();

    let sig = FunctionSignature {
        name: name.clone(),
        param_types: arg_types,
    };

    if let Some(func) = function_signatures.get(&sig) {
        if func.returns == 0 {
            println!("🪓 [Top-Level] Dropping result of function {:?}", name);
            instructions.push(Instruction::Drop);
            stack += 1; // manually reflect we dropped a value
        }
    }
}




    }
}


println!("🧾 Final instruction list for draw_ui:");
for (i, instr) in instructions.iter().enumerate() {
    println!("  {:>3}: {:?}", i, instr);
}

// ✅ Always end with an explicit Drop for safety (but shouldn't be needed)
instructions.push(Instruction::End);

let mut draw_ui = Function::new(wasm_locals.clone());
for instr in &instructions {
    draw_ui.instruction(instr);
}
codes.function(&draw_ui);




    // === GC Tick Stub ===
    println!("🔧 [gc_tick] Starting GC tick function");

let mut gc_tick_func = Function::new(vec![]);

println!("  ➕ Pushing i32.const 8");
gc_tick_func.instruction(&Instruction::I32Const(8));

println!("  ➕ Pushing i32.const TYPE_BOX ({})", TYPE_BOX);
gc_tick_func.instruction(&Instruction::I32Const(TYPE_BOX));

println!("  🛠️ Calling GC_ALLOC_FUNC");
gc_tick_func.instruction(&Instruction::Call(GC_ALLOC_FUNC));

println!("  🧹 Dropping allocated ptr (only one value)");
gc_tick_func.instruction(&Instruction::Drop);



// ✅ SAFETY: ensure there's *absolutely* nothing left
gc_tick_func.instruction(&Instruction::End);


    codes.function(&gc_tick_func);

    // === User-defined function bodies ===
    let mut ordered_sigs: Vec<_> = function_signatures.iter().collect();
ordered_sigs.sort_by_key(|(sig, _)| function_indices.get(sig).copied().unwrap());


for (sig, node) in ordered_sigs {
    println!("\n🧠 [FunctionCompile] Compiling function '{}({:?})'", sig.name, sig.param_types);

    if let Node::Function { params, body, .. } = &node.node {
        println!("    └── Params count = {}", params.len());
        println!("🧩 [Function] Compiling function '{}({:?})'", sig.name, sig.param_types);

        let mut func = Function::new(vec![]);
        let mut local_map = HashMap::new();

        for (i, (param_name, _)) in params.iter().enumerate() {
            local_map.insert(param_name.clone(), i as u32);
        }

        let mut offset = 0;
        let mut wasm_locals: Vec<(u32, ValType)> = vec![];
        let mut dummy_map: Vec<ElementMap> = vec![];
        let mut body_instrs: Vec<Instruction> = vec![];

        println!("🔽 [Function Body] Starting statements for '{}'", sig.name);

for stmt in body {
    println!("   🔸 [stmt] Compiling statement in {:?}: {:?}", sig.name, stmt);
   let stack: i32;
{
    stack = crate::transpile::compile_node(
        stmt,
        &mut body_instrs,
        &mut dummy_map,
        &mut offset,
        &mut local_map,
        &mut wasm_locals,
        &function_indices,
        &mut stack_counter,
    );
}


    println!("   📏 Stack returned by stmt = {}", stack);

    for _ in 0..stack {
        println!("   🧯 Dropping leftover stack value");
        body_instrs.push(Instruction::Drop);
    }
}


        println!("✅ Function '{}' done → Instruction count: {}", sig.name, body_instrs.len());

        for instr in &body_instrs {
            func.instruction(instr);
        }

        func.instruction(&Instruction::End);
        codes.function(&func);
    }
}



    module.section(&codes);

    let wasm = module.finish();
    let semantic = SemanticMap { elements: map };
    let map_json = serde_json::to_string_pretty(&semantic).unwrap();

    (wasm, map_json)
}

pub fn compile_expr(
    expr: &Expr,
    instructions: &mut Vec<Instruction>,
    map: &mut Vec<ElementMap>,
    offset_counter: &mut u32,
    local_map: &HashMap<String, u32>,
    wasm_locals: &mut Vec<(u32, ValType)>,
    function_indices: &HashMap<FunctionSignature, u32>,
    stack_counter: &mut i32, // ✅ NEW
) -> i32 {


    println!("🧮 [compile_expr] Entered with expr: {:?}", expr);
    match expr {
        Expr::Literal(n) => {
            instructions.push(Instruction::I32Const(*n));
            println!("   ↳ Literal {} → push I32Const", n);
            1
        }

        Expr::Identifier(name) => {
            println!("   ↳ Identifier lookup: '{}'", name);
            if let Some(&index) = local_map.get(name) {
                instructions.push(Instruction::LocalGet(index));
                println!("   ↳ Found. Pushed LocalGet({})", index);
                1
            } else {
                panic!("❌ Undefined variable: {}", name);
            }
        }

        Expr::StringLiteral(s) => {
            println!("   ↳ Allocating GC buffer for string '{}'", s);
            let len = s.len() as i32;

            instructions.push(Instruction::I32Const(len));
            instructions.push(Instruction::I32Const(TYPE_TEXT));
            instructions.push(Instruction::Call(GC_ALLOC_FUNC));
            instructions.push(Instruction::LocalTee(0)); // keep pointer in local 0
            println!("   ↳ GC ptr stored in local[0]");

            for (i, byte) in s.bytes().enumerate() {
                instructions.push(Instruction::LocalGet(0));
                instructions.push(Instruction::I32Const(i as i32));
                instructions.push(Instruction::I32Add);
                instructions.push(Instruction::I32Const(byte as i32));
                instructions.push(Instruction::I32Store8(MemArg {
                    align: 0,
                    offset: 0,
                    memory_index: 0,
                }));
            }

            instructions.push(Instruction::LocalGet(0)); // leave the string pointer on stack
            println!("   ↳ Done writing string. Leaving ptr on stack.");
            1
        }

        Expr::Binary { left, op, right } => {
            println!("   ↳ Binary Expression: {:?} {} {:?}", left, op, right);
            let left_stack: i32 = compile_expr(
    left,
    instructions,
    map,
    offset_counter,
    local_map,
    wasm_locals,
    function_indices,
    stack_counter,
);
            let right_stack: i32 = compile_expr(
    right,
    instructions,
    map,
    offset_counter,
    local_map,
    wasm_locals,
    function_indices,
    stack_counter,
);
            assert_eq!(left_stack, 1);
            assert_eq!(right_stack, 1);
            let op_instr = match op.as_str() {
                "+" => Instruction::I32Add,
                "-" => Instruction::I32Sub,
                "*" => Instruction::I32Mul,
                "/" => Instruction::I32DivS,
                "==" => Instruction::I32Eq,
                ">" => Instruction::I32GtS,
                "<" => Instruction::I32LtS,
                _ => panic!("❌ Unsupported binary operator: {}", op),
            };
            println!("   ↳ Applied operator: '{}'", op);
            instructions.push(op_instr);
            1
        }

        Expr::Layout(inner_node) => {
            let mut dummy_map = vec![];
            let mut dummy_offset = 0;
            let mut dummy_locals = HashMap::new();
            let mut dummy_layouts = vec![(1, ValType::I32)];
println!("   ↳ Compiling embedded layout node");
{
    let _ = crate::transpile::compile_node(
    inner_node,
    instructions,
    &mut dummy_map,
    &mut dummy_offset,
    &mut dummy_locals,
    &mut dummy_layouts,
    function_indices,
    stack_counter,
);

}


            0
        }

        Expr::Call { name, args } => {
            println!("   ↳ Function call '{}', with {} args", name, args.len());
            let param_types: Vec<String> = args
                .iter()
                .map(|arg| match arg {
                    Expr::Literal(_) => "int",
                    Expr::Identifier(_) => "int",
                    Expr::Binary { .. } => "int",
                    Expr::Layout(_) => "int",
                    Expr::Call { .. } => "int",
                    Expr::StringLiteral(_) => "string",
                }.to_string())
                .collect();
            println!("🔍 Inferred param types: {:?}", param_types);

            let sig = FunctionSignature {
                name: name.clone(),
                param_types,
            };
            println!("📞 Preparing to call function {:?} with param types: {:?}", name, sig.param_types);

            let func_index = match function_indices.get(&sig) {
    Some(index) => index,
    None => {
        println!("❌ No function found for '{}', with param types: {:?}", name, sig.param_types);

        println!("📚 Available overloads for '{}':", name);
        for FunctionSignature { name: func_name, param_types } in function_indices.keys() {
            if func_name == name {
                println!("   → {:?}({:?})", func_name, param_types);
            }
        }

        panic!("❌ Dispatch error: No matching function for '{}({:?})'", name, sig.param_types);
    }
};


println!("📞 Calling function {:?} with signature {:?} → index {}", name, sig.param_types, func_index);

for (i, arg) in args.iter().enumerate() {
    println!("   ↪️ Compiling arg #{}: {:?}", i, arg);
    let count: i32 = compile_expr(
    arg,                // ✅ This is fine, it's &Expr
    instructions,
    map,
    offset_counter,
    local_map,
    wasm_locals,
    function_indices,
    stack_counter,
);

    println!("   ✅ Arg #{} left {} value(s) on stack", i, count);
    assert_eq!(count, 1, "Each argument must leave 1 value on the stack");
}


println!("🔔 Calling function {:?} with param types: {:?}", name, sig.param_types);
instructions.push(Instruction::Call(*func_index));

            0 // assuming user functions return void
        }
    }
}



fn compile_expr_and_discard(
    expr: &Expr,
    instructions: &mut Vec<Instruction>,
    map: &mut Vec<ElementMap>,
    offset_counter: &mut u32,
    local_map: &HashMap<String, u32>,
    wasm_locals: &mut Vec<(u32, ValType)>,
    function_indices: &HashMap<FunctionSignature, u32>,
    stack_counter: &mut i32,
)

 {
   let count = compile_expr(
    expr,
    instructions,
    map,
    offset_counter,
    local_map,
    wasm_locals,
    function_indices,
    stack_counter,
);

    if count > 0 {
        instructions.push(Instruction::Drop);
    }
}



pub fn compile_node(
    node: &Node,
    instructions: &mut Vec<Instruction>,
    map: &mut Vec<ElementMap>,
    offset_counter: &mut u32,
    local_map: &mut HashMap<String, u32>,
    wasm_locals: &mut Vec<(u32, ValType)>,
    function_indices: &HashMap<FunctionSignature, u32>,
    stack_counter: &mut i32, // ✅ NEW
) -> i32 {


    println!("🔸 [compile_node] Entered with node: {:?}", node);
    match node {
        Node::Let { name, value } => {
    let count = compile_expr(
    value,
    instructions,
    map,
    offset_counter,
    local_map,
    wasm_locals,
    function_indices,
    stack_counter,
);

 // value pushed on stack

    let local_index = wasm_locals.len() as u32;
    local_map.insert(name.clone(), local_index);      // symbol → local
    wasm_locals.push((1, ValType::I32));              // add to final locals list

    instructions.push(Instruction::LocalSet(local_index)); // assign to local

    0
}


        Node::Group { direction, gap, align, justify, padding, children } => {
    let group_offset = *offset_counter;

    // === GC Allocation for Group ===
    instructions.push(Instruction::I32Const(8));
instructions.push(Instruction::I32Const(TYPE_GROUP));
instructions.push(Instruction::Call(GC_ALLOC_FUNC));
instructions.push(Instruction::LocalTee(0));
instructions.push(Instruction::LocalGet(0));
instructions.push(Instruction::Call(ADD_ROOT_FUNC));
instructions.push(Instruction::Drop);


    // === Layout Pre-Pass ===
    let is_horizontal = direction == "horizontal";
    let mut total_main_size = 0;
    let mut child_dims = vec![]; // (main_size, cross_size)
println!("📦 [Group] direction = {}, gap = {}, padding = {}, align = {}, justify = {}",
         direction, gap, padding, align, justify);

    for child in children {
        match child {
            Node::Box { width, height, .. } => {
                let main = if is_horizontal { *width } else { *height };
                let cross = if is_horizontal { *height } else { *width };
                total_main_size += main + gap;
                child_dims.push((main, cross));
            }
            Node::Text { value, .. } => {
    let (width, height) = match value {
        Expr::StringLiteral(s) => {
            let width = s.len() as i32 * 8;
            (width, 16)
        }
        Expr::Identifier(_) => {
            // You can’t know the actual length at compile time — use placeholder dimensions
            (999, 16)
        }
        _ => panic!("Unsupported value type in Text layout: {:?}", value),
    };

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
println!("  🔧 Child #{}: original = {:?}", i, child);
println!("     → Layout x = {}, y = {}, main_size = {}, cross_size = {}", x, y, main_size, cross_size);

       let rewritten: Node = match child {
    Node::Box { width, height, .. } => {
        let rewritten = Node::Box {
            x,
            y,
            width: *width,
            height: *height,
        };
        println!("  🎨 Compiling child node (rewritten): {:?}", rewritten);
        rewritten
    }

    Node::Text { value, .. } => {
        let rewritten = Node::Text {
            x,
            y,
            value: value.clone(),
        };
        println!("  🎨 Compiling child node (rewritten): {:?}", rewritten);
        rewritten
    }

    _ => {
        let rewritten = child.clone();
        println!("  🎨 Compiling child node (rewritten - fallback): {:?}", rewritten);
        rewritten
    }
};


        let stack = compile_node(&rewritten, instructions, map, offset_counter, local_map, wasm_locals, function_indices, stack_counter,);
        println!("   🔁 Recursively compiled child #{}: {:?} → stack = {}", i, rewritten, stack);



        if stack > 0 {
    println!("🧯 [compile_node] Dropping leftover stack value after compiling {:?}", node);
    instructions.push(Instruction::Drop);
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
    println!("📦 [Box] Compiling Box at ({}, {}) size {}x{}", x, y, width, height);

    println!("➡️ Allocating GC box...");
    instructions.push(Instruction::I32Const(0)); *offset_counter += 5;
    println!("   ↳ Pushed I32Const(0)");
    
    instructions.push(Instruction::I32Const(TYPE_BOX)); *offset_counter += 5;
    println!("   ↳ Pushed I32Const(TYPE_BOX = {})", TYPE_BOX);
    
    instructions.push(Instruction::Call(GC_ALLOC_FUNC)); *offset_counter += 2;
    println!("   ↳ Called GC_ALLOC_FUNC");

    instructions.push(Instruction::LocalTee(0)); *offset_counter += 2;
    println!("   ↳ Teed result into local[0]");

    instructions.push(Instruction::LocalGet(0));
    instructions.push(Instruction::Call(ADD_ROOT_FUNC)); *offset_counter += 2;
    println!("➡️ Added to GC roots");

    instructions.push(Instruction::Drop);
    println!("⚠️ Dropped extra pointer (was not reused immediately)");

    instructions.push(Instruction::LocalGet(0));
instructions.push(Instruction::Call(MARK_USED_FUNC));
instructions.push(Instruction::Drop); // ✅ NEW LINE

    println!("✅ Marked GC object as used");

    println!("🖼️ Drawing box with drawRect({}, {}, {}, {})", x, y, width, height);
    instructions.push(Instruction::I32Const(*x)); *offset_counter += 5;
    instructions.push(Instruction::I32Const(*y)); *offset_counter += 5;
    instructions.push(Instruction::I32Const(*width)); *offset_counter += 5;
    instructions.push(Instruction::I32Const(*height)); *offset_counter += 5;
    instructions.push(Instruction::Call(DRAW_RECT_FUNC)); *offset_counter += 2;

    println!("🗺️ Pushing box element to semantic map...");
    map.push(ElementMap {
        kind: "box".to_string(),
        wasm_offset: off,
        pointer: None,
        source: Some(format!("wpp:x={},y={}", x, y)),
        props: Some(json!({ "x": x, "y": y, "width": width, "height": height })),
    });

    0
}

        Node::Text { x, y, value } => {
    let off = *offset_counter;
    println!("📝 [Text] Compiling Text at ({}, {}) with value {:?}", x, y, value);

    match value {
        Expr::StringLiteral(s) => {
            let len = s.len() as i32;
            println!("➡️ Allocating text buffer for string of length {}", len);

            instructions.push(Instruction::I32Const(len)); *offset_counter += 5;
            println!("   ↳ Pushed I32Const(len = {})", len);

            instructions.push(Instruction::I32Const(TYPE_TEXT)); *offset_counter += 5;
            println!("   ↳ Pushed I32Const(TYPE_TEXT = {})", TYPE_TEXT);

            instructions.push(Instruction::Call(GC_ALLOC_FUNC)); *offset_counter += 2;
            println!("   ↳ Called GC_ALLOC_FUNC");

            instructions.push(Instruction::LocalTee(0)); *offset_counter += 2;
            println!("   ↳ Teed allocated pointer into local[0]");

            instructions.push(Instruction::LocalGet(0));
            instructions.push(Instruction::Call(ADD_ROOT_FUNC)); *offset_counter += 2;
            println!("   ↳ Added to GC roots");

            instructions.push(Instruction::Drop);
            println!("⚠️ Dropped pointer (not reused here directly)");

            instructions.push(Instruction::LocalGet(0));
instructions.push(Instruction::Call(MARK_USED_FUNC));
instructions.push(Instruction::Drop); // ✅ NEW LINE

            println!("✅ Marked GC text buffer as used");

            println!("✍️ Writing string bytes to memory:");
            for (i, byte) in s.bytes().enumerate() {
                instructions.push(Instruction::LocalGet(0));
                instructions.push(Instruction::I32Const(i as i32));
                instructions.push(Instruction::I32Add);
                instructions.push(Instruction::I32Const(byte as i32));
                instructions.push(Instruction::I32Store8(MemArg {
                    align: 0,
                    offset: 0,
                    memory_index: 0,
                }));
                *offset_counter += 8;
                println!("   ↳ Wrote byte '{}' at offset {}", byte as char, i);
            }

            println!("🖼️ Drawing string at ({}, {})", x, y);
            instructions.push(Instruction::I32Const(*x)); *offset_counter += 5;
            instructions.push(Instruction::I32Const(*y)); *offset_counter += 5;
            instructions.push(Instruction::LocalGet(0)); *offset_counter += 2;
            instructions.push(Instruction::I32Const(len)); *offset_counter += 5;
            instructions.push(Instruction::Call(DRAW_TEXT_FUNC)); *offset_counter += 2;
        }

        Expr::Identifier(var) => {
            println!("🔗 [Text] Resolving identifier '{}'", var);
            if let Some(&idx) = local_map.get(var) {
                instructions.push(Instruction::I32Const(*x)); *offset_counter += 5;
                instructions.push(Instruction::I32Const(*y)); *offset_counter += 5;
                instructions.push(Instruction::LocalGet(idx)); *offset_counter += 2;
                instructions.push(Instruction::I32Const(999)); // TODO: estimate string length
                instructions.push(Instruction::Call(DRAW_TEXT_FUNC)); *offset_counter += 2;
                println!("🖼️ Drew text from variable '{}'", var);
            } else {
                panic!("❌ Unknown string variable '{}'", var);
            }
        }

        _ => panic!("❌ Unsupported text value: {:?}", value),
    }

    println!("🗺️ Pushing text element to semantic map...");
    map.push(ElementMap {
        kind: "text".to_string(),
        wasm_offset: off,
        pointer: None,
        source: Some(format!("wpp:text='{:?}'", value)),
        props: Some(json!({ "x": x, "y": y })),
    });

    0
}



        Node::List { direction, gap, padding, items } => {
    let is_horizontal = direction == "horizontal";
    let mut cursor = *padding;
println!("📋 [List] direction = {}, gap = {}, padding = {}", direction, gap, padding);

    for item in items {
        if let Node::Item { value } = item {
            let width = value.len() as i32 * 8;
            let height = 16;

            let (x, y) = if is_horizontal {
                (cursor, *padding)
            } else {
                (*padding, cursor)
            };

            let rewritten = Node::Text {
                x,
                y,
                value: Expr::StringLiteral(value.clone()),

            };
println!("  📎 Item '{}': → rendered at ({}, {})", value, x, y);

            let stack = compile_node(&rewritten, instructions, map, offset_counter, local_map, wasm_locals, function_indices, stack_counter);


            if stack > 0 {
    println!("🧯 [compile_node] Dropping leftover stack value after compiling {:?}", node);
    instructions.push(Instruction::Drop);
}


            cursor += if is_horizontal { width } else { height } + gap;
        }
    }

    map.push(ElementMap {
        kind: "list".to_string(),
        wasm_offset: *offset_counter,
        pointer: None,
        source: None,
        props: Some(json!({
            "direction": direction,
            "gap": gap,
            "padding": padding
        })),
    });

    0
}


        Node::Expr(e) => {
    compile_expr_and_discard(
        e,
        instructions,
        map,
        offset_counter,
        local_map,
        wasm_locals,
        function_indices,
        stack_counter,
    );
    0
}



       Node::If { condition, then_body, else_body } => {
        println!("🧪 [If] Compiling condition: {:?}", condition);
    let count = compile_expr(
    condition,
    instructions,
    map,
    offset_counter,
    local_map,
    wasm_locals,
    function_indices,
    stack_counter,
);
 // ✅ Push the condition
    println!("🔀 Entering IF block");
    instructions.push(Instruction::If(BlockType::Empty));


    let mut max_stack = 0;
println!("🟩 [Then] Block has {} statements", then_body.len());

    for stmt in then_body {
        println!("   🟢 Compiling THEN stmt: {:?}", stmt);

    max_stack = max_stack.max(compile_node(stmt, instructions, map, offset_counter, local_map, wasm_locals, function_indices, stack_counter)
);
}

if let Some(else_branch) = else_body {
    println!("🔁 Entering ELSE block");
    instructions.push(Instruction::Else);
    println!("🟥 [Else] Block has {} statements", else_branch.len());
    for stmt in else_branch {
        println!("   🔴 Compiling ELSE stmt: {:?}", stmt);
        max_stack = max_stack.max(compile_node(stmt, instructions, map, offset_counter, local_map, wasm_locals, function_indices, stack_counter)
);
    }
}
println!("🧱 Closing IF/ELSE block");

    instructions.push(Instruction::End);
println!("📥 IF block left value on stack ({}), dropping...", max_stack);

    if max_stack > 0 {
        instructions.push(Instruction::Drop);
    }

    0
}


        _ => {
    println!("⚠️ [compile_node] Unhandled node variant → {:?}", node);
    0 // don't emit Drop if nothing was pushed
}


    }
    
}
/// Infer a type for an expression for dispatch resolution
pub fn infer_type(expr: &Expr, locals: &HashMap<String, u32>) -> String {
    match expr {
        Expr::Literal(_) => "int".to_string(),
        Expr::StringLiteral(_) => "string".to_string(),
        Expr::Identifier(name) => {
            // Very basic logic: use variable naming heuristics
            if name.starts_with('s') {
                "string".to_string()
            } else {
                "int".to_string()
            }
        }
        _ => "int".to_string(), // fallback for now
    }
}
fn stack_effect(instr: &Instruction) -> i32 {
    use Instruction::*;
    match instr {
        // Pushes a value
        I32Const(_) => 1,
        LocalGet(_) => 1,
        LocalTee(_) => 0, // Leaves one copy on the stack
        // Pops one value
        Drop => -1,
        Call(_) => {
            // We will handle Call separately (Step 3)
            0
        }
        I32Add | I32Sub | I32Mul | I32DivS | I32DivU => -1, // Pops 2, pushes 1 → net -1
        I32Store8(_) => -2, // addr and value
        I32Load8U(_) => -1 + 1, // loads from addr, pushes result
        End => 0,
        _ => 0, // fallback, add more as needed
    }
}


