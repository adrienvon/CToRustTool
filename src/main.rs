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

    // 额外：尝试解析 translate_chibicc 项目源码
    println!("\n=== 尝试解析 translate_chibicc/src 下的 .c 文件 ===\n");
    parse_translate_chibicc_dir("translate_chibicc/src");
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

fn parse_translate_chibicc_dir(dir: &str) {
    use std::fs;
    use std::path::Path;

    let path = Path::new(dir);
    let mut total = 0usize;
    let mut ok = 0usize;

    let prelude = r#"
typedef int bool;
typedef long long int64_t;
typedef unsigned long long uint64_t;
typedef int int32_t;
typedef unsigned int uint32_t;
typedef long ssize_t;
typedef unsigned long size_t;
typedef unsigned long uintptr_t;
typedef long intptr_t;
typedef unsigned char uint8_t;
typedef signed char int8_t;
typedef unsigned short uint16_t;
typedef signed short int16_t;
typedef double double_t;
typedef float float_t;
// chibicc forward-declared/user types often used across files
typedef struct Type Type;
typedef struct Node Node;
typedef struct Member Member;
typedef struct Relocation Relocation;
typedef struct Hideset Hideset;
typedef struct File File;
typedef struct Obj Obj;
typedef struct Token Token;
typedef struct StringArray StringArray;
typedef struct HashMap HashMap;
typedef struct HashEntry HashEntry;
typedef int FILE;
typedef int va_list;
typedef int NodeKind;
typedef int TokenKind;
typedef int TypeKind;
"#;

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            println!("无法读取目录 {}: {}", dir, e);
            return;
        }
    };

    for entry in entries.flatten() {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("c") {
            continue;
        }

        total += 1;
        let fname = p.display().to_string();
        match fs::read_to_string(&p) {
            Ok(src) => {
                let sanitized = sanitize_source(&src);
                let input = format!("{}\n{}", prelude, sanitized);
                let mut parser = Parser::new(&input);
                match parser.parse_program() {
                    Ok(_program) => {
                        ok += 1;
                        println!("✓ 解析成功: {}", fname);
                    }
                    Err(e) => {
                        println!("✗ 解析失败: {}\n  -> {}", fname, e);
                    }
                }
            }
            Err(e) => println!("✗ 读取失败: {} -> {}", fname, e),
        }
    }

    println!("\n统计: 成功 {}/{} 文件", ok, total);
}

fn sanitize_source(src: &str) -> String {
    // 1) 去掉预处理指令行（以#开头），并处理续行反斜杠，将整个宏定义块移除
    let mut out_lines: Vec<String> = Vec::new();
    let mut iter = src.lines();
    while let Some(line) = iter.next() {
        let t = line.trim_start();
        if t.starts_with('#') {
            // 跳过该行以及后续以反斜杠续行的行
            let prev_ends_with_bs = t.trim_end().ends_with('\\');
            if !prev_ends_with_bs {
                continue;
            }
            while let Some(next_line) = iter.next() {
                let tt = next_line.trim_end();
                let cont = tt.ends_with('\\');
                if !cont {
                    break;
                }
            }
            continue;
        }
        out_lines.push(line.to_string());
    }
    let mut s = out_lines.join("\n");

    // 2) 移除 __attribute__((...)) / __attribute__ (...) 块（简单括号匹配）
    s = remove_attribute_blocks(&s, "__attribute__");

    // 3) 移除 GCC 扩展关键字/限定符：inline, _Noreturn, noreturn, restrict
    for kw in ["inline", "_Noreturn", "noreturn", "restrict"] {
        s = replace_word(&s, kw, "");
    }

    // 4) 常见内建宏/关键字占位（如果存在，直接删除，不参与解析）
    for kw in ["__restrict", "__restrict__", "__inline", "__inline__"] {
        s = replace_word(&s, kw, "");
    }
    // 定向移除 codegen.c 中使用的宏片段（无预处理状态下无法展开）
    for kw in ["FROM_F80_1", "FROM_F80_2"] {
        s = replace_word(&s, kw, "");
    }

    // 5) 去掉常见的系统头文件 include 行（如果 sanitize 第一步遗漏了尾随空格等情况）
    let mut out2 = Vec::new();
    for line in s.lines() {
        let t = line.trim();
        if t.starts_with("#include <") || t.starts_with("# include <") {
            continue;
        }
        if t.starts_with("#define FROM_F80_1") || t.starts_with("#define FROM_F80_2") {
            continue;
        }
        out2.push(line.to_string());
    }
    s = out2.join("\n");

    s
}

fn remove_attribute_blocks(input: &str, marker: &str) -> String {
    let mut s = input.to_string();
    while let Some(pos) = s.find(marker) {
        // 找到第一个 '('
        let start_paren = match s[pos..].find('(') {
            Some(off) => pos + off,
            None => {
                s.replace_range(pos..pos + marker.len(), "");
                continue;
            }
        };
        // 匹配括号直到配平
        let mut i = start_paren;
        let mut depth = 0i32;
        while i < s.len() {
            let ch = s.as_bytes()[i] as char;
            if ch == '(' {
                depth += 1;
            }
            if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            i += 1;
        }
        let end = i.min(s.len());
        s.replace_range(pos..end, "");
    }
    s
}

fn replace_word(input: &str, word: &str, repl: &str) -> String {
    // 简单基于分隔符的词替换，避免替换到标识符子串
    let mut out = String::with_capacity(input.len());
    let mut start = 0usize;
    while let Some(pos) = input[start..].find(word) {
        let abs = start + pos;
        let left_ok = abs == 0 || !is_ident_char(input.as_bytes()[abs - 1] as char);
        let right_ok = abs + word.len() >= input.len()
            || !is_ident_char(input.as_bytes()[abs + word.len()] as char);
        if left_ok && right_ok {
            out.push_str(&input[start..abs]);
            out.push_str(repl);
            start = abs + word.len();
        } else {
            // 非独立单词，跳过该位置
            out.push_str(&input[start..=abs]);
            start = abs + 1;
        }
    }
    out.push_str(&input[start..]);
    out
}

fn is_ident_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}
