# C到Rust转译工具开发进度

## 项目概述

本项目旨在开发一个能够将C语言代码转译为Rust代码的工具，重点支持两个测试项目：
- **translate_chibicc**: C编译器项目（~236KB，17个文件）
- **translate_littlefs_fuse**: littlefs FUSE文件系统（~291KB，13个文件，核心5593行）

## 已完成工作

### ✅ 阶段1：基础框架搭建（已完成）

#### 1. AST数据结构扩展
- ✅ 基本类型：int, char, float, double, void
- ✅ 扩展类型：long, short, unsigned, signed
- ✅ 复合类型：指针、数组、函数指针
- ✅ 用户定义类型：struct, union, enum, typedef
- ✅ 类型修饰符：const, volatile

#### 2. 表达式系统
- ✅ 字面量：整数、浮点数、字符、字符串
- ✅ 基本运算符：算术、比较、逻辑
- ✅ 位运算符：&, |, ^, ~, <<, >>
- ✅ 复合赋值：+=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=
- ✅ 一元运算符：-, !, ~, *, &, ++, --
- ✅ 其他表达式：类型转换、数组访问、成员访问、三元运算符、sizeof

#### 3. 语句系统
- ✅ 变量声明和初始化
- ✅ 控制流：if-else, while, do-while, for, switch-case
- ✅ 跳转：break, continue, goto, return
- ✅ 代码块

#### 4. 声明系统
- ✅ 函数声明
- ✅ 结构体定义
- ✅ 联合体定义
- ✅ 枚举定义
- ✅ Typedef定义
- ✅ 全局变量
- ✅ 预处理器指令框架

#### 5. 词法分析器
- ✅ 完整的关键字识别（30+个）
- ✅ 所有运算符支持（40+个）
- ✅ 字面量解析（整数、浮点、字符、字符串）
- ✅ 转义字符处理
- ✅ 分隔符识别

#### 6. 代码生成器
- ✅ C代码生成（所有已实现的AST节点）
- ✅ 格式化和缩进
- ✅ 运算符优先级处理

#### 7. Demo验证
- ✅ 简单函数解析
- ✅ 带条件和循环的复杂函数
- ✅ 函数调用
- ✅ 复杂表达式运算
- ✅ AST可视化输出

## 待完成工作

### 🔄 阶段2：语法分析器增强（进行中）

需要实现的解析功能：
- [ ] 结构体定义解析
- [ ] 联合体定义解析
- [ ] 枚举定义解析  
- [ ] Typedef解析
- [ ] 数组声明解析（包括多维数组）
- [ ] 函数指针解析
- [ ] 指针数组、数组指针等复杂类型
- [ ] 全局变量声明
- [ ] 预处理器指令（#include, #define, #ifdef等）
- [ ] 注释处理（单行//和多行/**/）
- [ ] 初始化列表
- [ ] 复合字面量

### 📋 阶段3：类型系统和符号表

- [ ] 符号表实现
  - [ ] 作用域管理
  - [ ] 变量查找
  - [ ] 类型定义查找
  - [ ] 函数签名管理
  
- [ ] 类型检查
  - [ ] 表达式类型推导
  - [ ] 类型兼容性检查
  - [ ] 隐式类型转换
  - [ ] 指针类型检查

- [ ] 语义分析
  - [ ] 变量使用前声明检查
  - [ ] 函数调用参数检查
  - [ ] return语句检查
  - [ ] break/continue合法性检查

### 🎯 阶段4：Rust代码生成器

这是核心功能，需要实现C到Rust的转换：

#### 4.1 类型映射
```
C类型          ->  Rust类型
int            ->  i32
unsigned int   ->  u32
char           ->  i8
unsigned char  ->  u8
long           ->  i64
float          ->  f32
double         ->  f64
void           ->  ()
T*             ->  *mut T 或 &mut T 或 *const T 或 &T
T[]            ->  Vec<T> 或 &[T]
struct S       ->  struct S
enum E         ->  enum E
```

#### 4.2 表达式转换
- [ ] 指针运算转换为安全操作
- [ ] 数组访问转换
- [ ] 类型转换（as关键字）
- [ ] NULL -> None 或 null_mut()
- [ ] 位运算保持不变

#### 4.3 语句转换
- [ ] 变量声明添加类型标注
- [ ] 循环转换（for特殊处理）
- [ ] switch转换为match
- [ ] goto处理（尽量消除或使用loop+break）

#### 4.4 函数转换
- [ ] 函数签名转换
- [ ] 参数可变性推断
- [ ] 返回值处理
- [ ] unsafe块包装

#### 4.5 内存管理
- [ ] malloc/free -> Box::new/drop
- [ ] 手动内存管理识别
- [ ] 生命周期标注（高级）

### 🛠️ 阶段5：完整转译工具

#### 5.1 文件处理
- [ ] 多文件项目支持
- [ ] 头文件解析
- [ ] #include依赖分析
- [ ] 模块化输出

#### 5.2 项目转译
- [ ] 读取Makefile或CMakeLists.txt
- [ ] 生成Cargo.toml
- [ ] 组织模块结构
- [ ] 依赖库映射（如fuse -> fuse-rs）

#### 5.3 命令行工具
```bash
c-to-rust-tool [OPTIONS] <INPUT>

OPTIONS:
  -o, --output <DIR>       输出目录
  -p, --project            整个项目模式
  -v, --verbose            详细输出
  --unsafe-blocks          生成unsafe块
  --no-format              不格式化输出
```

### 🐳 阶段6：Docker和评测环境

#### 6.1 完善Dockerfile
```dockerfile
FROM debian:latest

# 安装依赖
RUN apt update && apt install -y \
    build-essential cmake ninja-build \
    bear pkg-config libfuse-dev \
    clang llvm lld curl

# 安装Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 复制转译工具
COPY . /app
WORKDIR /app

# 构建转译工具
RUN cargo build --release
```

#### 6.2 配置config.toml

**translate_chibicc配置：**
```toml
[problem.translate_chibicc]
docker_image = "c-to-rust-tool:latest"
codegen_workdir = "/app"
codegen_command = "/app/target/release/c-to-rust-tool -p /app/translate_chibicc/src -o /tmp/chibicc_rust"
codegen_logfile = "/tmp/chibicc_codegen.log"
codegen_resultdir = "/tmp/chibicc_rust"
build_command = "cargo build --release"
exe = "/tmp/chibicc_rust/target/release/chibicc"
```

**translate_littlefs_fuse配置：**
```toml
[problem.translate_littlefs_fuse]
docker_image = "c-to-rust-tool:latest"
codegen_workdir = "/app"
codegen_command = "/app/target/release/c-to-rust-tool -p /app/translate_littlefs_fuse/src -o /tmp/lfs_rust"
codegen_logfile = "/tmp/lfs_codegen.log"
codegen_resultdir = "/tmp/lfs_rust"
build_command = "cargo build --release"
exe = "/tmp/lfs_rust/target/release/lfs"
```

#### 6.3 测试流程
1. 使用bear生成compilation database
2. 解析C项目结构
3. 转译所有C文件到Rust
4. 生成Cargo项目
5. 运行测试用例验证功能

## 技术难点

### 🔴 高优先级
1. **指针和引用的正确转换**
   - 区分可变和不可变引用
   - 原始指针 vs 安全引用
   - 生命周期推断

2. **内存管理**
   - malloc/free识别
   - 所有权转移
   - 借用检查器友好的代码

3. **预处理器**
   - 宏展开
   - 条件编译
   - 常量定义

### 🟡 中优先级
4. **复杂类型系统**
   - 函数指针转换为闭包或fn指针
   - 可变参数函数
   - 联合体（unsafe）

5. **系统调用和外部函数**
   - ioctl等系统调用
   - FFI边界处理
   - libc函数映射

### 🟢 低优先级
6. **代码优化**
   - 生成惯用的Rust代码
   - 消除不必要的unsafe
   - 使用Rust标准库替代C库

## 项目结构

```
CToRustTool/
├── src/
│   ├── main.rs           # 主程序（当前是demo）
│   ├── ast.rs            # AST定义 ✅
│   ├── lexer.rs          # 词法分析器 ✅
│   ├── parser.rs         # 语法分析器 🔄
│   ├── codegen.rs        # C代码生成器 ✅
│   ├── rust_codegen.rs   # Rust代码生成器 ❌
│   ├── type_system.rs    # 类型系统 ❌
│   ├── symbol_table.rs   # 符号表 ❌
│   └── project.rs        # 项目管理 ❌
├── docs/
│   └── c_to_c_ast_demo.md  # Demo文档
├── translate_chibicc/     # 测试项目1
├── translate_littlefs_fuse/  # 测试项目2
├── Cargo.toml
├── Dockerfile
├── config.toml
└── README.md
```

## 下一步计划

### 立即执行（本周）
1. ✅ 完善AST数据结构
2. ✅ 扩展词法分析器
3. 🔄 增强语法分析器支持结构体/枚举/typedef
4. 创建符号表和类型系统框架
5. 开始Rust代码生成器原型

### 短期目标（2周内）
1. 完成基本的C到Rust类型映射
2. 实现简单函数的转译
3. 支持结构体定义转换
4. 测试简单C程序的转译

### 中期目标（1月内）
1. 完整的littlefs_fuse项目转译
2. 通过部分测试用例
3. 优化生成的Rust代码质量

### 长期目标（比赛前）
1. 两个项目完整转译并通过测试
2. 代码质量优化
3. 文档和演示准备

## 评估标准

根据比赛要求，评分将基于：
1. **功能正确性**（最重要）：转译后的程序能否通过测试用例
2. **代码覆盖率**：能转译多少C代码
3. **代码质量**：生成的Rust代码的可读性和安全性
4. **性能**：转译后程序的运行性能

## 风险和挑战

1. **时间限制**：项目规模较大，需要合理安排优先级
2. **复杂性**：C语言特性众多，完整支持很困难
3. **测试覆盖**：需要确保转译正确性
4. **unsafe代码**：如何平衡安全性和兼容性

## 总结

当前已经完成了基础框架的搭建，建立了完整的AST系统和基本的解析能力。接下来的重点是：
1. 完善语法分析器以支持更复杂的C语法
2. 实现类型系统和符号表
3. 开发Rust代码生成器（核心功能）
4. 整合为完整的转译工具

这是一个有挑战但可实现的目标！💪
