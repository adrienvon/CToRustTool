use crate::ast::*;
use crate::lexer::{Lexer, Token};
use std::collections::HashSet;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    typedef_names: HashSet<String>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        Parser {
            tokens,
            pos: 0,
            typedef_names: HashSet::new(),
        }
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
        // 存储类说明符（丢弃）
        while matches!(
            self.current_token(),
            Token::Static | Token::Extern | Token::Auto | Token::Register
        ) {
            self.advance();
        }

        // 类型修饰/说明收集
        let mut is_const = false;
        let mut is_volatile = false;
        let mut is_unsigned = false;
        let mut is_signed = false;
        let mut saw_char = false;
        let mut saw_float = false;
        let mut saw_double = false;
        let mut saw_void = false;
        let mut long_count: u8 = 0; // 支持 long long
        let mut saw_short = false;

        // 基础类型（可能来自 struct/union/enum/typedef 或组合关键字）
        let mut base_type: Option<CType> = None;
        let mut consumed_any = false;
        loop {
            match self.current_token().clone() {
                Token::Const => {
                    is_const = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Volatile => {
                    is_volatile = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Unsigned => {
                    is_unsigned = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Signed => {
                    is_signed = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Long => {
                    long_count = long_count.saturating_add(1);
                    self.advance();
                    consumed_any = true;
                }
                Token::Short => {
                    saw_short = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Int => {
                    /* mark int */
                    self.advance();
                    consumed_any = true;
                }
                Token::Char => {
                    saw_char = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Float => {
                    saw_float = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Double => {
                    saw_double = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Void => {
                    saw_void = true;
                    self.advance();
                    consumed_any = true;
                }
                Token::Struct => {
                    self.advance();
                    match self.current_token().clone() {
                        Token::Identifier(name) => {
                            self.advance();
                            base_type = Some(CType::Struct(name));
                            consumed_any = true;
                        }
                        Token::LBrace => {
                            // 内联结构体定义，跳过块，作为匿名类型处理
                            self.skip_brace_block()?;
                            base_type = Some(CType::Struct(String::new()));
                            consumed_any = true;
                        }
                        _ => return Err("Expected struct name".to_string()),
                    }
                }
                Token::Union => {
                    self.advance();
                    match self.current_token().clone() {
                        Token::Identifier(name) => {
                            self.advance();
                            base_type = Some(CType::Union(name));
                            consumed_any = true;
                        }
                        Token::LBrace => {
                            self.skip_brace_block()?;
                            base_type = Some(CType::Union(String::new()));
                            consumed_any = true;
                        }
                        _ => return Err("Expected union name".to_string()),
                    }
                }
                Token::Enum => {
                    self.advance();
                    match self.current_token().clone() {
                        Token::Identifier(name) => {
                            self.advance();
                            base_type = Some(CType::Enum(name));
                            consumed_any = true;
                        }
                        Token::LBrace => {
                            self.skip_brace_block()?;
                            base_type = Some(CType::Enum(String::new()));
                            consumed_any = true;
                        }
                        _ => return Err("Expected enum name".to_string()),
                    }
                }
                Token::Identifier(name) => {
                    if self.typedef_names.contains(&name) {
                        self.advance();
                        base_type = Some(CType::Typedef(name));
                        consumed_any = true;
                    } else {
                        break;
                    }
                }
                // 暂不支持在无符号表情况下将任意 Identifier 视为 typedef 类型，避免吞掉声明中的变量名
                _ => break,
            }
        }

        if !consumed_any {
            return Err(format!("Expected type, got {:?}", self.current_token()));
        }

        // 归一化推导基本类型（当未通过 struct/union/enum/typedef 指定时）
        let mut typ = if let Some(bt) = base_type {
            bt
        } else if saw_char {
            if is_unsigned {
                CType::UnsignedChar
            } else if is_signed {
                CType::SignedChar
            } else {
                CType::Char
            }
        } else if saw_double {
            // long double 简化为 Double
            CType::Double
        } else if saw_float {
            CType::Float
        } else if saw_void {
            CType::Void
        } else {
            // int 系：考虑 short / long / signed / unsigned
            if saw_short {
                if is_unsigned {
                    CType::UnsignedShort
                } else {
                    CType::Short
                }
            } else if long_count > 0 {
                if is_unsigned {
                    CType::UnsignedLong
                } else {
                    CType::Long
                }
            } else {
                if is_unsigned {
                    CType::UnsignedInt
                } else if is_signed {
                    CType::SignedInt
                } else {
                    CType::Int
                }
            }
        };

        // 指针星号
        while self.current_token() == &Token::Star {
            self.advance();
            typ = CType::Pointer(Box::new(typ));
        }

        // 应用 const/volatile（简单包裹）
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
            let basety = self.parse_type()?;
            let (field_name, field_type) = self.parse_declarator(basety)?;
            self.expect(Token::Semicolon)?;
            fields.push(StructField {
                typ: field_type,
                name: field_name,
            });
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
            let basety = self.parse_type()?;
            let (field_name, field_type) = self.parse_declarator(basety)?;
            self.expect(Token::Semicolon)?;
            fields.push(StructField {
                typ: field_type,
                name: field_name,
            });
        }

        self.expect(Token::RBrace)?;

        Ok(UnionDef { name, fields })
    }

    // 解析枚举定义
    fn parse_enum_def(&mut self) -> Result<EnumDef, String> {
        self.expect(Token::Enum)?;

        // 允许匿名枚举：enum { ... }
        let name = if let Token::Identifier(n) = self.current_token().clone() {
            self.advance();
            n
        } else {
            String::new()
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
        // 专门处理 typedef 与 struct/union/enum 组合的几种形式：
        //   typedef struct { ... } Name;
        //   typedef struct Tag { ... } Name;
        //   typedef struct Tag Name;
        //   typedef enum { ... } Name;  等
        match self.current_token().clone() {
            Token::Struct | Token::Union | Token::Enum => {
                // 记录哪一种
                let kind = self.current_token().clone();
                self.advance();

                // 可选的标签名
                let mut tag_name: Option<String> = None;
                if let Token::Identifier(n) = self.current_token().clone() {
                    // 下一个如果是标识符且后续不是 "(" 之类，则视为标签名
                    tag_name = Some(n.clone());
                    self.advance();
                }

                // 如遇到内联定义，跳过 { ... }
                if self.current_token() == &Token::LBrace {
                    self.skip_brace_block()?;
                }

                // 基础类型（匿名时可临时以别名名作为类型名占位，稍后由 declarator 返回 name）
                let base = match kind {
                    Token::Struct => CType::Struct(tag_name.unwrap_or_else(|| "".to_string())),
                    Token::Union => CType::Union(tag_name.unwrap_or_else(|| "".to_string())),
                    Token::Enum => CType::Enum(tag_name.unwrap_or_else(|| "".to_string())),
                    _ => unreachable!(),
                };

                // 读取 declarator，拿到名字与可能的数组/函数等后缀
                let (name, target_type) = self.parse_declarator(base)?;

                self.expect(Token::Semicolon)?;
                // 记录 typedef 名称
                self.typedef_names.insert(name.clone());
                Ok(TypedefDef { name, target_type })
            }
            _ => {
                // 常规形式：typedef <type> declarator (, declarator)* ;
                let base_type = self.parse_type()?;
                let base_clone = base_type.clone();
                let (name, target_type) = self.parse_declarator(base_type)?;
                self.typedef_names.insert(name.clone());
                // 额外 typedef 名称仅加入表中
                while self.current_token() == &Token::Comma {
                    self.advance();
                    let (n2, _t2) = self.parse_declarator(base_clone.clone())?;
                    self.typedef_names.insert(n2);
                }
                self.expect(Token::Semicolon)?;
                Ok(TypedefDef { name, target_type })
            }
        }
    }

    // 解析 declarator 的后缀部分：
    // - 数组声明： [N]
    // - 函数类型： (param_types)
    fn parse_declarator_suffix(&mut self, mut base: CType) -> Result<CType, String> {
        loop {
            match self.current_token() {
                Token::LBracket => {
                    self.advance();
                    let size = if let Token::IntLiteral(n) = self.current_token() {
                        let s = *n as usize;
                        self.advance();
                        Some(s)
                    } else {
                        // 允许不写大小，如 typedef int T[]; 简化为 None
                        None
                    };
                    self.expect(Token::RBracket)?;
                    base = CType::Array {
                        element_type: Box::new(base),
                        size,
                    };
                }
                Token::LParen => {
                    // 函数类型声明：返回类型为当前 base
                    self.advance();
                    let mut params: Vec<CType> = Vec::new();
                    if self.current_token() != &Token::RParen {
                        loop {
                            // 处理可变参数 ...
                            if self.current_token() == &Token::Ellipsis {
                                // 记录为一个特殊的占位类型：用 "..." 的 typedef 名占位以保留信息
                                self.advance();
                                // 我们用 void 类型作为占位，不影响后续流程
                                //（当前实现不真正使用参数类型信息进行代码生成）
                                // 不再接受更多参数
                                break;
                            }

                            let pty = self.parse_type()?;
                            // 可选的参数名（忽略）
                            if let Token::Identifier(_) = self.current_token() {
                                self.advance();
                            }
                            params.push(pty);
                            if self.current_token() == &Token::Comma {
                                self.advance();
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(Token::RParen)?;
                    base = CType::Function {
                        return_type: Box::new(base),
                        params,
                    };
                }
                _ => break,
            }
        }
        Ok(base)
    }

    // 解析 C declarator，返回 (名称, 完整类型)
    // 支持形式： ident 后接 []/() 后缀；以及括号包裹的 declarator（如 (*fn)(T)）
    fn parse_declarator(&mut self, base: CType) -> Result<(String, CType), String> {
        // 先解析可选的指针前缀（例如 `*`、`**`）
        let mut ty = base;
        while self.current_token() == &Token::Star {
            self.advance();
            ty = CType::Pointer(Box::new(ty));
        }

        // 解析直接声明子句：标识符 或 (declarator)
        let (name, mut ty) = match self.current_token().clone() {
            Token::Identifier(n) => {
                self.advance();
                (n, ty)
            }
            Token::LParen => {
                // 括号中的 declarator 可以携带自己的指针前缀
                self.advance();
                let (n, inner_ty) = self.parse_declarator(ty)?;
                self.expect(Token::RParen)?;
                (n, inner_ty)
            }
            _ => {
                return Err(format!(
                    "Expected typedef name, got {:?}",
                    self.current_token()
                ))
            }
        };

        // 解析后缀：数组或函数参数列表
        ty = self.parse_declarator_suffix(ty)?;

        Ok((name, ty))
    }

    // 跳过一个用大括号包裹的块（支持嵌套）
    fn skip_brace_block(&mut self) -> Result<(), String> {
        self.expect(Token::LBrace)?;
        let mut depth: i32 = 1;
        while depth > 0 {
            match self.current_token() {
                Token::LBrace => {
                    depth += 1;
                    self.advance();
                }
                Token::RBrace => {
                    depth -= 1;
                    self.advance();
                }
                Token::Eof => {
                    // 清洗过的源码可能丢失配对的 '}'，此处容错退出
                    break;
                }
                _ => self.advance(),
            }
        }
        Ok(())
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
                let mut acc = s;
                // C 允许相邻字符串字面量在词法阶段进行拼接
                while let Token::StringLiteral(s2) = self.current_token().clone() {
                    self.advance();
                    acc.push_str(&s2);
                }
                Ok(Expr::StringLiteral(acc))
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
                // 为了区分 (type)expr 与 (expr)，先消耗 '('
                self.advance();
                // GNU 扩展：语句表达式 ({ ... })
                if self.current_token() == &Token::LBrace {
                    // 消耗一个块，直到 '}'，然后期望 ')'
                    self.skip_brace_block()?;
                    self.expect(Token::RParen)?;
                    return Ok(Expr::Null);
                }

                // 仅当后续是明确的类型关键字或已知 typedef 名称时，按类型转换/复合字面量处理
                if self.is_type_keyword()
                    || matches!(self.current_token(), Token::Identifier(name) if self.typedef_names.contains(name))
                {
                    let typ = self.parse_type()?;
                    self.expect(Token::RParen)?;
                    // 复合字面量 (Type){ ... }
                    if self.current_token() == &Token::LBrace {
                        self.skip_brace_block()?;
                        return Ok(Expr::Null);
                    }
                    let expr = self.parse_unary()?;
                    Ok(Expr::Cast {
                        typ,
                        expr: Box::new(expr),
                    })
                } else {
                    // 否则是普通括号表达式
                    let expr = self.parse_expr()?;
                    self.expect(Token::RParen)?;
                    Ok(expr)
                }
            }
            Token::Sizeof => {
                self.advance();
                if self.current_token() == &Token::LParen {
                    self.advance();
                    if self.is_type_keyword()
                        || matches!(self.current_token(), Token::Identifier(name) if self.typedef_names.contains(name))
                    {
                        let typ = self.parse_type()?;
                        self.expect(Token::RParen)?;
                        Ok(Expr::SizeOf(typ))
                    } else {
                        // sizeof(表达式)
                        let _ = self.parse_expr()?;
                        self.expect(Token::RParen)?;
                        Ok(Expr::Null)
                    }
                } else {
                    // sizeof 后直接接一元表达式（如 sizeof *p）
                    let _ = self.parse_unary()?;
                    Ok(Expr::Null)
                }
            }
            _ => Err(format!(
                "Unexpected token in expression: {:?}",
                self.current_token()
            )),
        }
    }

    // 辅助函数：检查当前token是否是类型关键字
    fn is_type_keyword(&self) -> bool {
        matches!(
            self.current_token(),
            Token::Int
                | Token::Char
                | Token::Float
                | Token::Double
                | Token::Void
                | Token::Long
                | Token::Short
                | Token::Unsigned
                | Token::Signed
                | Token::Const
                | Token::Volatile
                | Token::Struct
                | Token::Union
                | Token::Enum
        )
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
            Token::BitNot => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::BitNot,
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
            Token::Increment => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::PreIncrement,
                    operand: Box::new(operand),
                })
            }
            Token::Decrement => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::Unary {
                    op: UnaryOp::PreDecrement,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    // 新增：处理后缀表达式（数组访问、成员访问、后缀++/--）
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current_token() {
                Token::LBracket => {
                    // 数组访问 arr[index]
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(Token::RBracket)?;
                    expr = Expr::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                Token::Dot => {
                    // 结构体成员访问 obj.member
                    self.advance();
                    if let Token::Identifier(member) = self.current_token().clone() {
                        self.advance();
                        expr = Expr::MemberAccess {
                            object: Box::new(expr),
                            member,
                        };
                    } else {
                        return Err(format!(
                            "Expected identifier after '.', got {:?}",
                            self.current_token()
                        ));
                    }
                }
                Token::Arrow => {
                    // 指针成员访问 ptr->member
                    self.advance();
                    if let Token::Identifier(member) = self.current_token().clone() {
                        self.advance();
                        expr = Expr::PointerMemberAccess {
                            object: Box::new(expr),
                            member,
                        };
                    } else {
                        return Err(format!(
                            "Expected identifier after '->', got {:?}",
                            self.current_token()
                        ));
                    }
                }
                Token::Increment => {
                    // 后缀递增 x++
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostIncrement,
                        operand: Box::new(expr),
                    };
                }
                Token::Decrement => {
                    // 后缀递减 x--
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostDecrement,
                        operand: Box::new(expr),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
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
        let mut left = self.parse_shift()?;

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
            let right = self.parse_shift()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 新增：位移运算符 << >>
    fn parse_shift(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;

        loop {
            let op = match self.current_token() {
                Token::LeftShift => BinaryOp::LeftShift,
                Token::RightShift => BinaryOp::RightShift,
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
        let mut left = self.parse_bitwise_or()?;

        loop {
            let op = match self.current_token() {
                Token::And => BinaryOp::And,
                Token::Or => BinaryOp::Or,
                _ => break,
            };
            self.advance();
            let right = self.parse_bitwise_or()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 新增：位或运算 |
    fn parse_bitwise_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bitwise_xor()?;

        while self.current_token() == &Token::BitOr {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            left = Expr::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 新增：位异或运算 ^
    fn parse_bitwise_xor(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bitwise_and()?;

        while self.current_token() == &Token::BitXor {
            self.advance();
            let right = self.parse_bitwise_and()?;
            left = Expr::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // 新增：位与运算 &
    fn parse_bitwise_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;

        while self.current_token() == &Token::Ampersand {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_assignment(&mut self) -> Result<Expr, String> {
        let left = self.parse_ternary()?;

        // 处理赋值与复合赋值
        let make_assign = |target: Expr, value: Expr| -> Expr {
            Expr::Assignment {
                target: Box::new(target),
                value: Box::new(value),
            }
        };

        match self.current_token() {
            Token::Assign => {
                self.advance();
                let right = self.parse_assignment()?;
                Ok(make_assign(left, right))
            }
            Token::PlusAssign
            | Token::MinusAssign
            | Token::StarAssign
            | Token::SlashAssign
            | Token::PercentAssign
            | Token::AndAssign
            | Token::OrAssign
            | Token::XorAssign
            | Token::LeftShiftAssign
            | Token::RightShiftAssign => {
                // 将 a += b 降级为 a = a + b（等价）
                let op_token = self.current_token().clone();
                self.advance();
                let right = self.parse_assignment()?;
                let bin_op = match op_token {
                    Token::PlusAssign => BinaryOp::Add,
                    Token::MinusAssign => BinaryOp::Sub,
                    Token::StarAssign => BinaryOp::Mul,
                    Token::SlashAssign => BinaryOp::Div,
                    Token::PercentAssign => BinaryOp::Mod,
                    Token::AndAssign => BinaryOp::BitAnd,
                    Token::OrAssign => BinaryOp::BitOr,
                    Token::XorAssign => BinaryOp::BitXor,
                    Token::LeftShiftAssign => BinaryOp::LeftShift,
                    Token::RightShiftAssign => BinaryOp::RightShift,
                    _ => unreachable!(),
                };
                let value = Expr::Binary {
                    op: bin_op,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                };
                Ok(make_assign(left, value))
            }
            _ => Ok(left),
        }
    }

    // 新增：三元运算符 ? :
    fn parse_ternary(&mut self) -> Result<Expr, String> {
        let cond = self.parse_logical()?;

        if self.current_token() == &Token::Question {
            self.advance();
            let then_expr = self.parse_expr()?;
            self.expect(Token::Colon)?;
            let else_expr = self.parse_ternary()?;
            Ok(Expr::Ternary {
                cond: Box::new(cond),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            })
        } else {
            Ok(cond)
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_assignment()
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match self.current_token() {
            // 基础类型关键字开头的声明
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
            | Token::Extern
            | Token::Struct
            | Token::Union
            | Token::Enum => {
                // 局部变量声明，支持逗号分隔的多个声明符
                let basety = self.parse_type()?;
                let base_clone = basety.clone();
                let mut decls: Vec<Stmt> = Vec::new();
                // 第一个声明符
                {
                    let (name, final_type) = self.parse_declarator(basety)?;
                    let init = if self.current_token() == &Token::Assign {
                        self.advance();
                        if self.current_token() == &Token::LBrace {
                            // 跳过聚合初始化器 { ... }
                            self.skip_brace_block()?;
                            None
                        } else {
                            Some(self.parse_expr()?)
                        }
                    } else {
                        None
                    };
                    decls.push(Stmt::VarDecl {
                        typ: final_type,
                        name,
                        init,
                    });
                }
                // 额外的逗号后续声明符（丢入同一块中）
                while self.current_token() == &Token::Comma {
                    self.advance();
                    let (name, final_type) = self.parse_declarator(base_clone.clone())?;
                    let init = if self.current_token() == &Token::Assign {
                        self.advance();
                        if self.current_token() == &Token::LBrace {
                            self.skip_brace_block()?;
                            None
                        } else {
                            Some(self.parse_expr()?)
                        }
                    } else {
                        None
                    };
                    decls.push(Stmt::VarDecl {
                        typ: final_type,
                        name,
                        init,
                    });
                }
                self.expect(Token::Semicolon)?;
                if decls.len() == 1 {
                    Ok(decls.remove(0))
                } else {
                    Ok(Stmt::Block(decls))
                }
            }
            // 以 typedef 名称开头的声明
            Token::Identifier(_) if matches!(self.current_token(), Token::Identifier(name) if self.typedef_names.contains(name)) =>
            {
                let basety = self.parse_type()?;
                let (name, final_type) = self.parse_declarator(basety)?;
                let init = if self.current_token() == &Token::Assign {
                    self.advance();
                    if self.current_token() == &Token::LBrace {
                        self.skip_brace_block()?;
                        None
                    } else {
                        Some(self.parse_expr()?)
                    }
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::VarDecl {
                    typ: final_type,
                    name,
                    init,
                })
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
            Token::Switch => {
                // 简化支持：消费 switch (<expr>) { ... }，将其作为一个空语句占位
                self.advance();
                self.expect(Token::LParen)?;
                // 条件表达式
                let _ = self.parse_expr()?;
                self.expect(Token::RParen)?;
                if self.current_token() == &Token::LBrace {
                    // 跳过整个 switch 块
                    self.skip_brace_block()?;
                } else {
                    // 如果不是块，尽量消费一个语句（容错）
                    let _ = self.parse_statement()?;
                }
                Ok(Stmt::Empty)
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
                // 尝试解析函数或全局变量：使用 declarator 支持指针/数组/函数声明
                let base_type = self.parse_type()?;
                let base_clone = base_type.clone();
                let (name, full_type) = self.parse_declarator(base_type)?;

                // 函数声明或定义
                if let CType::Function {
                    return_type,
                    params: param_types,
                } = full_type.clone()
                {
                    // 参数名在当前实现中忽略，使用空名
                    let params: Vec<Param> = param_types
                        .into_iter()
                        .map(|t| Param {
                            typ: t,
                            name: String::new(),
                        })
                        .collect();

                    if self.current_token() == &Token::Semicolon {
                        self.advance();
                        return Ok(Declaration::Function(Function {
                            return_type: *return_type,
                            name,
                            params,
                            body: Vec::new(),
                        }));
                    }

                    // 函数定义
                    self.expect(Token::LBrace)?;
                    let mut body = Vec::new();
                    while self.current_token() != &Token::RBrace
                        && self.current_token() != &Token::Eof
                    {
                        body.push(self.parse_statement()?);
                    }
                    self.expect(Token::RBrace)?;
                    return Ok(Declaration::Function(Function {
                        return_type: *return_type,
                        name,
                        params,
                        body,
                    }));
                }

                // 全局变量：支持逗号分隔的多个声明符。我们仅返回第一个，其余的消费但丢弃。
                let init = if self.current_token() == &Token::Assign {
                    self.advance();
                    if self.current_token() == &Token::LBrace {
                        // 跳过全局变量的聚合初始化器 { ... }
                        self.skip_brace_block()?;
                        None
                    } else {
                        Some(self.parse_expr()?)
                    }
                } else {
                    None
                };

                // 吃掉逗号分隔的其他声明（丢弃）
                while self.current_token() == &Token::Comma {
                    self.advance();
                    let (_name2, _type2) = self.parse_declarator(base_clone.clone())?;
                    if self.current_token() == &Token::Assign {
                        self.advance();
                        if self.current_token() == &Token::LBrace {
                            self.skip_brace_block()?;
                        } else {
                            // 丢弃一个表达式初始化器
                            let _ = self.parse_expr()?;
                        }
                    }
                }

                self.expect(Token::Semicolon)?;

                Ok(Declaration::GlobalVar {
                    typ: full_type,
                    name,
                    init,
                })
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
