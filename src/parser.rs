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
        // 处理类型修饰符
        let mut is_const = false;
        let mut is_volatile = false;
        let mut is_unsigned = false;
        let mut is_signed = false;

        // 跳过存储类说明符
        while matches!(
            self.current_token(),
            Token::Static | Token::Extern | Token::Auto | Token::Register
        ) {
            self.advance();
        }

        // 处理const/volatile
        loop {
            match self.current_token() {
                Token::Const => {
                    is_const = true;
                    self.advance();
                }
                Token::Volatile => {
                    is_volatile = true;
                    self.advance();
                }
                Token::Unsigned => {
                    is_unsigned = true;
                    self.advance();
                }
                Token::Signed => {
                    is_signed = true;
                    self.advance();
                }
                _ => break,
            }
        }

        // 解析基本类型
        let mut typ = match self.current_token() {
            Token::Int => {
                self.advance();
                if is_unsigned {
                    CType::UnsignedInt
                } else if is_signed {
                    CType::SignedInt
                } else {
                    CType::Int
                }
            }
            Token::Char => {
                self.advance();
                if is_unsigned {
                    CType::UnsignedChar
                } else if is_signed {
                    CType::SignedChar
                } else {
                    CType::Char
                }
            }
            Token::Long => {
                self.advance();
                if is_unsigned {
                    CType::UnsignedLong
                } else {
                    CType::Long
                }
            }
            Token::Short => {
                self.advance();
                if is_unsigned {
                    CType::UnsignedShort
                } else {
                    CType::Short
                }
            }
            Token::Float => {
                self.advance();
                CType::Float
            }
            Token::Double => {
                self.advance();
                CType::Double
            }
            Token::Void => {
                self.advance();
                CType::Void
            }
            Token::Struct => {
                self.advance();
                if let Token::Identifier(name) = self.current_token().clone() {
                    self.advance();
                    CType::Struct(name)
                } else {
                    return Err("Expected struct name".to_string());
                }
            }
            Token::Union => {
                self.advance();
                if let Token::Identifier(name) = self.current_token().clone() {
                    self.advance();
                    CType::Union(name)
                } else {
                    return Err("Expected union name".to_string());
                }
            }
            Token::Enum => {
                self.advance();
                if let Token::Identifier(name) = self.current_token().clone() {
                    self.advance();
                    CType::Enum(name)
                } else {
                    return Err("Expected enum name".to_string());
                }
            }
            Token::Identifier(name) => {
                // 可能是typedef定义的类型
                let name = name.clone();
                self.advance();
                CType::Typedef(name)
            }
            _ => return Err(format!("Expected type, got {:?}", self.current_token())),
        };

        // 处理指针类型
        while self.current_token() == &Token::Star {
            self.advance();
            typ = CType::Pointer(Box::new(typ));
        }

        // 应用const/volatile修饰符
        if is_const {
            typ = CType::Const(Box::new(typ));
        }
        if is_volatile {
            typ = CType::Volatile(Box::new(typ));
        }

        Ok(typ)
    }

    // 解析结构体定义
    fn parse_struct_def(&mut self) -> Result<StructDef, String> {
        self.expect(Token::Struct)?;

        let name = if let Token::Identifier(n) = self.current_token().clone() {
            self.advance();
            n
        } else {
            return Err("Expected struct name".to_string());
        };

        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();

        while self.current_token() != &Token::RBrace && self.current_token() != &Token::Eof {
            let typ = self.parse_type()?;
            let field_name = if let Token::Identifier(n) = self.current_token().clone() {
                self.advance();
                n
            } else {
                return Err("Expected field name".to_string());
            };

            // 处理数组字段
            let field_type = if self.current_token() == &Token::LBracket {
                self.advance();
                let size = if let Token::IntLiteral(n) = self.current_token() {
                    let s = *n as usize;
                    self.advance();
                    Some(s)
                } else {
                    None
                };
                self.expect(Token::RBracket)?;
                CType::Array {
                    element_type: Box::new(typ),
                    size,
                }
            } else {
                typ
            };

            fields.push(StructField {
                typ: field_type,
                name: field_name,
            });

            self.expect(Token::Semicolon)?;
        }

        self.expect(Token::RBrace)?;

        Ok(StructDef { name, fields })
    }

    // 解析联合体定义
    fn parse_union_def(&mut self) -> Result<UnionDef, String> {
        self.expect(Token::Union)?;

        let name = if let Token::Identifier(n) = self.current_token().clone() {
            self.advance();
            n
        } else {
            return Err("Expected union name".to_string());
        };

        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();

        while self.current_token() != &Token::RBrace && self.current_token() != &Token::Eof {
            let typ = self.parse_type()?;
            let field_name = if let Token::Identifier(n) = self.current_token().clone() {
                self.advance();
                n
            } else {
                return Err("Expected field name".to_string());
            };

            fields.push(StructField {
                typ,
                name: field_name,
            });

            self.expect(Token::Semicolon)?;
        }

        self.expect(Token::RBrace)?;

        Ok(UnionDef { name, fields })
    }

    // 解析枚举定义
    fn parse_enum_def(&mut self) -> Result<EnumDef, String> {
        self.expect(Token::Enum)?;

        let name = if let Token::Identifier(n) = self.current_token().clone() {
            self.advance();
            n
        } else {
            return Err("Expected enum name".to_string());
        };

        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();

        while self.current_token() != &Token::RBrace && self.current_token() != &Token::Eof {
            let variant_name = if let Token::Identifier(n) = self.current_token().clone() {
                self.advance();
                n
            } else {
                return Err("Expected enum variant name".to_string());
            };

            let value = if self.current_token() == &Token::Assign {
                self.advance();
                if let Token::IntLiteral(n) = self.current_token() {
                    let v = *n;
                    self.advance();
                    Some(v)
                } else {
                    return Err("Expected integer literal for enum value".to_string());
                }
            } else {
                None
            };

            variants.push(EnumVariant {
                name: variant_name,
                value,
            });

            if self.current_token() == &Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        Ok(EnumDef { name, variants })
    }

    // 解析typedef定义
    fn parse_typedef(&mut self) -> Result<TypedefDef, String> {
        self.expect(Token::Typedef)?;

        let target_type = self.parse_type()?;

        let name = if let Token::Identifier(n) = self.current_token().clone() {
            self.advance();
            n
        } else {
            return Err("Expected typedef name".to_string());
        };

        self.expect(Token::Semicolon)?;

        Ok(TypedefDef { name, target_type })
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
            Token::Int
            | Token::Char
            | Token::Float
            | Token::Double
            | Token::Long
            | Token::Short
            | Token::Unsigned
            | Token::Signed
            | Token::Const
            | Token::Volatile
            | Token::Static
            | Token::Extern => {
                let typ = self.parse_type()?;
                if let Token::Identifier(name) = self.current_token().clone() {
                    self.advance();

                    // 处理数组声明
                    let final_type = if self.current_token() == &Token::LBracket {
                        self.advance();
                        let size = if let Token::IntLiteral(n) = self.current_token() {
                            let s = *n as usize;
                            self.advance();
                            Some(s)
                        } else {
                            None
                        };
                        self.expect(Token::RBracket)?;
                        CType::Array {
                            element_type: Box::new(typ),
                            size,
                        }
                    } else {
                        typ
                    };

                    let init = if self.current_token() == &Token::Assign {
                        self.advance();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(Token::Semicolon)?;
                    Ok(Stmt::VarDecl {
                        typ: final_type,
                        name,
                        init,
                    })
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
            Token::Do => {
                self.advance();
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

                self.expect(Token::While)?;
                self.expect(Token::LParen)?;
                let cond = self.parse_expr()?;
                self.expect(Token::RParen)?;
                self.expect(Token::Semicolon)?;

                Ok(Stmt::DoWhile { body, cond })
            }
            Token::For => {
                self.advance();
                self.expect(Token::LParen)?;

                // 解析初始化语句
                let init = if self.current_token() == &Token::Semicolon {
                    self.advance();
                    None
                } else {
                    let stmt = self.parse_statement()?;
                    Some(Box::new(stmt))
                };

                // 解析条件表达式
                let cond = if self.current_token() == &Token::Semicolon {
                    self.advance();
                    None
                } else {
                    let expr = self.parse_expr()?;
                    self.expect(Token::Semicolon)?;
                    Some(expr)
                };

                // 解析更新表达式
                let update = if self.current_token() == &Token::RParen {
                    None
                } else {
                    Some(self.parse_expr()?)
                };

                self.expect(Token::RParen)?;

                // 解析循环体
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

                Ok(Stmt::For {
                    init,
                    cond,
                    update,
                    body,
                })
            }
            Token::Break => {
                self.advance();
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Break)
            }
            Token::Continue => {
                self.advance();
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Continue)
            }
            Token::Goto => {
                self.advance();
                if let Token::Identifier(label) = self.current_token().clone() {
                    self.advance();
                    self.expect(Token::Semicolon)?;
                    Ok(Stmt::Goto(label))
                } else {
                    Err("Expected label after goto".to_string())
                }
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

    // 解析顶层声明（函数、结构体、枚举等）
    fn parse_declaration(&mut self) -> Result<Declaration, String> {
        match self.current_token() {
            Token::Struct => {
                let struct_def = self.parse_struct_def()?;
                // 可能有分号
                if self.current_token() == &Token::Semicolon {
                    self.advance();
                }
                Ok(Declaration::Struct(struct_def))
            }
            Token::Union => {
                let union_def = self.parse_union_def()?;
                if self.current_token() == &Token::Semicolon {
                    self.advance();
                }
                Ok(Declaration::Union(union_def))
            }
            Token::Enum => {
                let enum_def = self.parse_enum_def()?;
                if self.current_token() == &Token::Semicolon {
                    self.advance();
                }
                Ok(Declaration::Enum(enum_def))
            }
            Token::Typedef => {
                let typedef_def = self.parse_typedef()?;
                Ok(Declaration::Typedef(typedef_def))
            }
            _ => {
                // 尝试解析函数或全局变量
                let return_type = self.parse_type()?;

                let name = if let Token::Identifier(n) = self.current_token().clone() {
                    self.advance();
                    n
                } else {
                    return Err("Expected identifier".to_string());
                };

                // 检查是否是函数（有左括号）
                if self.current_token() == &Token::LParen {
                    // 解析函数
                    self.advance();
                    let mut params = Vec::new();

                    if self.current_token() != &Token::RParen {
                        loop {
                            let typ = self.parse_type()?;
                            let param_name =
                                if let Token::Identifier(n) = self.current_token().clone() {
                                    self.advance();
                                    n
                                } else {
                                    // 参数可以没有名字
                                    String::new()
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

                    // 检查是函数声明还是函数定义
                    if self.current_token() == &Token::Semicolon {
                        // 函数声明，跳过
                        self.advance();
                        // 创建空函数体的函数（或者可以选择不添加到declarations中）
                        Ok(Declaration::Function(Function {
                            return_type,
                            name,
                            params,
                            body: Vec::new(),
                        }))
                    } else {
                        // 函数定义
                        self.expect(Token::LBrace)?;

                        let mut body = Vec::new();
                        while self.current_token() != &Token::RBrace
                            && self.current_token() != &Token::Eof
                        {
                            body.push(self.parse_statement()?);
                        }

                        self.expect(Token::RBrace)?;

                        Ok(Declaration::Function(Function {
                            return_type,
                            name,
                            params,
                            body,
                        }))
                    }
                } else {
                    // 全局变量
                    let init = if self.current_token() == &Token::Assign {
                        self.advance();
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };

                    self.expect(Token::Semicolon)?;

                    Ok(Declaration::GlobalVar {
                        typ: return_type,
                        name,
                        init,
                    })
                }
            }
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut declarations = Vec::new();

        while self.current_token() != &Token::Eof {
            declarations.push(self.parse_declaration()?);
        }

        Ok(Program { declarations })
    }
}
