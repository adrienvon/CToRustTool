use crate::ast::*;
use crate::lexer::{Lexer, Token};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Parser { tokens, pos: 0 }
    }

    fn current_token(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.current_token() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!(
                "Expected {:?}, got {:?}",
                expected,
                self.current_token()
            ))
        }
    }

    fn parse_type(&mut self) -> Result<CType, String> {
        let typ = match self.current_token() {
            Token::Int => CType::Int,
            Token::Char => CType::Char,
            Token::Float => CType::Float,
            Token::Double => CType::Double,
            Token::Void => CType::Void,
            _ => return Err(format!("Expected type, got {:?}", self.current_token())),
        };
        self.advance();

        // 处理指针类型
        let mut result = typ;
        while self.current_token() == &Token::Star {
            self.advance();
            result = CType::Pointer(Box::new(result));
        }

        Ok(result)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current_token().clone() {
            Token::IntLiteral(n) => {
                self.advance();
                Ok(Expr::IntLiteral(n))
            }
            Token::FloatLiteral(f) => {
                self.advance();
                Ok(Expr::FloatLiteral(f))
            }
            Token::CharLiteral(c) => {
                self.advance();
                Ok(Expr::CharLiteral(c))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(Expr::StringLiteral(s))
            }
            Token::Identifier(name) => {
                self.advance();
                // 检查是否是函数调用
                if self.current_token() == &Token::LParen {
                    self.advance();
                    let mut args = Vec::new();

                    if self.current_token() != &Token::RParen {
                        args.push(self.parse_expr()?);
                        while self.current_token() == &Token::Comma {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }

                    self.expect(Token::RParen)?;
                    Ok(Expr::Call { func: name, args })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            _ => Err(format!(
                "Unexpected token in expression: {:?}",
                self.current_token()
            )),
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                })
            }
            Token::Not => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                })
            }
            Token::Star => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::Deref,
                    operand: Box::new(operand),
                })
            }
            Token::Ampersand => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::AddressOf,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.current_token() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.current_token() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;

        loop {
            let op = match self.current_token() {
                Token::Lt => BinaryOp::Lt,
                Token::Gt => BinaryOp::Gt,
                Token::Le => BinaryOp::Le,
                Token::Ge => BinaryOp::Ge,
                Token::Eq => BinaryOp::Eq,
                Token::Ne => BinaryOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_logical(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = match self.current_token() {
                Token::And => BinaryOp::And,
                Token::Or => BinaryOp::Or,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_assignment(&mut self) -> Result<Expr, String> {
        let left = self.parse_logical()?;

        if self.current_token() == &Token::Assign {
            self.advance();
            let right = self.parse_assignment()?;
            Ok(Expr::Assignment {
                target: Box::new(left),
                value: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assignment()
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current_token() {
            Token::Int | Token::Char | Token::Float | Token::Double => {
                let typ = self.parse_type()?;
                if let Token::Identifier(name) = self.current_token().clone() {
                    self.advance();
                    let init = if self.current_token() == &Token::Assign {
                        self.advance();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(Token::Semicolon)?;
                    Ok(Stmt::VarDecl { typ, name, init })
                } else {
                    Err("Expected identifier after type".to_string())
                }
            }
            Token::Return => {
                self.advance();
                let expr = if self.current_token() != &Token::Semicolon {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Return(expr))
            }
            Token::If => {
                self.advance();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;

                let then_block = if self.current_token() == &Token::LBrace {
                    self.advance();
                    let mut stmts = Vec::new();
                    while self.current_token() != &Token::RBrace
                        && self.current_token() != &Token::Eof
                    {
                        stmts.push(self.parse_statement()?);
                    }
                    self.expect(Token::RBrace)?;
                    stmts
                } else {
                    vec![self.parse_statement()?]
                };

                let else_block = if self.current_token() == &Token::Else {
                    self.advance();
                    if self.current_token() == &Token::LBrace {
                        self.advance();
                        let mut stmts = Vec::new();
                        while self.current_token() != &Token::RBrace
                            && self.current_token() != &Token::Eof
                        {
                            stmts.push(self.parse_statement()?);
                        }
                        self.expect(Token::RBrace)?;
                        Some(stmts)
                    } else {
                        Some(vec![self.parse_statement()?])
                    }
                } else {
                    None
                };

                Ok(Stmt::If {
                    cond,
                    then_block,
                    else_block,
                })
            }
            Token::While => {
                self.advance();
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;

                let body = if self.current_token() == &Token::LBrace {
                    self.advance();
                    let mut stmts = Vec::new();
                    while self.current_token() != &Token::RBrace
                        && self.current_token() != &Token::Eof
                    {
                        stmts.push(self.parse_statement()?);
                    }
                    self.expect(Token::RBrace)?;
                    stmts
                } else {
                    vec![self.parse_statement()?]
                };

                Ok(Stmt::While { cond, body })
            }
            Token::LBrace => {
                self.advance();
                let mut stmts = Vec::new();
                while self.current_token() != &Token::RBrace && self.current_token() != &Token::Eof
                {
                    stmts.push(self.parse_statement()?);
                }
                self.expect(Token::RBrace)?;
                Ok(Stmt::Block(stmts))
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_function(&mut self) -> Result<Function, String> {
        let return_type = self.parse_type()?;

        let name = if let Token::Identifier(n) = self.current_token().clone() {
            self.advance();
            n
        } else {
            return Err("Expected function name".to_string());
        };

        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        if self.current_token() != &Token::RParen {
            loop {
                let typ = self.parse_type()?;
                let param_name = if let Token::Identifier(n) = self.current_token().clone() {
                    self.advance();
                    n
                } else {
                    return Err("Expected parameter name".to_string());
                };
                params.push(Param {
                    typ,
                    name: param_name,
                });

                if self.current_token() == &Token::Comma {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;

        let mut body = Vec::new();
        while self.current_token() != &Token::RBrace && self.current_token() != &Token::Eof {
            body.push(self.parse_statement()?);
        }

        self.expect(Token::RBrace)?;

        Ok(Function {
            return_type,
            name,
            params,
            body,
        })
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut functions = Vec::new();

        while self.current_token() != &Token::Eof {
            functions.push(self.parse_function()?);
        }

        Ok(Program { functions })
    }
}
