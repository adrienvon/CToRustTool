use crate::ast::*;

pub struct CodeGenerator {
    indent: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator { indent: 0 }
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn generate_type(&self, typ: &CType) -> String {
        match typ {
            CType::Int => "int".to_string(),
            CType::Char => "char".to_string(),
            CType::Float => "float".to_string(),
            CType::Double => "double".to_string(),
            CType::Void => "void".to_string(),
            CType::Pointer(inner) => format!("{}*", self.generate_type(inner)),
        }
    }

    fn generate_binary_op(&self, op: &BinaryOp) -> &str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Le => "<=",
            BinaryOp::Ge => ">=",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }

    fn generate_unary_op(&self, op: &UnaryOp) -> &str {
        match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::Deref => "*",
            UnaryOp::AddressOf => "&",
        }
    }

    fn generate_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLiteral(n) => n.to_string(),
            Expr::FloatLiteral(f) => f.to_string(),
            Expr::CharLiteral(c) => format!("'{}'", c),
            Expr::StringLiteral(s) => format!("\"{}\"", s),
            Expr::Identifier(name) => name.clone(),
            Expr::Binary { op, left, right } => {
                format!(
                    "({} {} {})",
                    self.generate_expr(left),
                    self.generate_binary_op(op),
                    self.generate_expr(right)
                )
            }
            Expr::Unary { op, operand } => {
                format!(
                    "({}{})",
                    self.generate_unary_op(op),
                    self.generate_expr(operand)
                )
            }
            Expr::Call { func, args } => {
                let args_str = args
                    .iter()
                    .map(|arg| self.generate_expr(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", func, args_str)
            }
            Expr::Assignment { target, value } => {
                format!(
                    "{} = {}",
                    self.generate_expr(target),
                    self.generate_expr(value)
                )
            }
        }
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> String {
        match stmt {
            Stmt::VarDecl { typ, name, init } => {
                let mut result =
                    format!("{}{} {}", self.indent_str(), self.generate_type(typ), name);
                if let Some(expr) = init {
                    result.push_str(&format!(" = {}", self.generate_expr(expr)));
                }
                result.push_str(";\n");
                result
            }
            Stmt::Return(expr) => {
                let mut result = format!("{}return", self.indent_str());
                if let Some(e) = expr {
                    result.push_str(&format!(" {}", self.generate_expr(e)));
                }
                result.push_str(";\n");
                result
            }
            Stmt::Expr(expr) => {
                format!("{}{};\n", self.indent_str(), self.generate_expr(expr))
            }
            Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let mut result = format!(
                    "{}if ({}) {{\n",
                    self.indent_str(),
                    self.generate_expr(cond)
                );
                self.indent += 1;
                for stmt in then_block {
                    result.push_str(&self.generate_stmt(stmt));
                }
                self.indent -= 1;
                result.push_str(&format!("{}}}", self.indent_str()));

                if let Some(else_stmts) = else_block {
                    result.push_str(" else {\n");
                    self.indent += 1;
                    for stmt in else_stmts {
                        result.push_str(&self.generate_stmt(stmt));
                    }
                    self.indent -= 1;
                    result.push_str(&format!("{}}}", self.indent_str()));
                }
                result.push('\n');
                result
            }
            Stmt::While { cond, body } => {
                let mut result = format!(
                    "{}while ({}) {{\n",
                    self.indent_str(),
                    self.generate_expr(cond)
                );
                self.indent += 1;
                for stmt in body {
                    result.push_str(&self.generate_stmt(stmt));
                }
                self.indent -= 1;
                result.push_str(&format!("{}}}\n", self.indent_str()));
                result
            }
            Stmt::For {
                init,
                cond,
                update,
                body,
            } => {
                let mut result = format!("{}for (", self.indent_str());

                if let Some(init_stmt) = init {
                    // 特殊处理 init 语句，移除缩进和换行
                    let init_str = self.generate_stmt(init_stmt).trim().to_string();
                    result.push_str(&init_str.trim_end_matches(';').to_string());
                } else {
                    result.push(';');
                }

                result.push(' ');

                if let Some(cond_expr) = cond {
                    result.push_str(&self.generate_expr(cond_expr));
                }
                result.push_str("; ");

                if let Some(update_expr) = update {
                    result.push_str(&self.generate_expr(update_expr));
                }

                result.push_str(") {\n");
                self.indent += 1;
                for stmt in body {
                    result.push_str(&self.generate_stmt(stmt));
                }
                self.indent -= 1;
                result.push_str(&format!("{}}}\n", self.indent_str()));
                result
            }
            Stmt::Block(stmts) => {
                let mut result = format!("{}{{\n", self.indent_str());
                self.indent += 1;
                for stmt in stmts {
                    result.push_str(&self.generate_stmt(stmt));
                }
                self.indent -= 1;
                result.push_str(&format!("{}}}\n", self.indent_str()));
                result
            }
        }
    }

    pub fn generate_function(&mut self, func: &Function) -> String {
        let mut result = format!("{} {}(", self.generate_type(&func.return_type), func.name);

        let params_str = func
            .params
            .iter()
            .map(|p| format!("{} {}", self.generate_type(&p.typ), p.name))
            .collect::<Vec<_>>()
            .join(", ");

        result.push_str(&params_str);
        result.push_str(") {\n");

        self.indent += 1;
        for stmt in &func.body {
            result.push_str(&self.generate_stmt(stmt));
        }
        self.indent -= 1;

        result.push_str("}\n");
        result
    }

    pub fn generate_program(&mut self, program: &Program) -> String {
        let mut result = String::new();

        for func in &program.functions {
            result.push_str(&self.generate_function(func));
            result.push('\n');
        }

        result
    }
}
