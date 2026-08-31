use crate::ast::{BinaryOp, Expr, Program, Stmt};
use crate::token::{Token, TokenType};
use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn check(&mut self, token_type: &TokenType) -> bool {
        if let Some(token) = self.peek() {
            &token.token_type == token_type
        } else {
            false
        }
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::Eof) && self.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    // ── Statements ────────────────────────────────────────────────────────────

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&TokenType::Bang) {
            self.parse_var_decl()
        } else if self.match_token(&TokenType::Const) {
            self.parse_const_decl()
        } else if self.match_token(&TokenType::If) {
            self.parse_if_stmt()
        } else if self.match_token(&TokenType::While) {
            self.parse_while_stmt()
        } else if self.match_token(&TokenType::For) {
            self.parse_for_stmt()
        } else if self.match_token(&TokenType::Fn) {
            self.parse_fn_decl()
        } else if self.match_token(&TokenType::Type) {
            self.parse_type_decl()
        } else if self.match_token(&TokenType::Give) {
            let expr = self.parse_expression()?;
            Ok(Stmt::Give(expr))
        } else if self.match_token(&TokenType::Stop) {
            Ok(Stmt::Stop)
        } else if self.match_token(&TokenType::Skip) {
            Ok(Stmt::Skip)
        } else if self.match_token(&TokenType::SayBang) {
            self.parse_say_stmt()
        } else if self.match_token(&TokenType::Use) {
            self.parse_use_stmt()
        } else if self.match_token(&TokenType::Try) {
            self.parse_try_catch()
        } else if self.match_token(&TokenType::Throw) {
            let expr = self.parse_expression()?;
            Ok(Stmt::Throw(expr))
        } else if self.match_token(&TokenType::Match) {
            self.parse_match_stmt()
        } else {
            self.parse_assignment_or_expr()
        }
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, String> {
        if let Some(Token { token_type: TokenType::Ident(name), .. }) = self.advance() {
            // Optional type annotation: !name: Type = expr (parse and ignore type for now)
            if self.match_token(&TokenType::Colon) {
                self.advance(); // consume type name, ignore
            }
            if self.match_token(&TokenType::Assign) {
                let init = self.parse_expression()?;
                Ok(Stmt::VarDecl { name, init })
            } else {
                Err("Expected '=' after variable name".to_string())
            }
        } else {
            Err("Expected identifier after '!'".to_string())
        }
    }

    fn parse_const_decl(&mut self) -> Result<Stmt, String> {
        if let Some(Token { token_type: TokenType::Ident(name), .. }) = self.advance() {
            if self.match_token(&TokenType::Assign) {
                let init = self.parse_expression()?;
                Ok(Stmt::ConstDecl { name, init })
            } else {
                Err("Expected '=' after constant name".to_string())
            }
        } else {
            Err("Expected identifier after 'const'".to_string())
        }
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, String> {
        let condition = self.parse_expression()?;
        let then_branch = Box::new(self.parse_block()?);

        let mut elif_branches = Vec::new();
        let mut else_branch = None;

        loop {
            if self.match_token(&TokenType::Elif) {
                let elif_cond = self.parse_expression()?;
                let elif_body = self.parse_block()?;
                elif_branches.push((elif_cond, elif_body));
            } else if self.match_token(&TokenType::Else) {
                else_branch = Some(Box::new(self.parse_block()?));
                break;
            } else {
                break;
            }
        }

        Ok(Stmt::IfStmt { condition, then_branch, elif_branches, else_branch })
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, String> {
        let condition = self.parse_expression()?;
        let body = Box::new(self.parse_block()?);
        Ok(Stmt::WhileStmt { condition, body })
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, String> {
        let var = if let Some(Token { token_type: TokenType::Ident(name), .. }) = self.advance() {
            name
        } else {
            return Err("Expected variable name after 'for'".to_string());
        };
        if !self.match_token(&TokenType::In) {
            return Err("Expected 'in' after loop variable".to_string());
        }
        let iterable = self.parse_expression()?;
        let body = Box::new(self.parse_block()?);
        Ok(Stmt::ForStmt { var, iterable, body })
    }

    fn parse_fn_decl(&mut self) -> Result<Stmt, String> {
        let name = if let Some(Token { token_type: TokenType::Ident(n), .. }) = self.advance() {
            n
        } else {
            return Err("Expected function name after 'fn'".to_string());
        };
        if !self.match_token(&TokenType::LParen) {
            return Err(format!("Expected '(' after function name '{}'", name));
        }
        let mut params = Vec::new();
        if !self.check(&TokenType::RParen) {
            loop {
                if let Some(Token { token_type: TokenType::Ident(p), .. }) = self.advance() {
                    // Optional type annotation on param: param: Type
                    if self.match_token(&TokenType::Colon) {
                        self.advance(); // consume type, ignore
                    }
                    params.push(p);
                } else {
                    return Err("Expected parameter name".to_string());
                }
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        if !self.match_token(&TokenType::RParen) {
            return Err("Expected ')' after parameters".to_string());
        }
        let body = Box::new(self.parse_block()?);
        Ok(Stmt::FnDecl { name, params, body })
    }

    fn parse_type_decl(&mut self) -> Result<Stmt, String> {
        let name = if let Some(Token { token_type: TokenType::Ident(n), .. }) = self.advance() {
            n
        } else {
            return Err("Expected type name after 'type'".to_string());
        };
        if !self.match_token(&TokenType::Begin) {
            return Err(format!("Expected 'begin' after type name '{}'", name));
        }
        let mut methods = Vec::new();
        while !self.check(&TokenType::End) && !self.check(&TokenType::Eof) {
            if self.match_token(&TokenType::Fn) {
                methods.push(self.parse_fn_decl()?);
            } else {
                return Err("Only method declarations (fn) are allowed inside a type".to_string());
            }
        }
        if !self.match_token(&TokenType::End) {
            return Err("Expected 'end' to close type declaration".to_string());
        }
        Ok(Stmt::TypeDecl { name, methods })
    }

    fn parse_use_stmt(&mut self) -> Result<Stmt, String> {
        if let Some(Token { token_type: TokenType::String(path), .. }) = self.advance() {
            Ok(Stmt::Use(path))
        } else {
            Err("Expected file path string after 'use'".to_string())
        }
    }

    fn parse_try_catch(&mut self) -> Result<Stmt, String> {
        let try_block = Box::new(self.parse_block()?);
        if !self.match_token(&TokenType::Catch) {
            return Err("Expected 'catch' after try block".to_string());
        }
        let error_var = if let Some(Token { token_type: TokenType::Ident(n), .. }) = self.advance() {
            n
        } else {
            return Err("Expected error variable name after 'catch'".to_string());
        };
        let catch_block = Box::new(self.parse_block()?);
        Ok(Stmt::TryCatch { try_block, error_var, catch_block })
    }

    fn parse_match_stmt(&mut self) -> Result<Stmt, String> {
        let target = self.parse_expression()?;
        let mut cases = Vec::new();
        let mut else_branch = None;

        while self.match_token(&TokenType::Case) {
            let case_val = self.parse_expression()?;
            let case_body = self.parse_block()?;
            cases.push((case_val, case_body));
        }

        if self.match_token(&TokenType::Else) {
            else_branch = Some(Box::new(self.parse_block()?));
        }

        Ok(Stmt::MatchStmt { target, cases, else_branch })
    }

    fn parse_block(&mut self) -> Result<Stmt, String> {
        if !self.match_token(&TokenType::Begin) {
            return Err("Expected 'begin' at the start of block".to_string());
        }
        let mut statements = Vec::new();
        while !self.check(&TokenType::End)
            && !self.check(&TokenType::Eof)
            && !self.check(&TokenType::Elif)
            && !self.check(&TokenType::Else)
            && !self.check(&TokenType::Catch)
            && !self.check(&TokenType::Case)
        {
            statements.push(self.parse_statement()?);
        }
        if !self.match_token(&TokenType::End) {
            // Don't consume elif/else/catch/case — let their parsers handle them
            if !self.check(&TokenType::Elif)
                && !self.check(&TokenType::Else)
                && !self.check(&TokenType::Catch)
                && !self.check(&TokenType::Case)
            {
                return Err("Expected 'end' to close block".to_string());
            }
        }
        Ok(Stmt::Block(statements))
    }

    fn parse_say_stmt(&mut self) -> Result<Stmt, String> {
        let mut exprs = Vec::new();
        exprs.push(self.parse_expression()?);
        while self.match_token(&TokenType::Comma) {
            exprs.push(self.parse_expression()?);
        }
        Ok(Stmt::Say { exprs })
    }

    fn parse_assignment_or_expr(&mut self) -> Result<Stmt, String> {
        let expr = self.parse_expression()?;

        if self.match_token(&TokenType::Assign) {
            let value = self.parse_expression()?;
            match expr {
                // plain variable: x = val
                Expr::Ident(name) => Ok(Stmt::Assign { name, expr: value }),
                // index: list[0] = val
                Expr::Index { target, index } => {
                    if let Expr::Ident(name) = *target {
                        Ok(Stmt::IndexAssign { name, index: *index, value })
                    } else {
                        Err("Only simple variable indexing is supported on the left side of '='".to_string())
                    }
                }
                // member: obj.field = val  (also self.field = val)
                Expr::MemberAccess { object, field } => {
                    if let Expr::Ident(name) = *object {
                        Ok(Stmt::MemberAssign { name, field, value })
                    } else {
                        Err("Only simple variable.field is supported on the left side of '='".to_string())
                    }
                }
                _ => Err("Invalid assignment target".to_string()),
            }
        } else {
            Ok(Stmt::Expr(expr))
        }
    }

    // ── Expressions ───────────────────────────────────────────────────────────

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_logical_and()?;
        while self.match_token(&TokenType::Or) {
            let right = self.parse_logical_and()?;
            expr = Expr::BinaryOp(Box::new(expr), BinaryOp::Or, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_not()?;
        while self.match_token(&TokenType::And) {
            let right = self.parse_not()?;
            expr = Expr::BinaryOp(Box::new(expr), BinaryOp::And, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_not(&mut self) -> Result<Expr, String> {
        if self.match_token(&TokenType::Not) {
            let operand = self.parse_not()?;
            return Ok(Expr::UnaryNot(Box::new(operand)));
        }
        self.parse_relational()
    }

    fn parse_relational(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_term()?;

        while self.check(&TokenType::EqEq)
            || self.check(&TokenType::BangEq)
            || self.check(&TokenType::Less)
            || self.check(&TokenType::Greater)
            || self.check(&TokenType::LessEq)
            || self.check(&TokenType::GreaterEq)
        {
            let op_token = self.advance().unwrap();
            let op = match op_token.token_type {
                TokenType::EqEq => BinaryOp::Eq,
                TokenType::BangEq => BinaryOp::NotEq,
                TokenType::Less => BinaryOp::Lt,
                TokenType::Greater => BinaryOp::Gt,
                TokenType::LessEq => BinaryOp::LtEq,
                TokenType::GreaterEq => BinaryOp::GtEq,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;

        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let op_token = self.advance().unwrap();
            let op = match op_token.token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_postfix()?;

        while self.check(&TokenType::Star) || self.check(&TokenType::Slash) {
            let op_token = self.advance().unwrap();
            let op = match op_token.token_type {
                TokenType::Star => BinaryOp::Mul,
                TokenType::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };
            let right = self.parse_postfix()?;
            expr = Expr::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    /// Parses an atom and then chains any postfix operations: .field, .method(args), [index]
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_atom()?;

        loop {
            if self.match_token(&TokenType::Dot) {
                // .field or .method(args)
                let field = if let Some(Token { token_type: TokenType::Ident(f), .. }) = self.advance() {
                    f
                } else {
                    return Err("Expected field or method name after '.'".to_string());
                };

                if self.check(&TokenType::LParen) {
                    // method call: expr.method(args)
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    if !self.check(&TokenType::RParen) {
                        args.push(self.parse_expression()?);
                        while self.match_token(&TokenType::Comma) {
                            args.push(self.parse_expression()?);
                        }
                    }
                    if !self.match_token(&TokenType::RParen) {
                        return Err("Expected ')' after method arguments".to_string());
                    }
                    expr = Expr::MethodCall { object: Box::new(expr), method: field, args };
                } else {
                    // member access: expr.field
                    expr = Expr::MemberAccess { object: Box::new(expr), field };
                }
            } else if self.check(&TokenType::LBracket) {
                // index: expr[index]
                self.advance(); // consume '['
                let index = self.parse_expression()?;
                if !self.match_token(&TokenType::RBracket) {
                    return Err("Expected ']' after index expression".to_string());
                }
                expr = Expr::Index { target: Box::new(expr), index: Box::new(index) };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        if let Some(token) = self.advance() {
            match token.token_type {
                TokenType::Number(n) => Ok(Expr::Number(n)),
                TokenType::True => Ok(Expr::Number(1.0)),
                TokenType::False => Ok(Expr::Number(0.0)),
                TokenType::Nothing => Ok(Expr::Ident("nothing".to_string())),

                TokenType::String(s) => {
                    if s.contains('{') && s.contains('}') {
                        self.parse_interpolated_string(s)
                    } else {
                        Ok(Expr::String(s))
                    }
                }

                TokenType::Ident(i) => {
                    // function call: name(args)
                    if self.check(&TokenType::LParen) {
                        self.advance();
                        let mut args = Vec::new();
                        if !self.check(&TokenType::RParen) {
                            args.push(self.parse_expression()?);
                            while self.match_token(&TokenType::Comma) {
                                args.push(self.parse_expression()?);
                            }
                        }
                        if !self.match_token(&TokenType::RParen) {
                            return Err("Expected ')' after function arguments".to_string());
                        }
                        Ok(Expr::Call { callee: i, args })
                    } else {
                        Ok(Expr::Ident(i))
                    }
                }

                TokenType::LParen => {
                    let first = self.parse_expression()?;
                    if self.match_token(&TokenType::Comma) {
                        // Tuple: (expr, expr, ...)
                        let mut elements = vec![first];
                        if !self.check(&TokenType::RParen) {
                            loop {
                                elements.push(self.parse_expression()?);
                                if !self.match_token(&TokenType::Comma) {
                                    break;
                                }
                                if self.check(&TokenType::RParen) {
                                    break;
                                }
                            }
                        }
                        if !self.match_token(&TokenType::RParen) {
                            return Err("Expected ')' after tuple elements".to_string());
                        }
                        Ok(Expr::Tuple(elements))
                    } else {
                        // Grouping
                        if !self.match_token(&TokenType::RParen) {
                            return Err("Expected ')' after expression".to_string());
                        }
                        Ok(first)
                    }
                }

                TokenType::LBracket => self.parse_list_or_map(),

                _ => Err(format!(
                    "Unexpected token in expression: {:?}",
                    token.token_type
                )),
            }
        } else {
            Err("Unexpected end of file".to_string())
        }
    }

    fn parse_list_or_map(&mut self) -> Result<Expr, String> {
        if self.match_token(&TokenType::RBracket) {
            return Ok(Expr::List(Vec::new()));
        }

        let first = self.parse_expression()?;

        if self.match_token(&TokenType::Colon) {
            // Map: ["key": val, ...]
            let mut elements = Vec::new();
            let first_val = self.parse_expression()?;
            elements.push((first, first_val));
            while self.match_token(&TokenType::Comma) {
                let key = self.parse_expression()?;
                if !self.match_token(&TokenType::Colon) {
                    return Err("Expected ':' after map key".to_string());
                }
                let val = self.parse_expression()?;
                elements.push((key, val));
            }
            if !self.match_token(&TokenType::RBracket) {
                return Err("Expected ']' after map elements".to_string());
            }
            Ok(Expr::Map(elements))
        } else {
            // List: [val, ...]
            let mut elements = vec![first];
            while self.match_token(&TokenType::Comma) {
                elements.push(self.parse_expression()?);
            }
            if !self.match_token(&TokenType::RBracket) {
                return Err("Expected ']' after list elements".to_string());
            }
            Ok(Expr::List(elements))
        }
    }

    fn parse_interpolated_string(&mut self, s: String) -> Result<Expr, String> {
        let mut exprs = Vec::new();
        let mut current_literal = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if !current_literal.is_empty() {
                    exprs.push(Expr::String(current_literal.clone()));
                    current_literal.clear();
                }
                let mut inner_expr_str = String::new();
                for inner_c in chars.by_ref() {
                    if inner_c == '}' {
                        break;
                    }
                    inner_expr_str.push(inner_c);
                }
                let mut inner_lexer = crate::lexer::Lexer::new(&inner_expr_str);
                let inner_tokens = inner_lexer.tokenize()?;
                let mut inner_parser = Parser::new(inner_tokens);
                let inner_expr = inner_parser.parse_expression()?;
                exprs.push(inner_expr);
            } else {
                current_literal.push(c);
            }
        }

        if !current_literal.is_empty() {
            exprs.push(Expr::String(current_literal));
        }

        if exprs.is_empty() {
            return Ok(Expr::String("".to_string()));
        }

        let mut final_expr = exprs[0].clone();
        for expr in exprs.into_iter().skip(1) {
            final_expr = Expr::BinaryOp(Box::new(final_expr), BinaryOp::Add, Box::new(expr));
        }

        Ok(final_expr)
    }
}
