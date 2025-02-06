use std::{fmt::Display, iter::Peekable, str::Chars};

type Result<T> = std::result::Result<T, ExpError>;

#[derive(Debug)]
enum ExpError {
    ParseError(String),
}

impl Display for ExpError {
    // 定义一个名为fmt的方法，该方法接收一个可变引用的self和一个可变引用的Formatter作为参数，返回一个fmt::Result
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 使用match表达式匹配self，根据self的值进行不同的处理
        match self {
            // 如果self是ExpError::ParseError，则将错误信息写入Formatter
            ExpError::ParseError(s) => write!(f, "ParseError: {}", s),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power, // 指数
    LParen,
    RParen,
}

const ASSOC_LEFT: i32 = 0; // 左结合

const ASSOC_RIGHT: i32 = 1; // 右结合

// 为 Token 实现标准库中的 Display trait，以便可以将其格式化为字符串
impl Display for Token {
    // 实现 fmt 方法，该方法接受一个可变的 Formatter 引用，并返回一个 fmt::Result
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 使用 write! 宏将格式化后的字符串写入 Formatter
        write!(
            f,
            "{}",
            // 使用 match 语句根据 Token 的不同变体返回相应的字符串表示
            match self {
                // 如果 Token 是 Number 变体，则将其值转换为字符串
                Token::Number(n) => n.to_string(),
                // 如果 Token 是 Plus 变体，则返回 "+" 字符串
                Token::Plus => "+".to_string(),
                // 如果 Token 是 Minus 变体，则返回 "-" 字符串
                Token::Minus => "-".to_string(),
                // 如果 Token 是 Multiply 变体，则返回 "*" 字符串
                Token::Multiply => "*".to_string(),
                // 如果 Token 是 Divide 变体，则返回 "/" 字符串
                Token::Divide => "/".to_string(),
                // 如果 Token 是 Power 变体，则返回 "^" 字符串
                Token::Power => "^".to_string(),
                // 如果 Token 是 LParen 变体，则返回 "(" 字符串
                Token::LParen => "(".to_string(),
                // 如果 Token 是 RParen 变体，则返回 ")" 字符串
                Token::RParen => ")".to_string(),
            }
        )
    }
}

impl Token {
    // 判断是不是运算符号
    // 定义一个名为 is_operator 的方法，该方法接收一个不可变引用的 self 参数，并返回一个布尔值
    fn is_operator(&self) -> bool {
        // 使用 matches! 宏来检查 self 是否匹配给定的模式
        // 这里检查 self 是否是 Token 枚举中的 Plus, Minus, Multiply, Divide 或 Power 变体之一
        // 如果匹配，则返回 true，否则返回 false
        matches!(
            self,
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Power
        )
    }

    // 获取运算符的优先级
    // 定义一个方法 `precedence`，它接收一个 `self` 引用，返回一个 `i32` 类型的值
    fn precedence(&self) -> i32 {
        // 使用 `match` 表达式来匹配 `self` 的不同值
        match self {
            // 如果 `self` 是 `Token::Plus` 或 `Token::Minus`，则返回 1
            Token::Plus | Token::Minus => 1,
            // 如果 `self` 是 `Token::Multiply` 或 `Token::Divide`，则返回 2
            Token::Multiply | Token::Divide => 2,
            // 如果 `self` 是 `Token::Power`，则返回 3
            Token::Power => 3,
            // 如果 `self` 是其他任何值，则返回 0
            _ => 0,
        }
    }

    // 获取运算符的结合性
    // 定义一个名为assoc的方法，它返回一个i32类型的结果
    fn assoc(&self) -> i32 {
        // 使用match语句来匹配self的值，根据不同的Token枚举值返回不同的结果
        match self {
            // 如果self是Token::Power，则返回ASSOC_RIGHT
            Token::Power => ASSOC_RIGHT,
            // 如果self不是Token::Power，则返回ASSOC_LEFT
            _ => ASSOC_LEFT,
    }

    }
    // 根据当前运算符进行计算
    // 定义一个名为compute的方法，它接收两个f64类型的参数left和right，并返回一个f64类型的结果
    fn compute(&self, left: i32, right: i32) -> Option<i32> {
        // 使用match语句来匹配self的值，根据不同的Token枚举值执行不同的操作
        match self {
            // 如果self是Token::Plus，则返回left和right的和
            Token::Plus => Some(left + right),
            // 如果self是Token::Minus，则返回left和right的差
            Token::Minus => Some(left - right),
            // 如果self是Token::Multiply，则返回left和right的乘积
            Token::Multiply => Some(left * right),
            // 如果self是Token::Divide，则返回left除以right的结果
            Token::Divide => Some(left / right),
            // 如果self是Token::Power，则返回left的right次幂
            Token::Power => Some(left.pow(right.try_into().unwrap())),
            // 如果self不是上述任何一种Token，则返回None
            _ => None,
        }
    }
}

struct Tokenizer<'a> {
    tokens: Peekable<Chars<'a>>, // tokens是一个可变引用，指向一个迭代器，该迭代器用于遍历输入字符串中的字符
}

impl<'a> Tokenizer<'a> {
    // 创建一个新的 Tokenizer 实例
    // 参数 expression 是一个字符串切片，表示要解析的表达式
    fn new(expression: &'a str) -> Self {
        Self {
            tokens: expression.chars().peekable(), // 创建一个新的 Tokenizer 实例，将输入字符串的字符迭代器包装在 Peekable 中
        }
    }

    // 清楚空白字符
    fn clear_whitespace(&mut self) {
        while let Some(c) = self.tokens.peek() {
            if c.is_whitespace() {
                self.tokens.next();
            } else {
                break;
            }
        }
    }

    // 扫描数字
    // 定义一个方法 scan_number，用于从 tokens 中扫描数字，并返回一个 Option<Token> 类型的结果
    fn scan_number(&mut self) -> Option<Token> {
        // 创建一个空的字符串 number，用于存储扫描到的数字字符
        let mut number = String::new();
        // 使用 while let 循环，不断检查 tokens 的下一个字符
        while let Some(c) = self.tokens.peek() {
            // 如果下一个字符是数字
            if c.is_numeric() {
                // 将该字符添加到 number 字符串中
                number.push(*c);
                // 移动 tokens 的指针，跳过已处理的字符
                self.tokens.next();
            } else {
                // 如果下一个字符不是数字，则跳出循环
                break;
            }
        }
        // 如果 number 字符串为空，说明没有扫描到数字，返回 None
        if number.is_empty() {
            None
        } else {
            // 否则，将 number 字符串解析为整数，并包装成 Token::Number 返回 Some
            Some(Token::Number(number.parse().unwrap()))
        }
    }

    // 扫描运算符
    // 定义一个名为 scan_operator 的方法，该方法接收一个可变引用的 self 参数，并返回一个 Option<Token> 类型的值
    fn scan_operator(&mut self) -> Option<Token> {
        // 使用 match 语句匹配 self.tokens 的下一个元素
        match self.tokens.next() {
            // 如果下一个元素是 '+'，则返回 Some(Token::Plus)
            Some('+') => Some(Token::Plus),
            // 如果下一个元素是 '-'，则返回 Some(Token::Minus)
            Some('-') => Some(Token::Minus),
            // 如果下一个元素是 '*'，则返回 Some(Token::Multiply)
            Some('*') => Some(Token::Multiply),
            // 如果下一个元素是 '/'，则返回 Some(Token::Divide)
            Some('/') => Some(Token::Divide),
            // 如果下一个元素是 '^'，则返回 Some(Token::Power)
            Some('^') => Some(Token::Power),
            // 如果下一个元素是 '('，则返回 Some(Token::LParen)
            Some('(') => Some(Token::LParen),
            // 如果下一个元素是 ')'，则返回 Some(Token::RParen)
            Some(')') => Some(Token::RParen),
            // 如果下一个元素不是上述任何一个，则返回 None
            _ => None,
        }
    }
}

// 实现Iterator trait
impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    // 定义一个方法 next，用于获取下一个解析项
    fn next(&mut self) -> Option<Self::Item> {
        // 调用 clear_whitespace 方法，清除当前标记中的空白字符
        self.clear_whitespace();
        // 使用 peek 方法查看当前标记的第一个字符
        if let Some(c) = self.tokens.peek() {
            // 如果字符是数字，则调用 scan_number 方法进行数字解析
            if c.is_numeric() {
                self.scan_number()
            } else {
                // 如果字符不是数字，则调用 scan_operator 方法进行操作符解析
                self.scan_operator()
            }
        } else {
            // 如果没有更多的标记，则返回 None，表示解析结束
            None
        }
    }
}

struct Expr<'a> {
    iter: Peekable<Tokenizer<'a>>,
}

impl<'a> Expr<'a> {
    // 创建一个新的表达式实例
    fn new(input: &'a str) -> Self {
        Expr {
            // 使用Tokenizer将输入字符串转换为Token迭代器，并使用peekable以便可以预览下一个Token
            iter: Tokenizer::new(input).peekable(),
        }
    }
    // 计算表达式的值
    fn eval(&mut self) -> Result<i32> {
        // 从最低优先级开始计算表达式
        let result = self.compute_expr(1)?;
        // 检查是否还有剩余的 Token
        if self.iter.peek().is_some() {
            // 如果还有剩余的 Token，说明表达式有误
            return Err(ExpError::ParseError("Unexpected token".to_string()));
        } else {
            // 如果没有剩余的 Token，返回计算结果
            Ok(result)
        }
    }

    // 计算表达式的值，参数min_prec表示当前处理的运算符的最小优先级
    fn compute_expr(&mut self, min_prec: i32) -> Result<i32> {
        // 计算第一个 Token
        let mut atom_lhs = self.compute_atom()?;

        loop {
            // 预览下一个 Token
            let cur_token = self.iter.peek();
            if cur_token.is_none() {
                // 如果没有下一个 Token，退出循环
                break;
            }
            let token = *cur_token.unwrap();

            // 1. Token 一定是运算符
            // 2. Token 的优先级必须大于等于 min_prec
            if !token.is_operator() || token.precedence() < min_prec {
                // 如果当前 Token 不是运算符或优先级不够，退出循环
                break;
            }

            let mut next_prec = token.precedence();
            if token.assoc() == ASSOC_LEFT {
                // 如果是左结合运算符，下一级优先级加1
                next_prec += 1;
            }

            // 移动到下一个 Token
            self.iter.next();

            // 递归计算右边的表达式
            let atom_rhs = self.compute_expr(next_prec)?;

            // 得到了两边的值，进行计算
            match token.compute(atom_lhs, atom_rhs) {
                Some(res) => atom_lhs = res, // 计算成功，更新左边的值
                None => return Err(ExpError::ParseError("Unexpected expr".into())), // 计算失败，返回错误
            }
        }
        Ok(atom_lhs) // 返回计算结果
    }

    // 计算原子表达式（数字或括号内的表达式）
    fn compute_atom(&mut self) -> Result<i32> {
        if let Some(token) = self.iter.next() {
            match token {
                Token::Number(n) => Ok(n as i32), // 如果是数字，直接返回其值
                Token::LParen => {
                    // 如果是左括号，计算括号内的表达式
                    let result = self.compute_expr(1)?;
                    if let Some(Token::RParen) = self.iter.next() {
                        // 检查是否有匹配的右括号
                        Ok(result)
                    } else {
                        // 如果没有匹配的右括号，返回错误
                        Err(ExpError::ParseError("Expected closing parenthesis".to_string()))
                    }
                }
                _ => Err(ExpError::ParseError("Unexpected token".to_string())), // 其他 Token 返回错误
            }
        } else {
            // 如果没有 Token，返回错误
            Err(ExpError::ParseError("Unexpected end of input".to_string()))
        }
    }
}


fn main() {
    let src = "92 + 5 + 5 * 27 - (92 - 12) / 4 + 26";
    let mut expr = Expr::new(src);
    let result = expr.eval();
    println!("res = {:?}", result);
}

// 编写测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_atom() {
        let mut expr = Expr::new("5");
        let result = expr.compute_atom().unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_compute_expr() {
        let mut expr = Expr::new("5 + 5");
        let result = expr.compute_expr(0).unwrap();
        assert_eq!(result, 10);
    }
}
