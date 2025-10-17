mod ast;
mod codegen;
mod lexer;
mod parser;

use codegen::CodeGenerator;
use parser::Parser;

fn main() {
    println!("=== C表达式解析增强测试 ===\n");

    // 测试1: 类型转换和malloc
    let code1 = r#"
int main() {
    int* p = (int*)malloc(sizeof(int));
    *p = 42;
    return 0;
}
"#;

    println!("测试1 - 类型转换和malloc:");
    println!("输入:\n{}", code1);
    process_code(code1);

    // 测试2: 数组访问
    let code2 = r#"
int main() {
    int arr[10];
    arr[0] = 1;
    arr[1] = 2;
    int sum = arr[0] + arr[1];
    return sum;
}
"#;

    println!("\n测试2 - 数组访问:");
    println!("输入:\n{}", code2);
    process_code(code2);

    // 测试3: 结构体成员访问
    let code3 = r#"
struct Point {
    int x;
    int y;
};

int main() {
    struct Point p;
    p.x = 10;
    p.y = 20;
    return p.x + p.y;
}
"#;

    println!("\n测试3 - 结构体成员访问:");
    println!("输入:\n{}", code3);
    process_code(code3);

    // 测试4: 指针成员访问
    let code4 = r#"
struct Point {
    int x;
    int y;
};

int main() {
    struct Point* p;
    p->x = 10;
    p->y = 20;
    return p->x + p->y;
}
"#;

    println!("\n测试4 - 指针成员访问:");
    println!("输入:\n{}", code4);
    process_code(code4);

    // 测试5: 递增递减运算符
    let code5 = r#"
int main() {
    int i = 0;
    i++;
    ++i;
    i--;
    --i;
    return i;
}
"#;

    println!("\n测试5 - 递增递减运算符:");
    println!("输入:\n{}", code5);
    process_code(code5);

    // 测试6: 位运算符
    let code6 = r#"
int main() {
    int a = 5;
    int b = 3;
    int c = a & b;
    int d = a | b;
    int e = a ^ b;
    int f = ~a;
    int g = a << 2;
    int h = a >> 1;
    return 0;
}
"#;

    println!("\n测试6 - 位运算符:");
    println!("输入:\n{}", code6);
    process_code(code6);

    // 测试7: 三元运算符
    let code7 = r#"
int main() {
    int a = 5;
    int b = 10;
    int max = (a > b) ? a : b;
    return max;
}
"#;

    println!("\n测试7 - 三元运算符:");
    println!("输入:\n{}", code7);
    process_code(code7);

    // 测试8: 指针解引用和取地址
    let code8 = r#"
int main() {
    int x = 42;
    int* p = &x;
    int y = *p;
    *p = 100;
    return y;
}
"#;

    println!("\n测试8 - 指针解引用和取地址:");
    println!("输入:\n{}", code8);
    process_code(code8);

    // 测试9: 复杂表达式组合
    let code9 = r#"
struct Node {
    int value;
    struct Node* next;
};

int main() {
    struct Node* head = (struct Node*)malloc(sizeof(struct Node));
    head->value = 42;
    int arr[10];
    arr[0] = head->value;
    int result = (arr[0] > 0) ? arr[0] * 2 : 0;
    return result;
}
"#;

    println!("\n测试9 - 复杂表达式组合:");
    println!("输入:\n{}", code9);
    process_code(code9);
}

fn process_code(code: &str) {
    let mut parser = Parser::new(code);
    match parser.parse_program() {
        Ok(program) => {
            println!("✓ 解析成功!");
            println!("AST: {:#?}\n", program);

            let mut codegen = CodeGenerator::new();
            let generated = codegen.generate_program(&program);
            println!("生成的C代码:");
            println!("{}", generated);
        }
        Err(e) => {
            println!("✗ 解析失败: {}", e);
        }
    }
}
