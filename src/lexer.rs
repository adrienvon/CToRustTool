/// 简单的词法分析器
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // 关键字 - 类型
    Int,
    Char,
    Float,
    Double,
    Void,
    Long,
    Short,
    Unsigned,
    Signed,
    Struct,
    Union,
    Enum,
    Typedef,
    Const,
    Volatile,
    Static,
    Extern,
    Auto,
    Register,

    // 关键字 - 控制流
    If,
    Else,
    While,
    Do,
    For,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Return,
    Goto,

    // 关键字 - 其他
    Sizeof,

    // 预处理器
    Include(String),
    Define(String, String),
    Ifdef,
    Ifndef,
    Endif,

    // 标识符和字面量
    Identifier(String),
    IntLiteral(i32),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),

    // 运算符 - 算术
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // 运算符 - 位运算
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    LeftShift,
    RightShift,

    // 运算符 - 赋值
    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftShiftAssign,
    RightShiftAssign,

    // 运算符 - 比较
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,

    // 运算符 - 逻辑
    And,
    Or,
    Not,

    // 运算符 - 其他
    Ampersand,
    Increment,
    Decrement,
    Arrow,
    Dot,
    Question,
    Colon,

    // 分隔符
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
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
            // 类型关键字
            "int" => Token::Int,
            "char" => Token::Char,
            "float" => Token::Float,
            "double" => Token::Double,
            "void" => Token::Void,
            "long" => Token::Long,
            "short" => Token::Short,
            "unsigned" => Token::Unsigned,
            "signed" => Token::Signed,
            "struct" => Token::Struct,
            "union" => Token::Union,
            "enum" => Token::Enum,
            "typedef" => Token::Typedef,
            "const" => Token::Const,
            "volatile" => Token::Volatile,
            "static" => Token::Static,
            "extern" => Token::Extern,
            "auto" => Token::Auto,
            "register" => Token::Register,

            // 控制流关键字
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "do" => Token::Do,
            "for" => Token::For,
            "switch" => Token::Switch,
            "case" => Token::Case,
            "default" => Token::Default,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "return" => Token::Return,
            "goto" => Token::Goto,

            // 其他关键字
            "sizeof" => Token::Sizeof,

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

    fn peek_char(&self, offset: usize) -> Option<char> {
        let peek_pos = self.pos + offset;
        if peek_pos < self.input.len() {
            Some(self.input[peek_pos])
        } else {
            None
        }
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
                            match self.current_char() {
                                Some('+') => {
                                    self.advance();
                                    Token::Increment
                                }
                                Some('=') => {
                                    self.advance();
                                    Token::PlusAssign
                                }
                                _ => Token::Plus,
                            }
                        }
                        '-' => {
                            self.advance();
                            match self.current_char() {
                                Some('-') => {
                                    self.advance();
                                    Token::Decrement
                                }
                                Some('=') => {
                                    self.advance();
                                    Token::MinusAssign
                                }
                                Some('>') => {
                                    self.advance();
                                    Token::Arrow
                                }
                                _ => Token::Minus,
                            }
                        }
                        '*' => {
                            self.advance();
                            if self.current_char() == Some('=') {
                                self.advance();
                                Token::StarAssign
                            } else {
                                Token::Star
                            }
                        }
                        '/' => {
                            self.advance();
                            if self.current_char() == Some('=') {
                                self.advance();
                                Token::SlashAssign
                            } else {
                                Token::Slash
                            }
                        }
                        '%' => {
                            self.advance();
                            if self.current_char() == Some('=') {
                                self.advance();
                                Token::PercentAssign
                            } else {
                                Token::Percent
                            }
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
                        '[' => {
                            self.advance();
                            Token::LBracket
                        }
                        ']' => {
                            self.advance();
                            Token::RBracket
                        }
                        ';' => {
                            self.advance();
                            Token::Semicolon
                        }
                        ',' => {
                            self.advance();
                            Token::Comma
                        }
                        '.' => {
                            self.advance();
                            Token::Dot
                        }
                        '?' => {
                            self.advance();
                            Token::Question
                        }
                        ':' => {
                            self.advance();
                            Token::Colon
                        }
                        '~' => {
                            self.advance();
                            Token::BitNot
                        }
                        '&' => {
                            self.advance();
                            match self.current_char() {
                                Some('&') => {
                                    self.advance();
                                    Token::And
                                }
                                Some('=') => {
                                    self.advance();
                                    Token::AndAssign
                                }
                                _ => Token::Ampersand,
                            }
                        }
                        '|' => {
                            self.advance();
                            match self.current_char() {
                                Some('|') => {
                                    self.advance();
                                    Token::Or
                                }
                                Some('=') => {
                                    self.advance();
                                    Token::OrAssign
                                }
                                _ => Token::BitOr,
                            }
                        }
                        '^' => {
                            self.advance();
                            if self.current_char() == Some('=') {
                                self.advance();
                                Token::XorAssign
                            } else {
                                Token::BitXor
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
                            match self.current_char() {
                                Some('=') => {
                                    self.advance();
                                    Token::Le
                                }
                                Some('<') => {
                                    self.advance();
                                    if self.current_char() == Some('=') {
                                        self.advance();
                                        Token::LeftShiftAssign
                                    } else {
                                        Token::LeftShift
                                    }
                                }
                                _ => Token::Lt,
                            }
                        }
                        '>' => {
                            self.advance();
                            match self.current_char() {
                                Some('=') => {
                                    self.advance();
                                    Token::Ge
                                }
                                Some('>') => {
                                    self.advance();
                                    if self.current_char() == Some('=') {
                                        self.advance();
                                        Token::RightShiftAssign
                                    } else {
                                        Token::RightShift
                                    }
                                }
                                _ => Token::Gt,
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
