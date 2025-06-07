use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug, Clone)]
pub enum Node {
    Group(Vec<Node>),
    Box { x: i32, y: i32, width: i32, height: i32 },
    If { condition: Expr, then_body: Vec<Node>, else_body: Option<Vec<Node>> },
    Print(String),
    
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(i32),
    Binary {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
    },
}






#[derive(Debug, PartialEq, Clone)]
enum Token {
    Ident(String),
    Number(i32),
    LBrace,
    RBrace,
    LParen,
    RParen,
    Colon,
    Comma,
    EOF,
    Operator(String), 
}

struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(code: &'a str) -> Self {
        Lexer {
            input: code.chars().peekable(),
        }
    }

    fn next_token(&mut self) -> Token {
    while let Some(&c) = self.input.peek() {
        match c {
            '{' => { self.input.next(); return Token::LBrace; }
            '}' => { self.input.next(); return Token::RBrace; }
            '(' => { self.input.next(); return Token::LParen; }
            ')' => { self.input.next(); return Token::RParen; }
            ':' => { self.input.next(); return Token::Colon; }
            ',' => { self.input.next(); return Token::Comma; }
            '0'..='9' => return self.read_number(),
            c if c.is_alphabetic() => return self.read_ident(),
            c if c.is_whitespace() => {
                self.input.next(); // Skip
                continue;
            }
            // 🎯 Handle operators like >, <, ==, >=, <=
            '>' | '<' | '=' => {
                let mut op = String::new();
                op.push(c);
                self.input.next(); // consume first

                if let Some(&'=') = self.input.peek() {
                    op.push('=');
                    self.input.next(); // consume second
                }

                return Token::Operator(op);
            }
            // 🧹 Ignore unknown characters (optional: error if strict)
            _ => {
                self.input.next();
                continue;
            }
        }
    }

    Token::EOF
}

    fn read_number(&mut self) -> Token {
        let mut num = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_ascii_digit() {
                num.push(c);
                self.input.next();
            } else {
                break;
            }
        }
        Token::Number(num.parse().unwrap())
    }

    fn read_ident(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(&c) = self.input.peek() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.input.next();
            } else {
                break;
            }
        }
        Token::Ident(ident)
    }
}

pub fn parse_wpp(source: &str) -> Vec<Node> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == Token::EOF {
            break;
        }
        tokens.push(token);
    }

    let mut parser = Parser { tokens, pos: 0 };
    parser.parse_nodes()
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF)
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek();
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: Token) {
        let tok = self.advance();
        if std::mem::discriminant(&tok) != std::mem::discriminant(&expected) {
    eprintln!("DEBUG: got {:?} at pos {}, remaining = {:?}", tok, self.pos, &self.tokens[self.pos..]);
    panic!("Expected {:?}, got {:?}", expected, tok);
}

    }

    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        while self.pos < self.tokens.len() {
           match self.peek() {
    Token::Ident(ref s) if s == "if" => nodes.push(self.parse_if()),
    Token::Ident(ref s) if s == "group" => nodes.push(self.parse_group()),
    Token::Ident(ref s) if s == "box" => nodes.push(self.parse_box()),
    _ => break,
}

        }
        nodes
    }

    fn parse_group(&mut self) -> Node {
        self.expect(Token::Ident("group".to_string()));
        self.expect(Token::LBrace);

        let mut children = Vec::new();
        while self.peek() != Token::RBrace {
            if let Token::Ident(ref s) = self.peek() {
                if s == "box" {
                    children.push(self.parse_box());
                } else {
                    panic!("Unexpected identifier in group: {:?}", s);
                }
            } else {
                panic!("Unexpected token in group");
            }
        }

        self.expect(Token::RBrace);
        Node::Group(children)
    }

    fn parse_box(&mut self) -> Node {
    self.expect_ident("box");
    self.expect(Token::LParen);

    let mut x = None;
    let mut y = None;
    let mut width = None;
    let mut height = None;

    while self.peek() != Token::RParen {
        match self.advance() {
            Token::Ident(name) => {
                self.expect(Token::Colon);
                let value = self.expect_number();

                match name.as_str() {
                    "x" => x = Some(value),
                    "y" => y = Some(value),
                    "width" => width = Some(value),
                    "height" => height = Some(value),
                    _ => panic!("Unknown parameter '{}'", name),
                }

                if self.peek() == Token::Comma {
                    self.advance(); // skip comma
                }
            }
            t => panic!("Expected identifier, got {:?}", t),
        }
    }

    self.expect(Token::RParen);

    Node::Box {
        x: x.expect("Missing x"),
        y: y.expect("Missing y"),
        width: width.expect("Missing width"),
        height: height.expect("Missing height"),
    }
}

fn expect_number(&mut self) -> i32 {
    match self.advance() {
        Token::Number(n) => n,
        t => panic!("Expected number, got {:?}", t),
    }
}



    fn expect_named_number(&mut self, name: &str) -> i32 {
        let label = self.advance();
        if label != Token::Ident(name.to_string()) {
            panic!("Expected '{}:', got {:?}", name, label);
        }
        self.expect(Token::Colon);
        if let Token::Number(n) = self.advance() {
            n
        } else {
            panic!("Expected number after '{}:'", name);
        }
    }
    fn parse_if(&mut self) -> Node {
    self.expect(Token::Ident("if".into()));
    self.expect(Token::LParen);
    let condition = self.parse_expr();
    self.expect(Token::RParen);
    self.expect(Token::LBrace);

    let mut then_body = Vec::new();
    while self.peek() != Token::RBrace {
        then_body.push(self.parse_node());
    }
    self.expect(Token::RBrace);

    let else_body = if let Token::Ident(ref s) = self.peek() {
        if s == "else" {
            self.advance(); // consume 'else'
            self.expect(Token::LBrace);
            let mut body = Vec::new();
            while self.peek() != Token::RBrace {
                body.push(self.parse_node());
            }
            self.expect(Token::RBrace);
            Some(body)
        } else {
            None
        }
    } else {
        None
    };

    Node::If {
        condition,
        then_body,
        else_body,
    }
}


fn parse_expr(&mut self) -> Expr {
    let mut left = self.parse_primary();

    while let Token::Operator(op) = self.peek() {
        eprintln!("DEBUG: Parsing binary op {}", op); // ✅ Add this

        let op = match self.advance() {
            Token::Operator(s) => s,
            _ => unreachable!(),
        };

        let right = self.parse_primary();

        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        };
    }

    left
}

fn parse_primary(&mut self) -> Expr {
    match self.advance() {
        Token::Number(n) => Expr::Literal(n),
        Token::LParen => {
            let expr = self.parse_expr();
            self.expect(Token::RParen);
            expr
        }
        t => panic!("Unexpected token in primary expression: {:?}", t),
    }
}



fn parse_node(&mut self) -> Node {
    match self.peek() {
        Token::Ident(ref s) if s == "box" => self.parse_box(),
        Token::Ident(ref s) if s == "group" => self.parse_group(),
        Token::Ident(ref s) if s == "if" => self.parse_if(),
        _ => panic!("Unexpected token: {:?}", self.peek()),
    }
}
fn expect_ident(&mut self, expected: &str) {
    match self.advance() {
        Token::Ident(s) if s == expected => {},
        other => panic!("Expected identifier '{}', got {:?}", expected, other),
    }
}





}
