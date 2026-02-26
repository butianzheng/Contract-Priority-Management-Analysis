use chrono::{Datelike, Duration, Local, NaiveDate};

use super::generic_parser::RawRow;

#[derive(Debug, Clone)]
pub struct ExpressionEngine;

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Value),
    FieldRef(String),
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    BinaryOp {
        op: Op,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Date(NaiveDate),
    Null,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    And,
    Or,
    Concat,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    String(String),
    Identifier(String),
    LParen,
    RParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    Amp,
}

impl ExpressionEngine {
    pub fn parse(expression: &str) -> Result<Expression, String> {
        let tokens = tokenize(expression)?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression()?;
        if parser.has_remaining() {
            return Err("表达式末尾存在无法解析的内容".to_string());
        }
        Ok(expr)
    }

    pub fn evaluate(expr: &Expression, row: &RawRow) -> Result<String, String> {
        let value = eval_expr(expr, row)?;
        Ok(value.to_string_value())
    }

    pub fn evaluate_str(expression: &str, row: &RawRow) -> Result<String, String> {
        let expr = Self::parse(expression)?;
        Self::evaluate(&expr, row)
    }

    pub fn validate(expression: &str) -> Result<(), String> {
        Self::parse(expression).map(|_| ())
    }
}

impl Value {
    fn to_string_value(&self) -> String {
        match self {
            Value::Number(n) => {
                if (n.fract()).abs() < f64::EPSILON {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Bool(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Value::Date(d) => d.format("%Y-%m-%d").to_string(),
            Value::Null => String::new(),
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::String(s) => s.trim().parse::<f64>().ok(),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::Date(d) => Some(d.num_days_from_ce() as f64),
            Value::Null => None,
        }
    }

    fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => {
                let lowered = s.trim().to_lowercase();
                !lowered.is_empty() && lowered != "false" && lowered != "0"
            }
            Value::Date(_) => true,
            Value::Null => false,
        }
    }

    fn as_date(&self) -> Option<NaiveDate> {
        match self {
            Value::Date(d) => Some(*d),
            Value::String(s) => parse_date(s),
            _ => None,
        }
    }

    fn from_cell_value(value: Option<&String>) -> Value {
        match value {
            Some(v) => {
                let trimmed = v.trim();
                if trimmed.is_empty() {
                    Value::Null
                } else if let Ok(n) = trimmed.parse::<f64>() {
                    Value::Number(n)
                } else if let Some(d) = parse_date(trimmed) {
                    Value::Date(d)
                } else {
                    Value::String(trimmed.to_string())
                }
            }
            None => Value::Null,
        }
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(ch) = chars.peek().copied() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        match ch {
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '%' => {
                tokens.push(Token::Percent);
                chars.next();
            }
            '&' => {
                tokens.push(Token::Amp);
                chars.next();
            }
            '=' => {
                tokens.push(Token::Eq);
                chars.next();
            }
            '!' => {
                chars.next();
                if chars.next_if_eq(&'=').is_some() {
                    tokens.push(Token::Ne);
                } else {
                    return Err("'!' 后必须跟 '='".to_string());
                }
            }
            '>' => {
                chars.next();
                if chars.next_if_eq(&'=').is_some() {
                    tokens.push(Token::Ge);
                } else {
                    tokens.push(Token::Gt);
                }
            }
            '<' => {
                chars.next();
                if chars.next_if_eq(&'=').is_some() {
                    tokens.push(Token::Le);
                } else {
                    tokens.push(Token::Lt);
                }
            }
            '"' | '\'' => {
                tokens.push(Token::String(parse_quoted_string(&mut chars, ch)?));
            }
            c if c.is_ascii_digit() || c == '.' => {
                tokens.push(Token::Number(parse_number(&mut chars)?));
            }
            _ => {
                tokens.push(Token::Identifier(parse_identifier(&mut chars)));
            }
        }
    }

    Ok(tokens)
}

fn parse_quoted_string(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    quote: char,
) -> Result<String, String> {
    let mut out = String::new();
    chars.next();

    while let Some(ch) = chars.next() {
        if ch == quote {
            return Ok(out);
        }
        if ch == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(ch);
        }
    }

    Err("字符串缺少闭合引号".to_string())
}

fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<f64, String> {
    let mut num_str = String::new();
    let mut dot_seen = false;

    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
            chars.next();
        } else if ch == '.' && !dot_seen {
            dot_seen = true;
            num_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    num_str
        .parse::<f64>()
        .map_err(|_| format!("无效数字: {}", num_str))
}

fn parse_identifier(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut ident = String::new();

    while let Some(ch) = chars.peek().copied() {
        if ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '$' || ch >= '\u{80}' {
            ident.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    ident
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn has_remaining(&self) -> bool {
        self.pos < self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_and()?;

        while self.match_identifier("OR") {
            let right = self.parse_and()?;
            expr = Expression::BinaryOp {
                op: Op::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_not()?;

        while self.match_identifier("AND") {
            let right = self.parse_not()?;
            expr = Expression::BinaryOp {
                op: Op::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_not(&mut self) -> Result<Expression, String> {
        if self.match_identifier("NOT") {
            let expr = self.parse_not()?;
            return Ok(Expression::UnaryOp {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        self.parse_compare()
    }

    fn parse_compare(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_concat()?;

        loop {
            let op = match self.peek() {
                Some(Token::Eq) => Some(Op::Eq),
                Some(Token::Ne) => Some(Op::Ne),
                Some(Token::Gt) => Some(Op::Gt),
                Some(Token::Lt) => Some(Op::Lt),
                Some(Token::Ge) => Some(Op::Ge),
                Some(Token::Le) => Some(Op::Le),
                _ => None,
            };

            if let Some(found) = op {
                self.next();
                let right = self.parse_concat()?;
                expr = Expression::BinaryOp {
                    op: found,
                    left: Box::new(expr),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_concat(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_add_sub()?;

        while matches!(self.peek(), Some(Token::Amp)) {
            self.next();
            let right = self.parse_add_sub()?;
            expr = Expression::BinaryOp {
                op: Op::Concat,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn parse_add_sub(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_mul_div()?;

        loop {
            let op = match self.peek() {
                Some(Token::Plus) => Some(Op::Add),
                Some(Token::Minus) => Some(Op::Sub),
                _ => None,
            };

            if let Some(found) = op {
                self.next();
                let right = self.parse_mul_div()?;
                expr = Expression::BinaryOp {
                    op: found,
                    left: Box::new(expr),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_mul_div(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                Some(Token::Star) => Some(Op::Mul),
                Some(Token::Slash) => Some(Op::Div),
                Some(Token::Percent) => Some(Op::Mod),
                _ => None,
            };

            if let Some(found) = op {
                self.next();
                let right = self.parse_unary()?;
                expr = Expression::BinaryOp {
                    op: found,
                    left: Box::new(expr),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        match self.peek() {
            Some(Token::Plus) => {
                self.next();
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Plus,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            Some(Token::Minus) => {
                self.next();
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Minus,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.next() {
            Some(Token::Number(n)) => Ok(Expression::Literal(Value::Number(n))),
            Some(Token::String(s)) => Ok(Expression::Literal(Value::String(s))),
            Some(Token::Identifier(name)) => {
                let upper = name.to_uppercase();

                if upper == "TRUE" {
                    return Ok(Expression::Literal(Value::Bool(true)));
                }
                if upper == "FALSE" {
                    return Ok(Expression::Literal(Value::Bool(false)));
                }

                if matches!(self.peek(), Some(Token::LParen)) {
                    self.next();
                    let mut args = Vec::new();

                    if !matches!(self.peek(), Some(Token::RParen)) {
                        loop {
                            args.push(self.parse_expression()?);
                            if matches!(self.peek(), Some(Token::Comma)) {
                                self.next();
                            } else {
                                break;
                            }
                        }
                    }

                    match self.next() {
                        Some(Token::RParen) => Ok(Expression::FunctionCall { name, args }),
                        _ => Err("函数调用缺少 ')'".to_string()),
                    }
                } else {
                    Ok(Expression::FieldRef(name))
                }
            }
            Some(Token::LParen) => {
                let expr = self.parse_expression()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err("缺少 ')'".to_string()),
                }
            }
            _ => Err("无法解析表达式".to_string()),
        }
    }

    fn match_identifier(&mut self, keyword: &str) -> bool {
        match self.peek() {
            Some(Token::Identifier(id)) if id.eq_ignore_ascii_case(keyword) => {
                self.next();
                true
            }
            _ => false,
        }
    }
}

fn eval_expr(expr: &Expression, row: &RawRow) -> Result<Value, String> {
    match expr {
        Expression::Literal(v) => Ok(v.clone()),
        Expression::FieldRef(name) => Ok(Value::from_cell_value(row.get(name))),
        Expression::UnaryOp { op, expr } => {
            let value = eval_expr(expr, row)?;
            match op {
                UnaryOp::Plus => Ok(Value::Number(value.as_f64().unwrap_or(0.0))),
                UnaryOp::Minus => Ok(Value::Number(-value.as_f64().unwrap_or(0.0))),
                UnaryOp::Not => Ok(Value::Bool(!value.as_bool())),
            }
        }
        Expression::BinaryOp { op, left, right } => {
            let l = eval_expr(left, row)?;
            let r = eval_expr(right, row)?;
            eval_binary(*op, l, r)
        }
        Expression::FunctionCall { name, args } => eval_function(name, args, row),
    }
}

fn eval_binary(op: Op, left: Value, right: Value) -> Result<Value, String> {
    match op {
        Op::Add => {
            if let (Some(l), Some(r)) = (left.as_f64(), right.as_f64()) {
                Ok(Value::Number(l + r))
            } else {
                Ok(Value::String(format!(
                    "{}{}",
                    left.to_string_value(),
                    right.to_string_value()
                )))
            }
        }
        Op::Sub => {
            let (l, r) = to_two_numbers(&left, &right)?;
            Ok(Value::Number(l - r))
        }
        Op::Mul => {
            let (l, r) = to_two_numbers(&left, &right)?;
            Ok(Value::Number(l * r))
        }
        Op::Div => {
            let (l, r) = to_two_numbers(&left, &right)?;
            if r.abs() < f64::EPSILON {
                return Err("除数不能为 0".to_string());
            }
            Ok(Value::Number(l / r))
        }
        Op::Mod => {
            let (l, r) = to_two_numbers(&left, &right)?;
            if r.abs() < f64::EPSILON {
                return Err("取模除数不能为 0".to_string());
            }
            Ok(Value::Number(l % r))
        }
        Op::Eq => Ok(Value::Bool(
            compare_values(&left, &right) == std::cmp::Ordering::Equal,
        )),
        Op::Ne => Ok(Value::Bool(
            compare_values(&left, &right) != std::cmp::Ordering::Equal,
        )),
        Op::Gt => Ok(Value::Bool(
            compare_values(&left, &right) == std::cmp::Ordering::Greater,
        )),
        Op::Lt => Ok(Value::Bool(
            compare_values(&left, &right) == std::cmp::Ordering::Less,
        )),
        Op::Ge => {
            let order = compare_values(&left, &right);
            Ok(Value::Bool(
                order == std::cmp::Ordering::Greater || order == std::cmp::Ordering::Equal,
            ))
        }
        Op::Le => {
            let order = compare_values(&left, &right);
            Ok(Value::Bool(
                order == std::cmp::Ordering::Less || order == std::cmp::Ordering::Equal,
            ))
        }
        Op::And => Ok(Value::Bool(left.as_bool() && right.as_bool())),
        Op::Or => Ok(Value::Bool(left.as_bool() || right.as_bool())),
        Op::Concat => Ok(Value::String(format!(
            "{}{}",
            left.to_string_value(),
            right.to_string_value()
        ))),
    }
}

fn compare_values(left: &Value, right: &Value) -> std::cmp::Ordering {
    if let (Some(l), Some(r)) = (left.as_f64(), right.as_f64()) {
        if l > r {
            std::cmp::Ordering::Greater
        } else if (l - r).abs() < f64::EPSILON {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Less
        }
    } else {
        left.to_string_value().cmp(&right.to_string_value())
    }
}

fn to_two_numbers(left: &Value, right: &Value) -> Result<(f64, f64), String> {
    let l = left
        .as_f64()
        .ok_or_else(|| format!("左侧值不是数字: {}", left.to_string_value()))?;
    let r = right
        .as_f64()
        .ok_or_else(|| format!("右侧值不是数字: {}", right.to_string_value()))?;
    Ok((l, r))
}

fn eval_function(name: &str, args: &[Expression], row: &RawRow) -> Result<Value, String> {
    let upper = name.to_uppercase();

    match upper.as_str() {
        "IF" => {
            if args.len() != 3 {
                return Err("IF 函数参数必须为 3 个".to_string());
            }
            let cond = eval_expr(&args[0], row)?;
            if cond.as_bool() {
                eval_expr(&args[1], row)
            } else {
                eval_expr(&args[2], row)
            }
        }
        "SWITCH" => {
            if args.len() < 3 {
                return Err("SWITCH 至少需要 3 个参数".to_string());
            }
            let target = eval_expr(&args[0], row)?;
            let mut idx = 1;
            while idx + 1 < args.len() {
                let case_val = eval_expr(&args[idx], row)?;
                if compare_values(&target, &case_val) == std::cmp::Ordering::Equal {
                    return eval_expr(&args[idx + 1], row);
                }
                idx += 2;
            }
            if idx < args.len() {
                eval_expr(&args[idx], row)
            } else {
                Ok(Value::Null)
            }
        }
        "ROUND" => {
            if args.is_empty() || args.len() > 2 {
                return Err("ROUND 参数个数应为 1 或 2".to_string());
            }
            let value = eval_expr(&args[0], row)?
                .as_f64()
                .ok_or_else(|| "ROUND 第1个参数必须为数字".to_string())?;
            let digits = if args.len() == 2 {
                eval_expr(&args[1], row)?
                    .as_f64()
                    .ok_or_else(|| "ROUND 第2个参数必须为数字".to_string())? as i32
            } else {
                0
            };
            let factor = 10f64.powi(digits);
            Ok(Value::Number((value * factor).round() / factor))
        }
        "CEIL" | "CEILING" => {
            let value = eval_one_number_arg(args, row, &upper)?;
            Ok(Value::Number(value.ceil()))
        }
        "FLOOR" => {
            let value = eval_one_number_arg(args, row, &upper)?;
            Ok(Value::Number(value.floor()))
        }
        "ABS" => {
            let value = eval_one_number_arg(args, row, &upper)?;
            Ok(Value::Number(value.abs()))
        }
        "MIN" => {
            let numbers = eval_number_args(args, row, &upper)?;
            let min = numbers
                .into_iter()
                .reduce(f64::min)
                .ok_or_else(|| "MIN 至少需要 1 个参数".to_string())?;
            Ok(Value::Number(min))
        }
        "MAX" => {
            let numbers = eval_number_args(args, row, &upper)?;
            let max = numbers
                .into_iter()
                .reduce(f64::max)
                .ok_or_else(|| "MAX 至少需要 1 个参数".to_string())?;
            Ok(Value::Number(max))
        }
        "LEFT" => {
            if args.len() != 2 {
                return Err("LEFT 参数必须为 2 个".to_string());
            }
            let text = eval_expr(&args[0], row)?.to_string_value();
            let n = eval_expr(&args[1], row)?
                .as_f64()
                .ok_or_else(|| "LEFT 第2个参数必须为数字".to_string())?
                .max(0.0) as usize;
            Ok(Value::String(text.chars().take(n).collect()))
        }
        "RIGHT" => {
            if args.len() != 2 {
                return Err("RIGHT 参数必须为 2 个".to_string());
            }
            let text = eval_expr(&args[0], row)?.to_string_value();
            let n = eval_expr(&args[1], row)?
                .as_f64()
                .ok_or_else(|| "RIGHT 第2个参数必须为数字".to_string())?
                .max(0.0) as usize;
            let chars: Vec<char> = text.chars().collect();
            let start = chars.len().saturating_sub(n);
            Ok(Value::String(chars[start..].iter().collect()))
        }
        "MID" => {
            if args.len() != 3 {
                return Err("MID 参数必须为 3 个".to_string());
            }
            let text = eval_expr(&args[0], row)?.to_string_value();
            let start = eval_expr(&args[1], row)?
                .as_f64()
                .ok_or_else(|| "MID 第2个参数必须为数字".to_string())?
                .max(1.0) as usize;
            let len = eval_expr(&args[2], row)?
                .as_f64()
                .ok_or_else(|| "MID 第3个参数必须为数字".to_string())?
                .max(0.0) as usize;

            let chars: Vec<char> = text.chars().collect();
            let begin = start.saturating_sub(1).min(chars.len());
            let end = (begin + len).min(chars.len());
            Ok(Value::String(chars[begin..end].iter().collect()))
        }
        "LEN" => {
            if args.len() != 1 {
                return Err("LEN 参数必须为 1 个".to_string());
            }
            let text = eval_expr(&args[0], row)?.to_string_value();
            Ok(Value::Number(text.chars().count() as f64))
        }
        "FIND" => {
            if args.len() != 2 {
                return Err("FIND 参数必须为 2 个".to_string());
            }
            let needle = eval_expr(&args[0], row)?.to_string_value();
            let haystack = eval_expr(&args[1], row)?.to_string_value();
            match haystack.find(&needle) {
                Some(index) => Ok(Value::Number((index + 1) as f64)),
                None => Ok(Value::Number(0.0)),
            }
        }
        "REPLACE" => {
            if args.len() != 3 {
                return Err("REPLACE 参数必须为 3 个".to_string());
            }
            let text = eval_expr(&args[0], row)?.to_string_value();
            let from = eval_expr(&args[1], row)?.to_string_value();
            let to = eval_expr(&args[2], row)?.to_string_value();
            Ok(Value::String(text.replace(&from, &to)))
        }
        "UPPER" => {
            if args.len() != 1 {
                return Err("UPPER 参数必须为 1 个".to_string());
            }
            Ok(Value::String(
                eval_expr(&args[0], row)?.to_string_value().to_uppercase(),
            ))
        }
        "LOWER" => {
            if args.len() != 1 {
                return Err("LOWER 参数必须为 1 个".to_string());
            }
            Ok(Value::String(
                eval_expr(&args[0], row)?.to_string_value().to_lowercase(),
            ))
        }
        "TRIM" => {
            if args.len() != 1 {
                return Err("TRIM 参数必须为 1 个".to_string());
            }
            Ok(Value::String(
                eval_expr(&args[0], row)?
                    .to_string_value()
                    .trim()
                    .to_string(),
            ))
        }
        "CONCAT" => {
            let mut out = String::new();
            for arg in args {
                out.push_str(&eval_expr(arg, row)?.to_string_value());
            }
            Ok(Value::String(out))
        }
        "TODAY" => {
            if !args.is_empty() {
                return Err("TODAY 不接受参数".to_string());
            }
            Ok(Value::Date(Local::now().date_naive()))
        }
        "DATEDIFF" => {
            if args.len() != 3 {
                return Err("DATEDIFF 参数必须为 3 个".to_string());
            }
            let start = eval_expr(&args[0], row)?
                .as_date()
                .ok_or_else(|| "DATEDIFF 第1个参数必须是日期".to_string())?;
            let end = eval_expr(&args[1], row)?
                .as_date()
                .ok_or_else(|| "DATEDIFF 第2个参数必须是日期".to_string())?;
            let unit = eval_expr(&args[2], row)?.to_string_value().to_lowercase();

            match unit.as_str() {
                "day" | "days" => Ok(Value::Number((end - start).num_days() as f64)),
                _ => Err(format!("DATEDIFF 不支持的单位: {}", unit)),
            }
        }
        "DATEADD" => {
            if args.len() != 3 {
                return Err("DATEADD 参数必须为 3 个".to_string());
            }
            let base = eval_expr(&args[0], row)?
                .as_date()
                .ok_or_else(|| "DATEADD 第1个参数必须是日期".to_string())?;
            let amount = eval_expr(&args[1], row)?
                .as_f64()
                .ok_or_else(|| "DATEADD 第2个参数必须为数字".to_string())?
                as i64;
            let unit = eval_expr(&args[2], row)?.to_string_value().to_lowercase();

            let date = match unit.as_str() {
                "day" | "days" => base + Duration::days(amount),
                _ => return Err(format!("DATEADD 不支持的单位: {}", unit)),
            };
            Ok(Value::Date(date))
        }
        "DATEFORMAT" => {
            if args.len() != 2 {
                return Err("DATEFORMAT 参数必须为 2 个".to_string());
            }
            let date = eval_expr(&args[0], row)?
                .as_date()
                .ok_or_else(|| "DATEFORMAT 第1个参数必须是日期".to_string())?;
            let format = eval_expr(&args[1], row)?.to_string_value();
            Ok(Value::String(format_date_by_pattern(date, &format)))
        }
        _ => Err(format!("不支持的函数: {}", name)),
    }
}

fn eval_one_number_arg(args: &[Expression], row: &RawRow, fn_name: &str) -> Result<f64, String> {
    if args.len() != 1 {
        return Err(format!("{} 参数必须为 1 个", fn_name));
    }
    eval_expr(&args[0], row)?
        .as_f64()
        .ok_or_else(|| format!("{} 参数必须为数字", fn_name))
}

fn eval_number_args(args: &[Expression], row: &RawRow, fn_name: &str) -> Result<Vec<f64>, String> {
    if args.is_empty() {
        return Err(format!("{} 至少需要 1 个参数", fn_name));
    }
    args.iter()
        .map(|arg| {
            eval_expr(arg, row)?
                .as_f64()
                .ok_or_else(|| format!("{} 参数必须为数字", fn_name))
        })
        .collect()
}

fn parse_date(input: &str) -> Option<NaiveDate> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    for fmt in ["%Y-%m-%d", "%Y/%m/%d", "%d-%m-%Y", "%d/%m/%Y", "%Y%m%d"] {
        if let Ok(d) = NaiveDate::parse_from_str(trimmed, fmt) {
            return Some(d);
        }
    }

    None
}

fn format_date_by_pattern(date: NaiveDate, pattern: &str) -> String {
    pattern
        .replace("YYYY", &format!("{:04}", date.year()))
        .replace("MM", &format!("{:02}", date.month()))
        .replace("DD", &format!("{:02}", date.day()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_row() -> RawRow {
        let mut row = RawRow::new();
        row.insert("customer_level".to_string(), "VIP".to_string());
        row.insert("steel_grade".to_string(), "Q235A".to_string());
        row.insert("thickness".to_string(), "1.56".to_string());
        row.insert("pdd".to_string(), "2026-03-15".to_string());
        row
    }

    #[test]
    fn test_if_expression() {
        let row = mock_row();
        let result =
            ExpressionEngine::evaluate_str("IF(customer_level = \"VIP\", \"A\", \"C\")", &row)
                .unwrap();
        assert_eq!(result, "A");
    }

    #[test]
    fn test_string_functions() {
        let row = mock_row();
        let result = ExpressionEngine::evaluate_str(
            "LEFT(steel_grade, 2) & \"-\" & MID(steel_grade, 3, 3)",
            &row,
        )
        .unwrap();
        assert_eq!(result, "Q2-35A");
    }

    #[test]
    fn test_round_expression() {
        let row = mock_row();
        let result = ExpressionEngine::evaluate_str("ROUND(thickness * 1000, 0)", &row).unwrap();
        assert_eq!(result, "1560");
    }

    #[test]
    fn test_date_diff() {
        let row = mock_row();
        let result =
            ExpressionEngine::evaluate_str("DATEDIFF(TODAY(), pdd, \"days\")", &row).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_validate() {
        assert!(ExpressionEngine::validate("IF(1>0, \"ok\", \"bad\")").is_ok());
        assert!(ExpressionEngine::validate("IF(1>0, \"ok\"").is_err());
    }
}
