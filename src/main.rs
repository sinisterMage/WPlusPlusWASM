use wasm_encoder::*;

fn main() {
    let mut module = Module::new();

    // 1. Type section: (i32, i32) -> () for print
    let mut types = TypeSection::new();
    types.function([ValType::I32, ValType::I32], []); // type 0
    module.section(&types);

    // 2. Import section: env.print
    let mut imports = ImportSection::new();
    imports.import("env", "print", EntityType::Function(0)); // uses type 0
    module.section(&imports);

    // 3. Function section: one function of type 0
    let mut functions = FunctionSection::new();
    functions.function(0); // type 0
    module.section(&functions);

    // 4. Memory section
    let mut memory = MemorySection::new();
    memory.memory(MemoryType {
        minimum: 1,
        maximum: None,
        memory64: false,
        shared: false,
    });
    module.section(&memory);

    // 5. Export section
    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, 1);      // CHANGED to index 1
    exports.export("memory", ExportKind::Memory, 0); // memory
    module.section(&exports);

    // 6. Code section
    let mut codes = CodeSection::new();
    let mut func = Function::new(vec![]);
    func.instruction(&Instruction::I32Const(0)); // pointer
    func.instruction(&Instruction::I32Const(15)); // length of "hello from W++"
    func.instruction(&Instruction::Call(0));     // call import at index 0
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    // 7. Data section: write string to offset 0
    let mut data = DataSection::new();
    let message = b"hello from W++";
    data.active(0, &ConstExpr::i32_const(0), message.to_vec());
    module.section(&data);

    // 8. Save to .wasm
    std::fs::write("hello.wasm", module.finish()).unwrap();
}
