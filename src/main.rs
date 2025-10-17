mod ast;
mod codegen;
mod lexer;
mod parser;

use codegen::CodeGenerator;
use parser::Parser;

fn main() {
    println!("=== C语法解析器增强测试 ===\n");

    // 测试1: 结构体定义
    let code1 = r#"
struct Point {
    int x;
    int y;
};
"#;

    println!("测试1 - 结构体定义:");
    println!("输入:\n{}", code1);
    process_code(code1);

    // 测试2: 枚举定义
    let code2 = r#"
enum Color {
    RED,
    GREEN,
    BLUE
};
"#;

    println!("\n测试2 - 枚举定义:");
    println!("输入:\n{}", code2);
    process_code(code2);

    // 测试3: Typedef
    let code3 = r#"
typedef int size_t;

int get_size() {
    int s;
    s = 100;
    return s;
}
"#;

    println!("\n测试3 - Typedef:");
    println!("输入:\n{}", code3);
    process_code(code3);

    // 测试4: For循环和break
    let code4 = r#"
int sum_numbers() {
    int sum;
    int i;
    sum = 0;
    for (i = 0; i < 10; i = i + 1) {
        if (i > 5) {
            break;
        }
        sum = sum + i;
    }
    return sum;
}
"#;

    println!("\n测试4 - For循环和break:");
    println!("输入:\n{}", code4);
    process_code(code4);

    // 测试5: 类型修饰符
    let code5 = r#"
int calculate() {
    unsigned int a;
    long c;
    a = 100;
    c = 300;
    return a + c;
}
"#;

    println!("\n测试5 - 类型修饰符:");
    println!("输入:\n{}", code5);
    process_code(code5);

    // 测试6: do-while循环
    let code6 = r#"
int sum_positive() {
    int sum;
    int i;
    sum = 0;
    i = 0;
    do {
        sum = sum + i;
        i = i + 1;
    } while (i < 10);
    return sum;
}
"#;

    println!("\n测试6 - do-while循环:");
    println!("输入:\n{}", code6);
    process_code(code6);

    // 测试7: 全局变量
    let code7 = r#"
int global_var;

int use_global() {
    global_var = 42;
    return global_var;
}
"#;

    println!("\n测试7 - 全局变量:");
    println!("输入:\n{}", code7);
    process_code(code7);

    // 测试8: 联合体
    let code8 = r#"
union Data {
    int i;
    float f;
    char c;
};
"#;

    println!("\n测试8 - 联合体:");
    println!("输入:\n{}", code8);
    process_code(code8);
}

fn process_code(code: &str) {
    let mut parser = Parser::new(code);

    match parser.parse_program() {
        Ok(program) => {
            println!("✓ AST解析成功！");
            println!("\nAST结构:");
            println!("{:#?}", program);

            let mut codegen = CodeGenerator::new();
            let generated_code = codegen.generate_program(&program);

            println!("\n生成的C代码:");
            println!("{}", generated_code);
            println!("---");
        }
        Err(e) => {
            println!("✗ 解析错误: {}", e);
            println!("---");
        }
    }
}
