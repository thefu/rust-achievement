#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Operator(char),
    LeftParen,
    RightParen,
}

// 定义一个函数 `tokenize`，它接受一个字符串切片 `expr` 作为参数，并返回一个 `Token` 类型的向量
fn tokenize(expr: &str) -> Vec<Token> {
    // 初始化一个空的 `Token` 向量，用于存储解析后的标记
    let mut tokens = Vec::new();
    // 初始化一个空的字符串，用于暂存数字字符
    let mut num_str = String::new();
    
    // 遍历表达式中的每一个字符
    for c in expr.chars() {
        // 使用 `match` 语句对字符进行模式匹配
        match c {
            // 如果字符是数字或小数点，则将其添加到 `num_str` 中
            '0'..='9' | '.' => num_str.push(c),
            // 如果字符是运算符（+、-、*、/、^），则处理当前暂存的数字
            '+' | '-' | '*' | '/' | '^' => {
                // 如果 `num_str` 不为空，则将其解析为数字并添加到 `tokens` 中
                if !num_str.is_empty() {
                    tokens.push(Token::Number(num_str.parse().unwrap()));
                    // 清空 `num_str` 以便存储下一个数字
                    num_str.clear();
                }
                // 将当前运算符作为 `Token::Operator` 添加到 `tokens` 中
                tokens.push(Token::Operator(c));
            }
            // 如果字符是左括号，则将其作为 `Token::LeftParen` 添加到 `tokens` 中
            '(' => tokens.push(Token::LeftParen),
            // 如果字符是右括号，则处理当前暂存的数字，并将其作为 `Token::RightParen` 添加到 `tokens` 中
            ')' => {
                // 如果 `num_str` 不为空，则将其解析为数字并添加到 `tokens` 中
                if !num_str.is_empty() {
                    tokens.push(Token::Number(num_str.parse().unwrap()));
                    // 清空 `num_str` 以便存储下一个数字
                    num_str.clear();
                }
                // 将右括号作为 `Token::RightParen` 添加到 `tokens` 中
                tokens.push(Token::RightParen);
            }
            ' ' => {
                // 如果 `num_str` 不为空，则将其解析为数字并添加到 `tokens` 中
                if !num_str.is_empty() {
                    tokens.push(Token::Number(num_str.parse().unwrap()));
                    num_str.clear();
                }
            }
            // 如果遇到无效字符，则抛出异常
            _ => panic!("Invalid character in expression"),
        }
    }
    // 如果遍历结束后 `num_str` 不为空，则将其解析为数字并添加到 `tokens` 中
    if !num_str.is_empty() {
        tokens.push(Token::Number(num_str.parse().unwrap()));
    }
    // 返回解析后的 `Token` 向量
    tokens
}

// 定义一个函数 `precedence`，它接受一个字符 `op` 作为参数，并返回一个无符号8位整数（u8）
fn precedence(op: char) -> u8 {
    // 使用 `match` 表达式来匹配输入的运算符 `op`
    match op {
        // 如果 `op` 是 '+' 或 '-'，则返回优先级 1
        '+' | '-' => 1,
        // 如果 `op` 是 '*' 或 '/'，则返回优先级 2
        '*' | '/' => 2,
        // 如果 `op` 是 '^'，则返回优先级 3
        '^' => 3,
        // 如果 `op` 不匹配上述任何一种情况，则返回优先级 0
        _ => 0,
    }
}

// 定义一个函数，将中缀表达式转换为后缀表达式
fn to_postfix(tokens: Vec<Token>) -> Vec<Token> {
    // 初始化输出向量，用于存储转换后的后缀表达式
    let mut output = Vec::new();
    // 初始化操作符栈，用于存储操作符
    let mut operator_stack = Vec::new();

    // 遍历输入的中缀表达式中的每个标记
    for token in tokens {
        // 使用模式匹配处理不同类型的标记
        match token {
            // 如果是数字，直接添加到输出向量
            Token::Number(_) => output.push(token),
            // 如果是操作符，进行以下处理
            Token::Operator(op) => {
                // 当操作符栈不为空且栈顶操作符的优先级大于等于当前操作符时
                while let Some(Token::Operator(top_op)) = operator_stack.last() {
                    if precedence(*top_op) >= precedence(op) {
                        // 将栈顶操作符弹出并添加到输出向量
                        output.push(operator_stack.pop().unwrap());
                    } else {
                        // 否则跳出循环
                        break;
                    }
                }
                // 将当前操作符压入操作符栈
                operator_stack.push(Token::Operator(op));
            }
            // 如果是左括号，直接压入操作符栈
            Token::LeftParen => operator_stack.push(token),
            // 如果是右括号，进行以下处理
            Token::RightParen => {
                // 当操作符栈不为空时
                while let Some(top_token) = operator_stack.pop() {
                    match top_token {
                        // 如果遇到左括号，跳出循环
                        Token::LeftParen => break,
                        // 否则将栈顶操作符添加到输出向量
                        _ => output.push(top_token),
                    }
                }
            }
        }
    }

    // 将操作符栈中剩余的操作符依次弹出并添加到输出向量
    while let Some(token) = operator_stack.pop() {
        output.push(token);
    }
    // 返回转换后的后缀表达式
    output
}

// 定义一个函数 evaluate_postfix，用于计算后缀表达式的值
fn evaluate_postfix(tokens: Vec<Token>) -> f64 {
    // 创建一个空的栈，用于存储操作数
    let mut stack = Vec::new();

    // 遍历输入的后缀表达式中的每个标记
    for token in tokens {
        // 使用 match 语句对标记进行模式匹配
        match token {
            // 如果标记是一个数字，则将其压入栈中
            Token::Number(num) => stack.push(num),
            // 如果标记是一个运算符，则从栈中弹出两个操作数进行计算，并将结果压入栈中
            Token::Operator(op) => {
                // 从栈中弹出第二个操作数
                let b = stack.pop().unwrap();
                // 从栈中弹出第一个操作数
                let a = stack.pop().unwrap();
                // 根据运算符进行相应的计算
                let result = match op {
                    '+' => a + b, // 加法
                    '-' => a - b, // 减法
                    '*' => a * b, // 乘法
                    '/' => a / b, // 除法
                    '^' => a.powf(b), // 幂运算
                    _ => panic!("Unknown operator"), // 未知运算符，抛出异常
                };
                // 将计算结果压入栈中
                stack.push(result);
            }
            // 如果标记既不是数字也不是运算符，则抛出异常
            _ => panic!("Invalid token in postfix expression"),
        }
    }
    // 返回栈顶元素，即最终的计算结果
    stack.pop().unwrap()
}

// 定义一个公共函数 expression_parsing_algorithm，用于解析表达式并计算其结果
// 参数 expr 是一个字符串切片，表示要解析的表达式
// 返回值是一个 f64 类型的浮点数，表示表达式的计算结果
pub fn expression_parsing_algorithm(expr: &str) -> f64 {
    // 调用 tokenize 函数，将表达式字符串分割成一个个的标记（token）
    // 例如，将 "3 + 4 * 2" 分割成 ["3", "+", "4", "*", "2"]
    let tokens = tokenize(expr);
    // 调用 to_postfix 函数，将标记列表从中缀表达式转换为后缀表达式（逆波兰表示法）
    // 例如，将 ["3", "+", "4", "*", "2"] 转换为 ["3", "4", "2", "*", "+"]
    let postfix = to_postfix(tokens);
    // 调用 evaluate_postfix 函数，计算后缀表达式的值
    // 例如，计算 ["3", "4", "2", "*", "+"] 的结果为 11.0
    evaluate_postfix(postfix)
}

fn main() {
    let expr = "92 + 5 + 5 * 27 - (92 - 12) / 4 + 26";
    let result = expression_parsing_algorithm(expr);
    println!("Result: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        assert_eq!(expression_parsing_algorithm("3+2"), 5.0);
        assert_eq!(expression_parsing_algorithm("3*2"), 6.0);
        assert_eq!(expression_parsing_algorithm("6/2"), 3.0);
        assert_eq!(expression_parsing_algorithm("2^3"), 8.0);
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(expression_parsing_algorithm("3 + 4 * 2 / ( 1 - 5 ) ^ 2"), 3.5);
    }
}