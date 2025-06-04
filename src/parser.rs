pub enum Statement {
    Print(String),
}

pub fn parse(source: &str) -> Vec<Statement> {
    let mut stmts = vec![];

    for line in source.lines() {
        let line = line.trim();

        if line.starts_with("print ") {
            let msg = line.trim_start_matches("print ").trim_matches('"');
            stmts.push(Statement::Print(msg.to_string()));
        }
    }

    stmts
}
