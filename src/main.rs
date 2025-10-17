mod ast;
mod codegen;
mod lexer;
mod parser;

use codegen::CodeGenerator;
use parser::Parser;

fn main() {
    println!("=== C to C AST Demo ===\n");

    // 示例1: 简单的函数
    let code1 = r#"
int add(int a, int b) {
    return a + b;
}
"#;

    println!("示例1 - 简单函数:");
    println!("输入:\n{}", code1);
    process_code(code1);

    // 示例2: 带变量声明和条件语句
    let code2 = r#"
int factorial(int n) {
    int result;
    result = 1;
    if (n <= 1) {
        return 1;
    }
    while (n > 1) {
        result = result * n;
        n = n - 1;
    }
    return result;
}
"#;

    println!("\n示例2 - 阶乘函数:");
    println!("输入:\n{}", code2);
    process_code(code2);

    // 示例3: 带函数调用
    let code3 = r#"
int main() {
    int x;
    x = 10;
    int y;
    y = add(x, 5);
    return y;
}
"#;

    println!("\n示例3 - 主函数:");
    println!("输入:\n{}", code3);
    process_code(code3);

    // 示例4: 复杂表达式
    let code4 = r#"
int calculate(int a, int b, int c) {
    int result;
    result = (a + b) * c - a / b;
    if (result > 100) {
        result = 100;
    }
    return result;
}
"#;

    println!("\n示例4 - 复杂表达式:");
    println!("输入:\n{}", code4);
    process_code(code4);
}

fn process_code(code: &str) {
    let mut parser = Parser::new(code);

    match parser.parse_program() {
        Ok(program) => {
            println!("AST解析成功！");
            println!("\nAST结构:");
            println!("{:#?}", program);

            let mut codegen = CodeGenerator::new();
            let generated_code = codegen.generate_program(&program);

            println!("\n生成的C代码:");
            println!("{}", generated_code);
            println!("---");
        }
        Err(e) => {
            println!("解析错误: {}", e);
        }
    }
}
