use crate::parser::Statement;

pub fn transpile_to_rust(stmts: &[Statement]) -> String {
    let mut rust_code = String::new();

    rust_code.push_str("fn run() {\n");

    for stmt in stmts {
        match stmt {
            Statement::Print(msg) => {
                rust_code.push_str(&format!("    println!(\"{}\\n\");\n", msg));
            }
        }
    }

    rust_code.push_str("}\n");

    rust_code
}
