use std::collections::VecDeque;

pub mod token;

pub use token::{Token, TokenKind};

pub fn lex(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();

    while !lexer.is_at_end() || !lexer.pending.is_empty() || lexer.indent_stack.len() > 1 {
        if let Some(token) = lexer.next_token() {
            tokens.push(token);
        }
    }

    tokens.push(Token::new(TokenKind::Eof, "", lexer.line, lexer.col));
    tokens
}

struct Lexer<'a> {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    pending: VecDeque<Token>,
    indent_stack: Vec<usize>,
    at_line_start: bool,
    pending_block: bool,
    brace_depth: usize,
    _source: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            pending: VecDeque::new(),
            indent_stack: vec![0],
            at_line_start: true,
            pending_block: false,
            brace_depth: 0,
            _source: source,
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        let ch = self.chars[self.pos];
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn next_token(&mut self) -> Option<Token> {
        if let Some(token) = self.pending.pop_front() {
            return Some(token);
        }

        if self.is_at_end() {
            if self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                return Some(Token::new(TokenKind::Punct, "}", self.line, self.col));
            }
            return None;
        }

        if self.at_line_start {
            self.handle_indentation();
            if let Some(token) = self.pending.pop_front() {
                return Some(token);
            }
        }

        let ch = self.peek()?;
        let line = self.line;
        let col = self.col;

        if ch == '\n' {
            self.advance();
            self.at_line_start = true;
            return Some(Token::new(TokenKind::Newline, "\\n", line, col));
        }

        if ch.is_whitespace() {
            self.advance();
            return None;
        }

        if ch == ':' {
            self.advance();
            self.pending_block = true;
            return None;
        }

        if ch == '/' {
            if self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                while let Some(next) = self.peek() {
                    if next == '\n' {
                        break;
                    }
                    self.advance();
                }
                return None;
            }
            if self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                while let Some(next) = self.peek() {
                    if next == '*' && self.peek_next() == Some('/') {
                        self.advance();
                        self.advance();
                        break;
                    }
                    self.advance();
                }
                return None;
            }
        }

        if ch.is_ascii_digit() {
            return Some(self.read_number());
        }

        if is_ident_start(ch) {
            return Some(self.read_identifier_or_keyword());
        }

        if ch == '"' || ch == '\'' {
            return Some(self.read_string());
        }

        if ch == ';' {
            self.advance();
            return Some(Token::new(TokenKind::Newline, ";", line, col));
        }

        if ch == '{' {
            self.advance();
            self.brace_depth += 1;
            return Some(Token::new(TokenKind::Punct, "{", line, col));
        }

        if ch == '}' {
            self.advance();
            if self.brace_depth > 0 {
                self.brace_depth -= 1;
            }
            return Some(Token::new(TokenKind::Punct, "}", line, col));
        }

        if let Some(token) = self.read_operator_or_punct() {
            return Some(token);
        }

        self.advance();
        None
    }

    fn handle_indentation(&mut self) {
        if self.brace_depth > 0 {
            self.consume_indent_whitespace();
            self.at_line_start = false;
            return;
        }

        let line = self.line;
        let col = self.col;
        let mut indent = 0usize;

        while let Some(ch) = self.peek() {
            if ch == ' ' {
                indent += 1;
                self.advance();
            } else if ch == '\t' {
                indent += 4;
                self.advance();
            } else {
                break;
            }
        }

        if matches!(self.peek(), Some('\n')) || self.peek().is_none() {
            self.at_line_start = false;
            return;
        }

        if self.peek() == Some('/') && self.peek_next() == Some('/') {
            self.skip_line_comment();
            self.at_line_start = false;
            return;
        }

        if self.peek() == Some('/') && self.peek_next() == Some('*') {
            self.skip_block_comment();
            self.at_line_start = true;
            return;
        }

        let current_indent = *self.indent_stack.last().unwrap_or(&0);

        if indent > current_indent {
            self.indent_stack.push(indent);
            self.pending
                .push_back(Token::new(TokenKind::Punct, "{", line, col));
            self.pending_block = false;
        } else {
            if self.pending_block {
                self.pending
                    .push_back(Token::new(TokenKind::Punct, "{", line, col));
                self.pending
                    .push_back(Token::new(TokenKind::Punct, "}", line, col));
                self.pending_block = false;
            }
            while indent < *self.indent_stack.last().unwrap_or(&0) {
                self.indent_stack.pop();
                self.pending
                    .push_back(Token::new(TokenKind::Punct, "}", line, col));
            }
        }

        self.at_line_start = false;
    }

    fn consume_indent_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();
        while let Some(next) = self.peek() {
            if next == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance();
        self.advance();
        while let Some(next) = self.peek() {
            if next == '*' && self.peek_next() == Some('/') {
                self.advance();
                self.advance();
                break;
            }
            self.advance();
        }
    }

    fn read_number(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') && self.peek_next().map(|c| c.is_ascii_digit()).unwrap_or(false)
        {
            value.push('.');
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    value.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Token::new(TokenKind::Number, value, line, col)
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if is_ident_continue(ch) {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = if is_keyword(&value) {
            TokenKind::Keyword
        } else {
            TokenKind::Identifier
        };

        Token::new(kind, value, line, col)
    }

    fn read_string(&mut self) -> Token {
        let line = self.line;
        let col = self.col;
        let quote = self.advance().unwrap_or('"');
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance();
                break;
            }
            if ch == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => {
                        value.push('\n');
                        self.advance();
                    }
                    Some('t') => {
                        value.push('\t');
                        self.advance();
                    }
                    Some('r') => {
                        value.push('\r');
                        self.advance();
                    }
                    Some('\\') => {
                        value.push('\\');
                        self.advance();
                    }
                    Some('"') => {
                        value.push('"');
                        self.advance();
                    }
                    Some('\'') => {
                        value.push('\'');
                        self.advance();
                    }
                    Some(other) => {
                        value.push(other);
                        self.advance();
                    }
                    None => break,
                }
                continue;
            }
            value.push(ch);
            self.advance();
        }
        Token::new(TokenKind::String, value, line, col)
    }

    fn read_operator_or_punct(&mut self) -> Option<Token> {
        let line = self.line;
        let col = self.col;
        let ch = self.peek()?;

        if let Some(next) = self.peek_next() {
            let two = [ch, next].iter().collect::<String>();
            if matches!(two.as_str(), "==" | "!=" | "<=" | ">=" | "&&" | "||") {
                self.advance();
                self.advance();
                return Some(Token::new(TokenKind::Operator, two, line, col));
            }
        }

        let single = ch.to_string();
        if matches!(ch, '+' | '-' | '*' | '/' | '%' | '<' | '>' | '=' | '!') {
            self.advance();
            return Some(Token::new(TokenKind::Operator, single, line, col));
        }

        if matches!(ch, '(' | ')' | '[' | ']' | ',' | '.') {
            self.advance();
            return Some(Token::new(TokenKind::Punct, single, line, col));
        }

        None
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "bikin"
            | "fun"
            | "kalo"
            | "ato"
            | "bentar"
            | "ulang"
            | "di"
            | "balikin"
            | "baka"
            | "lanjut"
            | "berhenti"
            | "true"
            | "false"
            | "nil"
            | "bener"
            | "salah"
            | "ya"
            | "tidak"
            | "kosong"
            | "butuh"
            | "ekspor"
            | "nani"
            | "yamete"
            | "sugoi"
    )
}
