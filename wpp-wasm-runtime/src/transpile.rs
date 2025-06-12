use wasm_encoder::*;
use crate::parser::{Node, Expr};
use crate::map::{ElementMap, SemanticMap};
use serde_json::json;
use std::collections::HashMap;
use crate::parser::FunctionMeta;



const DRAW_RECT_FUNC: u32 = 0;
const GC_ALLOC_FUNC: u32 = 1;
const DRAW_TEXT_FUNC: u32 = 2;
const ADD_ROOT_FUNC: u32 = 3;
const MARK_USED_FUNC: u32 = 4;
const TYPE_BOX: i32 = 1;
const TYPE_TEXT: i32 = 2;
const TYPE_GROUP: i32 = 3;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    pub name: String,
    pub param_types: Vec<String>,// <-- ADD THIS
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
    let mut next_index = BUILTIN_FUNCS.len() as u32; // Start at 5, reserve 0–4


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

        // ⛔ Don't insert metadata yet
        // Just save the signature to populate metadata *later*
        function_signatures.insert(sig.clone(), FunctionMetadata {
            returns: 0,
            wasm_index: 0, // ← dummy for now
            node: node.clone(),
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
    // ✅ Now update wasm_index in the metadata
for (sig, index) in &function_indices {
    if let Some(meta) = function_signatures.get_mut(sig) {
        meta.wasm_index = *index;
    }
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
    let mut local_types: HashMap<String, String> = HashMap::new(); // 🔁 shared across all compile_node calls
let mut reverse_func_index: HashMap<u32, FunctionSignature> = HashMap::new();
for (sig, index) in &function_indices {
    reverse_func_index.insert(*index, sig.clone());
}
let mut ordered_sigs: Vec<FunctionSignature> = function_signatures
    .keys()
    .cloned()
    .collect();

ordered_sigs.sort_by_key(|sig| function_indices.get(sig).copied());


for sig in ordered_sigs {
    let node = &function_signatures[&sig]; // safely look it up

    println!("\n🧠 [FunctionCompile] Compiling function '{}({:?})'", sig.name, sig.param_types);

    if let Node::Function { params, body, .. } = &node.node {
        println!("    └── Params count = {}", params.len());
        println!("🧩 [Function] Compiling function '{}({:?})'", sig.name, sig.param_types);

        let mut func = Function::new(vec![(1, ValType::I32)]); // reserve local[0] for GC ptr
        let mut local_map = HashMap::new();

        for (i, (param_name, _)) in params.iter().enumerate() {
            local_map.insert(param_name.clone(), i as u32);
        }

        let mut offset = 0;
        let mut wasm_locals: Vec<(u32, ValType)> = vec![];
        let mut dummy_map: Vec<ElementMap> = vec![];
        let mut body_instrs: Vec<Instruction> = vec![];

        println!("🔽 [Function Body] Starting statements for '{}'", sig.name);
let mut total_stack = 0;
let mut max_stack = 0;
let mut local_stack_counter = 0;
let mut local_idx: u32 = 0; // ← this makes the type known and fixes the error

for stmt in body {
    println!("   🔸 [stmt] Compiling statement in {:?}: {:?}", sig.name, stmt);

    let stack = crate::transpile::compile_node(
    stmt,
    &mut body_instrs,
    &mut dummy_map,
    &mut offset,
    &mut local_map,
    &mut local_types,
    &mut wasm_locals,
    &function_indices,
    &function_signatures,
    &mut local_stack_counter,
    &mut local_idx  // ✅ local per-function
);


    println!("   📏 Stack returned by stmt = {}", stack);
if sig.name == "describe" && sig.param_types == vec!["string".to_string()] {
    println!("🧠 Dumping function '{}({:?})'", sig.name, sig.param_types);
    for (i, instr) in body_instrs.iter().enumerate() {
        println!("   {:>3}: {:?}", i, instr);
    }
    println!("   🧮 Final stack before `end`: {}", local_stack_counter);
}

    total_stack += stack;
    max_stack = max_stack.max(total_stack); // 🔥 Track peak height

    if stack > 0 {
    for _ in 0..stack {
        println!("   🧯 Dropping leftover stack value");
        body_instrs.push(Instruction::Drop);
    }
}
if let Some(idx) = function_indices.get(&sig) {
    println!("📦 Compiling {}({:?}) → index {}", sig.name, sig.param_types, idx);
}

}
        println!("🔁 Inferred return count for '{}' = {}", sig.name, max_stack);
// 🧠 Store return count = max stack height
if let Some(meta) = function_signatures.get_mut(&sig) {
    meta.returns = max_stack as usize;
}


// 🧾 After compiling all statements in the function
println!(
    "📦 [FunctionEnd] Final stack for '{}' = {}, after drops pushed",
    sig.name, total_stack
);

// 🧹 Final cleanup before `End`

println!("🧠 Function {:?} has index {:?}", sig, function_indices.get(&sig));



        println!("✅ Function '{}' done → Instruction count: {}", sig.name, body_instrs.len());

         

        // 🧹 Drop any leftover values from stack
if local_stack_counter > 0 {
    println!("🧹 Appending {} Drop(s) to balance stack", local_stack_counter);
    for _ in 0..local_stack_counter {
        body_instrs.push(Instruction::Drop);
    }
}

// ➕ Emit all instructions including Drop(s)
for instr in &body_instrs {
    func.instruction(instr);
}

// Final sanity check
if local_stack_counter != 0 {
    println!(
        "❌ STACK MISMATCH in user function '{}({:?})' → final stack = {}",
        sig.name, sig.param_types, local_stack_counter
    );
    panic!("User function '{}' ends with unbalanced stack", sig.name);
} else {
    println!("✅ Stack clean at end of '{}({:?})'", sig.name, sig.param_types);
}

for (i, instr) in body_instrs.iter().enumerate() {
    println!("🧾 {:>3}: {:?}", i, instr);
}
println!("🧠 Pushed final Drop(s), stack should now be 0: {}", local_stack_counter);

        func.instruction(&Instruction::End);

        println!(
            "🧪 Compiled function at index {} with {} instructions",
            codes.len(),
            body_instrs.len()
        );

        codes.function(&func);
    }
}
   // === draw_ui body ===
let mut wasm_locals: Vec<(u32, ValType)> = vec![(1, ValType::I32)];
let mut local_map: HashMap<String, u32> = HashMap::new();
let mut instructions = vec![];
let mut map = vec![];
let mut offset = 0i32;

println!("📦 Compiling AST:\n{:#?}", ast);
let mut local_idx = 0;
for node in ast {
    if !matches!(node, Node::Function { .. }) {
        println!("🔵 [compile_to_wasm] Compiling top-level node: {:?}", node);


        let stack = crate::transpile::compile_node(
            node,
            &mut instructions,
            &mut map,
            &mut offset,
            &mut local_map,
            &mut local_types,
            &mut wasm_locals,
            &function_indices,
            &mut function_signatures,
            &mut stack_counter,
            &mut local_idx,
        );

        println!("🟢 [compile_to_wasm] Stack returned = {}", stack);
let mut sim_stack = 0;
for (i, instr) in instructions.iter().enumerate() {
    let before = sim_stack;

    match instr {
        Instruction::I32Const(_) | Instruction::LocalGet(_) => sim_stack += 1,
        Instruction::Drop | Instruction::LocalSet(_) => {
            if sim_stack > 0 {
                sim_stack -= 1;
            } else {
                println!("⚠️  DROPPING from empty stack at instruction #{}: {:?}", i, instr);
            }
        }
        Instruction::Call(index) => {
    match *index {
        DRAW_RECT_FUNC => sim_stack -= 4,
        GC_ALLOC_FUNC => { sim_stack -= 2; sim_stack += 1; }
        DRAW_TEXT_FUNC => sim_stack -= 4,
        ADD_ROOT_FUNC | MARK_USED_FUNC => sim_stack -= 1,
        _ => {
            if let Some((sig, meta)) = function_signatures.iter().find(|(_, f)| f.wasm_index == *index) {
                sim_stack -= sig.param_types.len() as i32;
                sim_stack += meta.returns as i32;
                println!(
                    "      ↪️ Call({}) → pops {}, pushes {} → stack change: {}",
                    index,
                    sig.param_types.len(),
                    meta.returns,
                    meta.returns as i32 - sig.param_types.len() as i32
                );
            } else {
                println!("❌ Unknown function call index: {}", index);
            }
        }
    }
}

        _ => {}
    }

    println!("{:>3}: {:?}    stack: {} → {}",
        i,
        instr,
        before,
        sim_stack
    );
}

        // 🧹 Drop any leftovers to ensure stack is empty
        match stack {
            0 => println!("✅ [Top-Level] Stack is clean"),
            1 => {
                println!("🪓 [Top-Level] Dropping 1 leftover value");
                instructions.push(Instruction::Drop);
            }
            n if n > 1 => {
                println!("🪓 [Top-Level] Dropping {} leftover values", n);
                for _ in 0..n {
                    instructions.push(Instruction::Drop);
                }
            }
            _ => panic!("❌ Invalid stack state: {}", stack),
        }
    }
}


println!("🧾 Final instruction list for draw_ui:");

let mut sim_stack = 0;
for (i, instr) in instructions.iter().enumerate() {
    match instr {
        Instruction::I32Const(_) => sim_stack += 1,
        Instruction::LocalGet(_) => sim_stack += 1,
        Instruction::Call(index) => {
    // ⚙️ Fetch metadata for the called function
    if let Some((sig, meta)) = function_signatures.iter().find(|(_, f)| f.wasm_index == *index) {
        let arg_count = sig.param_types.len() as i32;
        let ret_count = meta.returns as i32;
        sim_stack -= arg_count;
        sim_stack += ret_count;
        println!(
            "      ↪️ Call({}) → pops {}, pushes {} → stack change: {}",
            index,
            arg_count,
            ret_count,
            ret_count - arg_count
        );
    } else {
        println!("❌ Unknown function call index: {}", index);
    }
}

        Instruction::Drop | Instruction::LocalSet(_) => {
            if sim_stack > 0 { sim_stack -= 1; }
        }
        _ => {}
    }

    let approx_offset = i * 3; // crude estimate: each instruction ~3 bytes (very rough!)
println!("{:>3}: {:?}    ← stack: {}   @≈{}", i, instr, sim_stack, approx_offset);

}


// ✅ Always end with an explicit Drop for safety (but shouldn't be needed)
// ✅ FINAL cleanup using real stack simulation, not just stack_counter
let mut final_stack = 0;
for instr in &instructions {
    final_stack += stack_effect(instr, &reverse_func_index, &function_signatures);
}

// Calculate stack change across draw_ui body
let draw_ui_stack = stack_counter;

println!("🔍 FINAL STACK COUNT before `end`: {}", draw_ui_stack);
if draw_ui_stack != 0 {
    println!("❌ Stack counter at end of draw_ui is {}, expected 0", draw_ui_stack);
    panic!("Unbalanced stack in draw_ui");
}




instructions.push(Instruction::End);
stack_counter = 0; // Reset before gc_tick or other functions

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


    // ✅ DEBUG: Sanity check the GC Tick instruction list
println!("🧠 GC Tick Instruction Dump:");
println!("   ▶ I32Const(8)");
println!("   ▶ I32Const({})", TYPE_BOX);
println!("   ▶ Call({})", GC_ALLOC_FUNC);
println!("   ▶ Drop");
println!("   ▶ End");


// ✅ Add compiled function to code section
codes.function(&gc_tick_func);


    
    



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
    offset_counter: &mut i32,
    local_map: &HashMap<String, u32>,
    local_types: &mut HashMap<String, String>,
    wasm_locals: &mut Vec<(u32, ValType)>,
    function_indices: &HashMap<FunctionSignature, u32>,
    function_signatures: &HashMap<FunctionSignature, FunctionMetadata>,
    stack_counter: &mut i32,
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
    let len = s.len() as i32;

    // GC allocation
    instructions.push(Instruction::I32Const(len));
    instructions.push(Instruction::I32Const(TYPE_TEXT));
    instructions.push(Instruction::Call(GC_ALLOC_FUNC));
    instructions.push(Instruction::LocalTee(0)); // store pointer

    // GC bookkeeping
    instructions.push(Instruction::LocalGet(0));
    instructions.push(Instruction::Call(ADD_ROOT_FUNC));

    instructions.push(Instruction::LocalGet(0));
    instructions.push(Instruction::Call(MARK_USED_FUNC));

    // Write bytes to memory
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
        //instructions.push(Instruction::Drop);
    }

    // ✅ VERY IMPORTANT:
    // Only return a value if this is used in an expression (not a `Let`)
    *stack_counter += 1;
    1
}





        Expr::Binary { left, op, right } => {
    println!("   ↳ Binary Expression: {:?} {} {:?}", left, op, right);
    
    let left_stack: i32 = compile_expr(
    left,  // ✅ this was missing
    instructions,
    map,
    offset_counter,
    local_map,
    local_types,
    wasm_locals,
    function_indices,
    function_signatures,
    stack_counter,
);

let right_stack: i32 = compile_expr(
    right, // ✅ also insert this here
    instructions,
    map,
    offset_counter,
    local_map,
    local_types,
    wasm_locals,
    function_indices,
    function_signatures,
    stack_counter,
);


    assert_eq!(left_stack, 1, "Left side of binary expression must leave 1 value on stack");
    assert_eq!(right_stack, 1, "Right side of binary expression must leave 1 value on stack");

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

    *stack_counter -= 1; // ✅ Consumes 2, produces 1 → net change: -1
    1
}


        Expr::Layout(inner_node) => {
            let mut dummy_map = vec![];
            let mut dummy_offset = 0;
            let mut dummy_locals = HashMap::new();
            let mut dummy_types: HashMap<String, String> = HashMap::new();
            let mut dummy_layouts = vec![(1, ValType::I32)];
            let mut local_idx: u32 = 0;

println!("   ↳ Compiling embedded layout node");
{
    let _ = crate::transpile::compile_node(
    inner_node,
    instructions,
    &mut dummy_map,
    &mut dummy_offset,        // ✅ Correct order
    &mut dummy_locals,
    &mut dummy_types,         // ✅ now passes &mut HashMap<String, String>
    &mut dummy_layouts,
    function_indices,
    function_signatures,
    stack_counter,
    &mut local_idx,

);


}


            0
        }

        Expr::Call { name, args } => {
    let arg_types: Vec<String> = args.iter()
        .map(|arg| infer_type(arg, local_types))
        .collect();

    let sig = FunctionSignature {
        name: name.clone(),
        param_types: arg_types,
    };

    if let Some(index) = function_indices.get(&sig) {
        for arg in args {
            *stack_counter += compile_expr(
                arg,
                instructions,
                map,
                offset_counter,
                local_map,
                local_types,
                wasm_locals,
                function_indices,
                function_signatures,
                stack_counter,
            );
        }

        let meta = function_signatures.get(&sig).expect("Function signature missing");

        instructions.push(Instruction::Call(*index));

        // ✅ Don't manually adjust stack here — just return what it changes
        return meta.returns as i32;
    } else {
        panic!("Unknown function call: {:?}", sig);
    }
}


    }
}



fn compile_expr_and_discard(
    expr: &Expr,
    instructions: &mut Vec<Instruction>,
    map: &mut Vec<ElementMap>,
    offset_counter: &mut i32,
    local_map: &HashMap<String, u32>,
    local_types: &mut HashMap<String, String>,
    wasm_locals: &mut Vec<(u32, ValType)>,
    function_indices: &HashMap<FunctionSignature, u32>,
    function_signatures: &HashMap<FunctionSignature, FunctionMetadata>, // ✅ ADD THIS
    stack_counter: &mut i32,
)


 {
   let count: i32 = compile_expr(
    expr,
    instructions,
    map,
    offset_counter,
    local_map,
    local_types,
    wasm_locals,
    function_indices,
    function_signatures, // ✅ this was missing
    stack_counter,
);


    if count > 0 {
    instructions.push(Instruction::Drop);
    *stack_counter -= 1;
} else {
    println!("⚠️ Tried to Drop expression with no stack value");
}

}



pub fn compile_node(
    node: &Node,
    instructions: &mut Vec<Instruction>,
    map: &mut Vec<ElementMap>,
    offset_counter: &mut i32,
    local_map: &mut HashMap<String, u32>,
    local_types: &mut HashMap<String, String>,
    wasm_locals: &mut Vec<(u32, ValType)>,
    function_indices: &HashMap<FunctionSignature, u32>,
    function_signatures: &HashMap<FunctionSignature, FunctionMetadata>,
    stack_counter: &mut i32,
    local_idx: &mut u32, // ✅ NEW: added to support LocalSet
) -> i32 {




    println!("🔸 [compile_node] Entered with node: {:?}", node);
    match node {
        Node::Let { name, value } => {
    println!("🔸 [compile_node] Let {} = {:?}", name, value);

    let offset_before: i32 = *stack_counter;
let mut offset_counter: i32 = offset_before;

let returned: i32 = compile_expr(
    value,
    instructions,
    map,
    &mut offset_counter,   // ✅ now valid mutable borrow
    &local_map,
    local_types,
    wasm_locals,
    function_indices,
    function_signatures,
    stack_counter,
);



    if returned == 0 {
        // ✅ Special case: StringLiteral needs to store pointer manually
        if let Expr::StringLiteral(_) = value {
            instructions.push(Instruction::LocalGet(0));
            instructions.push(Instruction::LocalSet(*local_idx));
            println!("📍 Manually storing pointer from string literal to local '{}'", name);
            local_map.insert(name.clone(), *local_idx);
            let ty = infer_type(value, local_types);
            local_types.insert(name.clone(), ty);
            *stack_counter -= 1;
            *local_idx += 1;
        } else {
            println!("⚠️ Let binding for '{}' returned no value — skipping LocalSet", name);
        }
    } else {
        let ty = infer_type(value, local_types);
        instructions.push(Instruction::LocalSet(*local_idx));
        local_map.insert(name.clone(), *local_idx);
        local_types.insert(name.clone(), ty);
        //*stack_counter -= 1;
        
        *local_idx += 1;

    }

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


        let stack = compile_node(&rewritten, instructions, map, offset_counter, local_map, local_types, wasm_locals, function_indices,function_signatures, stack_counter, local_idx);
        println!("   🔁 Recursively compiled child #{}: {:?} → stack = {}", i, rewritten, stack);



        if stack > 0 {
    println!("🧯 [compile_node] Dropping leftover stack value after compiling {:?}", node);
    instructions.push(Instruction::Drop);
}


        cursor_main += main_size + space_between;
    }

    map.push(ElementMap {
        kind: "group".to_string(),
        wasm_offset: group_offset as u32,
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
        wasm_offset: off as u32,
        pointer: None,
        source: Some(format!("wpp:x={},y={}", x, y)),
        props: Some(json!({ "x": x, "y": y, "width": width, "height": height })),
    });

    0
}

        Node::Text { x, y, value } => {
    let off = *offset_counter;
    println!("📝 [Text] Compiling Text at ({}, {}) with value {:?}", x, y, value);

    let leaves_value_on_stack = match value {
        Expr::StringLiteral(s) => {
            let len = s.len() as i32;
            println!("➡️ Allocating text buffer for string of length {}", len);

            // Allocate buffer
            instructions.push(Instruction::I32Const(len));
            instructions.push(Instruction::I32Const(TYPE_TEXT));
            instructions.push(Instruction::Call(GC_ALLOC_FUNC));
            instructions.push(Instruction::LocalSet(0)); // Store in local[0]

            // Add to GC root
            instructions.push(Instruction::LocalGet(0));
            instructions.push(Instruction::Call(ADD_ROOT_FUNC));
            
            

            // Mark used
            instructions.push(Instruction::LocalGet(0));
            instructions.push(Instruction::Call(MARK_USED_FUNC));
            
            

            // Write bytes
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
                println!("   ↳ Wrote byte '{}' at offset {}", byte as char, i);
            }

            // Draw the string
            instructions.push(Instruction::I32Const(*x));
            instructions.push(Instruction::I32Const(*y));
            instructions.push(Instruction::LocalGet(0));
            instructions.push(Instruction::I32Const(len));
            instructions.push(Instruction::Call(DRAW_TEXT_FUNC));



            // ✅ No value left on stack (everything was consumed)
            false
        }

        Expr::Identifier(var) => {
            println!("🔗 [Text] Resolving identifier '{}'", var);
            if let Some(&idx) = local_map.get(var) {
                instructions.push(Instruction::I32Const(*x));
                instructions.push(Instruction::I32Const(*y));
                instructions.push(Instruction::LocalGet(idx));
                instructions.push(Instruction::I32Const(999)); // estimated length
                instructions.push(Instruction::Call(DRAW_TEXT_FUNC));
                println!("🖼️ Drew text from variable '{}'", var);
                false
            } else {
                panic!("❌ Unknown string variable '{}'", var);
            }
        }

        _ => panic!("❌ Unsupported text value: {:?}", value),
    };

    println!("🗺️ Pushing text element to semantic map...");
    map.push(ElementMap {
        kind: "text".to_string(),
        wasm_offset: off as u32,
        pointer: None,
        source: Some(format!("wpp:text='{:?}'", value)),
        props: Some(json!({ "x": x, "y": y })),
    });

    if leaves_value_on_stack {
        1 // ← will be dropped by caller if needed
    } else {
        0 // ← no value left on stack
    }
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

            let stack = compile_node(&rewritten, instructions, map, offset_counter, local_map, local_types, wasm_locals, function_indices,function_signatures, stack_counter, local_idx);


            if stack > 0 {
    println!("🧯 [compile_node] Dropping leftover stack value after compiling {:?}", node);
    instructions.push(Instruction::Drop);
}


            cursor += if is_horizontal { width } else { height } + gap;
        }
    }

    map.push(ElementMap {
        kind: "list".to_string(),
        wasm_offset: *offset_counter as u32,
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


        Node::Expr(expr) => {
    let result = compile_expr(
        expr, instructions, map, offset_counter, local_map,
        local_types, wasm_locals, function_indices, function_signatures,
        stack_counter,
    );

    if result > 0 {
        instructions.push(Instruction::Drop);
        *stack_counter -= 1;
    }

    0 // <- always return 0 for Node-level expr
}








       Node::If { condition, then_body, else_body } => {
        println!("🧪 [If] Compiling condition: {:?}", condition);
    let count = compile_expr(
    condition,
    instructions,
    map,
    offset_counter,
    local_map,
    local_types,
    wasm_locals,
    function_indices,
    function_signatures,
    stack_counter,
);
 // ✅ Push the condition
    println!("🔀 Entering IF block");
    instructions.push(Instruction::If(BlockType::Empty));


    let mut max_stack = 0;
println!("🟩 [Then] Block has {} statements", then_body.len());

    for stmt in then_body {
        println!("   🟢 Compiling THEN stmt: {:?}", stmt);

    max_stack = max_stack.max(compile_node(stmt, instructions, map, offset_counter, local_map, local_types, wasm_locals, function_indices, function_signatures, stack_counter, local_idx)
);
}

if let Some(else_branch) = else_body {
    println!("🔁 Entering ELSE block");
    instructions.push(Instruction::Else);
    println!("🟥 [Else] Block has {} statements", else_branch.len());
    for stmt in else_branch {
        println!("   🔴 Compiling ELSE stmt: {:?}", stmt);
        max_stack = max_stack.max(compile_node(stmt, instructions, map, offset_counter, local_map, local_types, wasm_locals, function_indices, function_signatures, stack_counter, local_idx)
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
pub fn infer_type(expr: &Expr, locals: &HashMap<String, String>) -> String {
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
const BUILTIN_FUNCS: &[(u32, i32)] = &[
    (0, -4),       // draw_rect(x, y, w, h) → 0
    (1, -2 + 1),   // gc_alloc(len, type) → ptr
    (2, -4),       // draw_text(x, y, text, size) → 0
    (3, -1),       // add_root(ptr) → 0
    (4, -1),       // drop_root(ptr) → 0
];





pub fn stack_effect(
    instr: &Instruction,
    reverse_func_index: &HashMap<u32, FunctionSignature>,
    function_signatures: &HashMap<FunctionSignature, FunctionMetadata>,
) -> i32 {
    match instr {
        Instruction::Call(index) => {
            // 🔍 1. Check built-in functions first
            for (builtin_index, result) in BUILTIN_FUNCS {
                if *index == *builtin_index {
                    println!("📞 Evaluating Call({}) as built-in → Δstack = {}", index, result);
                    return *result;
                }
            }

            // 🔍 2. Check user-defined functions
            if let Some(sig) = reverse_func_index.get(index) {
                if let Some(meta) = function_signatures.get(sig) {
                    let param_count = sig.param_types.len() as i32;
                    let return_count = meta.returns as i32;
                    let delta = return_count - param_count;
                    println!(
                        "📞 Evaluating Call({}) as user-defined → pops {}, pushes {} → Δstack = {}",
                        index, param_count, return_count, delta
                    );
                    return delta;
                } else {
                    println!("❌ Signature found but metadata missing for {:?}", sig);
                }
            }

            println!("❌ Unknown function call index: {}", index);
            0 // You may want to panic!() instead if this is a hard error
        }

        Instruction::I32Const(_) => 1,
        Instruction::LocalGet(_) => 1,
        Instruction::LocalSet(_) => -1,
        Instruction::LocalTee(_) => 0,
        Instruction::I32Add => -1,
        Instruction::I32Store8(_) => -2,
        Instruction::Drop => -1,
        Instruction::End => 0,

        _ => {
            println!("⚠️ Unhandled instruction: {:?}", instr);
            0
        }
    }
}


