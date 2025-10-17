/// 测试表达式解析功能
use c_to_rust_tool::parser::Parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cast_expression() {
        let input = r#"
        int main() {
            int* p = (int*)malloc(sizeof(int));
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse cast expression: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_array_access() {
        let input = r#"
        int main() {
            int arr[10];
            int x = arr[5];
            arr[0] = 42;
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse array access: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_struct_member_access() {
        let input = r#"
        struct Point {
            int x;
            int y;
        };
        
        int main() {
            struct Point p;
            p.x = 10;
            p.y = 20;
            int sum = p.x + p.y;
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse struct member access: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_pointer_member_access() {
        let input = r#"
        struct Point {
            int x;
            int y;
        };
        
        int main() {
            struct Point* p;
            p->x = 10;
            p->y = 20;
            int sum = p->x + p->y;
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse pointer member access: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_increment_decrement() {
        let input = r#"
        int main() {
            int i = 0;
            i++;
            ++i;
            i--;
            --i;
            int j = i++ + ++i;
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse increment/decrement: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_bitwise_operators() {
        let input = r#"
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

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse bitwise operators: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_ternary_operator() {
        let input = r#"
        int main() {
            int a = 5;
            int b = 10;
            int max = (a > b) ? a : b;
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse ternary operator: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_sizeof_operator() {
        let input = r#"
        int main() {
            int size1 = sizeof(int);
            int size2 = sizeof(char);
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse sizeof operator: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_pointer_dereference() {
        let input = r#"
        int main() {
            int x = 42;
            int* p = &x;
            int y = *p;
            *p = 100;
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse pointer dereference: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_complex_expression() {
        let input = r#"
        struct Node {
            int value;
            struct Node* next;
        };
        
        int main() {
            struct Node* head = (struct Node*)malloc(sizeof(struct Node));
            head->value = 42;
            head->next = NULL;
            
            int arr[10];
            arr[0] = head->value;
            
            int result = (arr[0] > 0) ? arr[0] * 2 : 0;
            
            return 0;
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Failed to parse complex expression: {:?}",
            result.err()
        );
    }
}
