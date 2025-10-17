/// 简单的词法分析器
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 关键字
    Int,
    Char,
    Float,
    Double,
    Void,
    If,
    Else,
    While,
    For,
    Return,

    // 标识符和字面量
    Identifier(String),
    IntLiteral(i32),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),

    // 运算符
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Not,
    Ampersand,

    // 分隔符
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,

    // 特殊
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn current_char(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char() {
            if ch.is_numeric() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            Token::FloatLiteral(num_str.parse().unwrap())
        } else {
            Token::IntLiteral(num_str.parse().unwrap())
        }
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        match ident.as_str() {
            "int" => Token::Int,
            "char" => Token::Char,
            "float" => Token::Float,
            "double" => Token::Double,
            "void" => Token::Void,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "return" => Token::Return,
            _ => Token::Identifier(ident),
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // skip opening "
        let mut string = String::new();

        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance();
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char() {
                    match escaped {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        '\\' => string.push('\\'),
                        '"' => string.push('"'),
                        _ => string.push(escaped),
                    }
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Token::StringLiteral(string)
    }

    fn read_char(&mut self) -> Token {
        self.advance(); // skip opening '
        let ch = self.current_char().unwrap_or('\0');
        self.advance();
        if self.current_char() == Some('\'') {
            self.advance();
        }
        Token::CharLiteral(ch)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current_char() {
            None => Token::Eof,
            Some(ch) => {
                if ch.is_numeric() {
                    self.read_number()
                } else if ch.is_alphabetic() || ch == '_' {
                    self.read_identifier()
                } else {
                    match ch {
                        '+' => {
                            self.advance();
                            Token::Plus
                        }
                        '-' => {
                            self.advance();
                            Token::Minus
                        }
                        '*' => {
                            self.advance();
                            Token::Star
                        }
                        '/' => {
                            self.advance();
                            Token::Slash
                        }
                        '%' => {
                            self.advance();
                            Token::Percent
                        }
                        '(' => {
                            self.advance();
                            Token::LParen
                        }
                        ')' => {
                            self.advance();
                            Token::RParen
                        }
                        '{' => {
                            self.advance();
                            Token::LBrace
                        }
                        '}' => {
                            self.advance();
                            Token::RBrace
                        }
                        ';' => {
                            self.advance();
                            Token::Semicolon
                        }
                        ',' => {
                            self.advance();
                            Token::Comma
                        }
                        '&' => {
                            self.advance();
                            if self.current_char() == Some('&') {
                                self.advance();
                                Token::And
                            } else {
                                Token::Ampersand
                            }
                        }
                        '|' => {
                            self.advance();
                            if self.current_char() == Some('|') {
                                self.advance();
                                Token::Or
                            } else {
                                Token::Eof // 暂不支持按位或
                            }
                        }
                        '!' => {
                            self.advance();
                            if self.current_char() == Some('=') {
                                self.advance();
                                Token::Ne
                            } else {
                                Token::Not
                            }
                        }
                        '=' => {
                            self.advance();
                            if self.current_char() == Some('=') {
                                self.advance();
                                Token::Eq
                            } else {
                                Token::Assign
                            }
                        }
                        '<' => {
                            self.advance();
                            if self.current_char() == Some('=') {
                                self.advance();
                                Token::Le
                            } else {
                                Token::Lt
                            }
                        }
                        '>' => {
                            self.advance();
                            if self.current_char() == Some('=') {
                                self.advance();
                                Token::Ge
                            } else {
                                Token::Gt
                            }
                        }
                        '"' => self.read_string(),
                        '\'' => self.read_char(),
                        _ => {
                            self.advance();
                            Token::Eof
                        }
                    }
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
