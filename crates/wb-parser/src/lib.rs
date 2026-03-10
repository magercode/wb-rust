use wb_ast::{BinaryOp, Expr, Literal, Stmt, UnaryOp};
use wb_diagnostics::Diagnostic;
use wb_lexer::token::{Token, TokenKind};

pub fn parse(tokens: &[Token]) -> Result<Vec<Stmt>, Diagnostic> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse_program(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
            self.skip_newlines();
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, Diagnostic> {
        if self.match_keyword("bikin") {
            return self.var_declaration();
        }
        if self.match_keyword("fun") {
            return self.fun_declaration();
        }
        self.statement()
    }

    fn statement(&mut self) -> Result<Stmt, Diagnostic> {
        if self.match_keyword("butuh") {
            return self.import_statement();
        }
        if self.match_keyword("ekspor") {
            return self.export_statement();
        }
        if self.match_keyword("kalo") {
            return self.if_statement_after_keyword();
        }
        if self.match_keyword("ulang") {
            return self.for_each_statement();
        }
        if self.match_keyword("bentar") {
            return self.while_statement();
        }
        if self.match_keyword("balikin") {
            return self.return_statement();
        }
        if self.match_keyword("lanjut") {
            self.consume_terminator();
            return Ok(Stmt::Continue);
        }
        if self.match_keyword("berhenti") {
            self.consume_terminator();
            return Ok(Stmt::Break);
        }
        if self.match_keyword("baka") {
            return self.baka_statement();
        }
        if self.match_punct("{") {
            let body = self.block()?;
            return Ok(Stmt::Block(body));
        }
        self.assignment_or_expression()
    }

    fn var_declaration(&mut self) -> Result<Stmt, Diagnostic> {
        let name = self.consume_identifier("Expected variable name after 'bikin'")?;
        if !self.match_operator("=") {
            return Err(self.error_at_current("Expected '=' after variable name"));
        }
        let value = if self.check_terminator() {
            Expr::Literal(Literal::Nil)
        } else {
            self.expression()?
        };
        self.consume_terminator();
        Ok(Stmt::Let { name, value })
    }

    fn fun_declaration(&mut self) -> Result<Stmt, Diagnostic> {
        let name = self.consume_identifier("Expected function name after 'fun'")?;
        self.consume_punct("(", "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check_punct(")") {
            loop {
                let param = self.consume_identifier("Expected parameter name")?;
                params.push(param);
                if !self.match_punct(",") {
                    break;
                }
            }
        }
        self.consume_punct(")", "Expected ')' after parameters")?;
        self.consume_block_start("Expected block after function declaration")?;
        let body = self.block()?;
        Ok(Stmt::Function { name, params, body })
    }

    fn if_statement_after_keyword(&mut self) -> Result<Stmt, Diagnostic> {
        let condition = self.parse_condition()?;
        self.consume_block_start("Expected block after if condition")?;
        let then_branch = self.block()?;

        let mut else_branch = None;
        if self.match_keyword("ato") {
            if self.match_keyword("kalo") {
                let nested_if = self.if_statement_after_keyword()?;
                else_branch = Some(vec![nested_if]);
            } else {
                self.consume_block_start("Expected block after 'ato'")?;
                let else_block = self.block()?;
                else_branch = Some(else_block);
            }
        }

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let condition = self.parse_condition()?;
        self.consume_block_start("Expected block after while condition")?;
        let body = self.block()?;
        Ok(Stmt::While { condition, body })
    }

    fn for_each_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let name = self.consume_identifier("Expected iterator name after 'ulang'")?;
        if !self.match_keyword("di") {
            return Err(self.error_at_current("Expected 'di' after iterator name"));
        }
        let iterable = self.expression()?;
        self.consume_block_start("Expected block after ulang statement")?;
        let body = self.block()?;
        Ok(Stmt::ForEach { name, iterable, body })
    }

    fn return_statement(&mut self) -> Result<Stmt, Diagnostic> {
        if self.check_terminator() {
            self.consume_terminator();
            return Ok(Stmt::Return(None));
        }
        let value = self.expression()?;
        self.consume_terminator();
        Ok(Stmt::Return(Some(value)))
    }

    fn import_statement(&mut self) -> Result<Stmt, Diagnostic> {
        if self.check_terminator() {
            return Err(self.error_at_current("Expected module name after 'butuh'"));
        }
        let module = self.expression()?;
        self.consume_terminator();
        Ok(Stmt::Import { module })
    }

    fn export_statement(&mut self) -> Result<Stmt, Diagnostic> {
        if self.check_terminator() {
            return Err(self.error_at_current("Expected value after 'ekspor'"));
        }
        let value = self.expression()?;
        self.consume_terminator();
        Ok(Stmt::Export { value })
    }

    fn baka_statement(&mut self) -> Result<Stmt, Diagnostic> {
        let mut args = Vec::new();
        if self.match_punct("(") {
            if !self.check_punct(")") {
                loop {
                    args.push(self.expression()?);
                    if !self.match_punct(",") {
                        break;
                    }
                }
            }
            self.consume_punct(")", "Expected ')' after baka arguments")?;
        } else if !self.check_terminator() {
            args.push(self.expression()?);
        }
        self.consume_terminator();
        let call = Expr::Call {
            callee: Box::new(Expr::Identifier("baka".to_string())),
            args,
        };
        Ok(Stmt::Expr(call))
    }

    fn assignment_or_expression(&mut self) -> Result<Stmt, Diagnostic> {
        if self.check_kind(TokenKind::Identifier) && self.peek_next_is_operator("=") {
            let name = self.consume_identifier("Expected variable name")?;
            self.consume_operator("=", "Expected '=' in assignment")?;
            let value = self.expression()?;
            self.consume_terminator();
            return Ok(Stmt::Assign { name, value });
        }
        let expr = self.expression()?;
        self.consume_terminator();
        Ok(Stmt::Expr(expr))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Diagnostic> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.check_punct("}") && !self.is_at_end() {
            statements.push(self.declaration()?);
            self.skip_newlines();
        }
        self.consume_punct("}", "Expected '}' after block")?;
        Ok(statements)
    }

    fn consume_block_start(&mut self, message: &str) -> Result<(), Diagnostic> {
        self.skip_newlines();
        self.consume_punct("{", message)
    }

    fn parse_condition(&mut self) -> Result<Expr, Diagnostic> {
        if self.match_punct("(") {
            let expr = self.expression()?;
            self.consume_punct(")", "Expected ')' after condition")?;
            Ok(expr)
        } else {
            self.expression()
        }
    }

    fn expression(&mut self) -> Result<Expr, Diagnostic> {
        self.or()
    }

    fn or(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.and()?;
        while self.match_operator("||") {
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::Or,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.equality()?;
        while self.match_operator("&&") {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: BinaryOp::And,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.comparison()?;
        while self.match_operator("==") || self.match_operator("!=") {
            let op_token = self.previous().lexeme.clone();
            let op = if op_token == "==" {
                BinaryOp::Equal
            } else {
                BinaryOp::NotEqual
            };
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.term()?;
        loop {
            let op = if self.match_operator("<") {
                Some(BinaryOp::Less)
            } else if self.match_operator("<=") {
                Some(BinaryOp::LessEqual)
            } else if self.match_operator(">") {
                Some(BinaryOp::Greater)
            } else if self.match_operator(">=") {
                Some(BinaryOp::GreaterEqual)
            } else {
                None
            };

            if let Some(op) = op {
                let right = self.term()?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.factor()?;
        while self.match_operator("+") || self.match_operator("-") {
            let op_token = self.previous().lexeme.clone();
            let op = if op_token == "+" {
                BinaryOp::Add
            } else {
                BinaryOp::Subtract
            };
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.unary()?;
        while self.match_operator("*") || self.match_operator("/") || self.match_operator("%") {
            let op_token = self.previous().lexeme.clone();
            let op = match op_token.as_str() {
                "*" => BinaryOp::Multiply,
                "/" => BinaryOp::Divide,
                _ => BinaryOp::Modulo,
            };
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Diagnostic> {
        if self.match_operator("!") {
            let expr = self.unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        if self.match_operator("-") {
            let expr = self.unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
            });
        }
        self.call()
    }

    fn call(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.primary()?;
        loop {
            if self.match_punct("(") {
                let mut args = Vec::new();
                if !self.check_punct(")") {
                    loop {
                        args.push(self.expression()?);
                        if !self.match_punct(",") {
                            break;
                        }
                    }
                }
                self.consume_punct(")", "Expected ')' after arguments")?;
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.match_punct("[") {
                let index = self.expression()?;
                self.consume_punct("]", "Expected ']' after index expression")?;
                expr = Expr::Index {
                    target: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        if self.check_kind(TokenKind::Number) {
            let token = self.advance().clone();
            let value = token
                .lexeme
                .parse::<f64>()
                .map_err(|_| self.error(&token, "Invalid number literal"))?;
            return Ok(Expr::Literal(Literal::Number(value)));
        }
        if self.check_kind(TokenKind::String) {
            let token = self.advance().clone();
            return Ok(Expr::Literal(Literal::String(token.lexeme)));
        }
        if self.check_kind(TokenKind::Identifier) {
            let token = self.advance().clone();
            return Ok(Expr::Identifier(token.lexeme));
        }
        if self.check_kind(TokenKind::Keyword) {
            let token = self.advance().clone();
            match token.lexeme.as_str() {
                "true" | "bener" | "ya" => {
                    return Ok(Expr::Literal(Literal::Boolean(true)));
                }
                "false" | "salah" | "tidak" => {
                    return Ok(Expr::Literal(Literal::Boolean(false)));
                }
                "nil" | "kosong" => return Ok(Expr::Literal(Literal::Nil)),
                "baka" => return Ok(Expr::Identifier("baka".to_string())),
                _ => {
                    return Err(self.error(&token, "Unexpected keyword in expression"));
                }
            }
        }
        if self.match_punct("(") {
            let expr = self.expression()?;
            self.consume_punct(")", "Expected ')' after expression")?;
            return Ok(expr);
        }
        if self.match_punct("[") {
            let mut items = Vec::new();
            if !self.check_punct("]") {
                loop {
                    items.push(self.expression()?);
                    if !self.match_punct(",") {
                        break;
                    }
                }
            }
            self.consume_punct("]", "Expected ']' after array literal")?;
            return Ok(Expr::Array(items));
        }

        Err(self.error_at_current("Expected expression"))
    }

    fn match_keyword(&mut self, keyword: &str) -> bool {
        if self.check_keyword(keyword) {
            self.advance();
            return true;
        }
        false
    }

    fn match_operator(&mut self, op: &str) -> bool {
        if self.check_operator(op) {
            self.advance();
            return true;
        }
        false
    }

    fn match_punct(&mut self, punct: &str) -> bool {
        if self.check_punct(punct) {
            self.advance();
            return true;
        }
        false
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, Diagnostic> {
        if self.check_kind(TokenKind::Identifier) {
            return Ok(self.advance().lexeme.clone());
        }
        Err(self.error_at_current(message))
    }

    fn consume_punct(&mut self, punct: &str, message: &str) -> Result<(), Diagnostic> {
        if self.check_punct(punct) {
            self.advance();
            return Ok(());
        }
        Err(self.error_at_current(message))
    }

    fn consume_operator(&mut self, op: &str, message: &str) -> Result<(), Diagnostic> {
        if self.check_operator(op) {
            self.advance();
            return Ok(());
        }
        Err(self.error_at_current(message))
    }

    fn skip_newlines(&mut self) {
        while self.check_kind(TokenKind::Newline) {
            self.advance();
        }
    }

    fn consume_terminator(&mut self) {
        self.skip_newlines();
    }

    fn check_terminator(&self) -> bool {
        self.check_kind(TokenKind::Newline) || self.check_punct("}") || self.is_at_end()
    }

    fn check_kind(&self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.peek().kind == kind
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        self.check_kind(TokenKind::Keyword) && self.peek().lexeme == keyword
    }

    fn check_operator(&self, op: &str) -> bool {
        self.check_kind(TokenKind::Operator) && self.peek().lexeme == op
    }

    fn check_punct(&self, punct: &str) -> bool {
        self.check_kind(TokenKind::Punct) && self.peek().lexeme == punct
    }

    fn peek_next_is_operator(&self, op: &str) -> bool {
        if self.current + 1 >= self.tokens.len() {
            return false;
        }
        let token = &self.tokens[self.current + 1];
        token.kind == TokenKind::Operator && token.lexeme == op
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn error_at_current(&self, message: &str) -> Diagnostic {
        self.error(self.peek(), message)
    }

    fn error(&self, token: &Token, message: &str) -> Diagnostic {
        Diagnostic::new(format!(
            "{} (line {}, col {})",
            message, token.line, token.col
        ))
    }
}
