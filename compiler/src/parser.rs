use crate::ast::*;
use crate::lexer::{Token, Keyword, Span};

pub struct Parser {
    tokens: Vec<Token>,
    spans: Vec<Span>,
    current: usize,
    /// Controls whether `identifier {` should be parsed as a struct literal.
    /// Set to false in contexts where `{` starts a block (e.g., match expressions).
    allow_struct_literal: bool,
    /// Tracks expression nesting depth to prevent stack overflow on deeply nested input.
    expression_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let span_count = tokens.len();
        Self {
            tokens,
            spans: vec![Span { line: 0, col: 0 }; span_count],
            current: 0,
            allow_struct_literal: true,
            expression_depth: 0,
        }
    }

    /// Create a parser with source span information for better error messages.
    pub fn new_with_spans(tokens: Vec<Token>, spans: Vec<Span>) -> Self {
        Self {
            tokens,
            spans,
            current: 0,
            allow_struct_literal: true,
            expression_depth: 0,
        }
    }

    /// Get the source span at the current token position.
    fn current_span(&self) -> Span {
        if self.current < self.spans.len() {
            self.spans[self.current]
        } else {
            Span { line: 0, col: 0 }
        }
    }

    /// Format an error message with source location.
    fn error_at_current(&self, msg: &str) -> String {
        let span = self.current_span();
        if span.line > 0 {
            format!("{} (line {}, col {})", msg, span.line, span.col)
        } else {
            msg.to_string()
        }
    }
    
    pub fn parse(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        
        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }
        
        Ok(Program { items })
    }
    
    fn parse_item(&mut self) -> Result<Item, String> {
        // Parse attributes first (e.g., #[userland], #[kernel], #[baremetal])
        let mut profile: Option<String> = None;
        let mut _is_interrupt = false;
        while self.check_token(Token::Hash) {
            self.advance(); // skip #
            self.expect_token(Token::LeftBracket)?;
            let attr_name = self.expect_identifier()?;
            self.expect_token(Token::RightBracket)?;
            
            // Check for known attributes
            match attr_name.as_str() {
                "userland" | "kernel" | "baremetal" => {
                    profile = Some(attr_name);
                }
                "interrupt" => {
                    _is_interrupt = true;
                }
                _ => {
                    // Ignore other attributes for forward-compatibility
                }
            }
        }
        
        let is_pub = self.check_keyword(Keyword::Pub);
        if is_pub {
            self.advance();
        }
        
        if self.check_keyword(Keyword::Func) {
            Ok(Item::Function(self.parse_function(is_pub, profile)?))
        } else if self.check_keyword(Keyword::Extern) {
            Ok(Item::ExternFunction(self.parse_extern_function(is_pub)?))
        } else if self.check_keyword(Keyword::Struct) {
            Ok(Item::Struct(self.parse_struct(is_pub)?))
        } else if self.check_keyword(Keyword::Enum) {
            Ok(Item::Enum(self.parse_enum(is_pub)?))
        } else if self.check_keyword(Keyword::Impl) {
            Ok(Item::Impl(self.parse_impl()?))
        } else if self.check_keyword(Keyword::Trait) {
            Ok(Item::Trait(self.parse_trait(is_pub)?))
        } else if self.check_keyword(Keyword::Mod) {
            Ok(Item::Module(self.parse_module(is_pub)?))
        } else if self.check_keyword(Keyword::Import) || self.check_keyword(Keyword::Use) {
            Ok(Item::Import(self.parse_import()?))
        } else if self.check_keyword(Keyword::Const) {
            Ok(Item::Const(self.parse_const(is_pub)?))
        } else if self.check_keyword(Keyword::Let) {
            Ok(Item::Const(self.parse_let_global(is_pub)?))
        } else {
            Err(self.error_at_current(&format!("Unexpected token: {:?}", self.peek())))
        }
    }
    
    fn parse_function(&mut self, is_pub: bool, profile: Option<String>) -> Result<Function, String> {
        self.expect_keyword(Keyword::Func)?;
        
        let is_unsafe = self.check_keyword(Keyword::Unsafe);
        if is_unsafe {
            self.advance();
        }
        
        let name = self.expect_identifier()?;
        
        // Parse optional generic type parameters: <T, U>
        let type_params = if self.check_token(Token::Lt) {
            self.advance(); // consume '<'
            let mut params = Vec::new();
            loop {
                params.push(self.expect_identifier()?);
                if !self.check_token(Token::Comma) {
                    break;
                }
                self.advance();
            }
            self.expect_token(Token::Gt)?;
            params
        } else {
            Vec::new()
        };
        
        self.expect_token(Token::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect_token(Token::RightParen)?;
        
        let return_type = if self.check_token(Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(Function {
            name,
            type_params,
            params,
            return_type,
            body,
            is_unsafe,
            is_pub,
            profile,
        })
    }

    fn parse_extern_function(&mut self, is_pub: bool) -> Result<ExternFunction, String> {
        self.expect_keyword(Keyword::Extern)?;
        self.expect_keyword(Keyword::Func)?;

        let name = self.expect_identifier()?;
        self.expect_token(Token::LeftParen)?;
        let params = self.parse_parameters()?;
        self.expect_token(Token::RightParen)?;

        let return_type = if self.check_token(Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect_token(Token::Semicolon)?;

        Ok(ExternFunction {
            name,
            params,
            return_type,
            is_pub,
        })
    }
    
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, String> {
        let mut params = Vec::new();
        
        if self.check_token(Token::RightParen) {
            return Ok(params);
        }
        
        loop {
            let name = self.expect_identifier()?;
            self.expect_token(Token::Colon)?;
            let ty = self.parse_type()?;
            params.push(Parameter { name, ty });
            
            if !self.check_token(Token::Comma) {
                break;
            }
            self.advance();
        }
        
        Ok(params)
    }
    
    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect_token(Token::LeftBrace)?;
        let mut statements = Vec::new();
        
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.expect_token(Token::RightBrace)?;
        Ok(Block { statements })
    }
    
    fn parse_statement(&mut self) -> Result<Statement, String> {
        if self.check_keyword(Keyword::Let) {
            Ok(Statement::Let(self.parse_let()?))
        } else if self.check_keyword(Keyword::Return) {
            self.advance();
            let expr = if !self.check_token(Token::Semicolon) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.expect_token(Token::Semicolon)?;
            Ok(Statement::Return(expr))
        } else if self.check_keyword(Keyword::If) {
            Ok(Statement::If(self.parse_if()?))
        } else if self.check_keyword(Keyword::While) {
            Ok(Statement::While(self.parse_while()?))
        } else if self.check_keyword(Keyword::For) {
            Ok(Statement::For(self.parse_for()?))
        } else if self.check_keyword(Keyword::Loop) {
            Ok(Statement::Loop(self.parse_loop()?))
        } else if self.check_keyword(Keyword::Match) {
            Ok(Statement::Match(self.parse_match()?))
        } else if self.check_keyword(Keyword::Break) {
            self.advance();
            self.expect_token(Token::Semicolon)?;
            Ok(Statement::Break)
        } else if self.check_keyword(Keyword::Continue) {
            self.advance();
            self.expect_token(Token::Semicolon)?;
            Ok(Statement::Continue)
        } else {
            // Parse an expression, but check if it's actually an assignment statement
            // This allows `x = value;` and `x += value;` to be parsed as Statement::Assign
            let expr = self.parse_or()?;  // Parse up to assignment level without consuming =
            
            // Check for simple assignment: identifier = value;
            if self.check_token(Token::Eq) {
                self.advance(); // consume '='
                let value = self.parse_expression()?;
                self.expect_token(Token::Semicolon)?;
                
                if let Expression::Variable(name) = expr {
                    return Ok(Statement::Assign(crate::ast::AssignStatement {
                        target: name,
                        value,
                    }));
                } else {
                    return Err("Can only assign to variables".to_string());
                }
            }
            
            // Check for compound assignment: identifier += value;
            if self.check_token(Token::PlusEq) || self.check_token(Token::MinusEq) ||
               self.check_token(Token::StarEq) || self.check_token(Token::SlashEq) ||
               self.check_token(Token::PercentEq) {
                
                let op = if self.check_token(Token::PlusEq) {
                    crate::ast::BinaryOperator::Add
                } else if self.check_token(Token::MinusEq) {
                    crate::ast::BinaryOperator::Sub
                } else if self.check_token(Token::StarEq) {
                    crate::ast::BinaryOperator::Mul
                } else if self.check_token(Token::SlashEq) {
                    crate::ast::BinaryOperator::Div
                } else {
                    crate::ast::BinaryOperator::Mod
                };
                
                self.advance(); // consume compound operator
                let rhs = self.parse_expression()?;
                self.expect_token(Token::Semicolon)?;
                
                // Desugar: x += 1  →  x = x + 1
                if let Expression::Variable(name) = expr {
                    return Ok(Statement::Assign(crate::ast::AssignStatement {
                        target: name.clone(),
                        value: Expression::BinaryOp {
                            op,
                            left: Box::new(Expression::Variable(name)),
                            right: Box::new(rhs),
                        },
                    }));
                } else {
                    return Err("Can only assign to variables".to_string());
                }
            }
            
            // Not an assignment - just a regular expression statement
            self.expect_token(Token::Semicolon)?;
            Ok(Statement::Expr(expr))
        }
    }
    
    fn parse_let(&mut self) -> Result<LetStatement, String> {
        self.expect_keyword(Keyword::Let)?;
        let mutable = self.check_keyword(Keyword::Mut);
        if mutable {
            self.advance();
        }
        
        // Determine the let pattern
        let pattern = if self.check_token(Token::LeftParen) {
            // Tuple destructuring: let (a, b, c) = ...
            self.advance(); // consume '('
            let mut names = Vec::new();
            if !self.check_token(Token::RightParen) {
                loop {
                    names.push(self.expect_identifier()?);
                    if !self.check_token(Token::Comma) {
                        break;
                    }
                    self.advance();
                }
            }
            self.expect_token(Token::RightParen)?;
            LetPattern::Tuple(names)
        } else {
            let name = self.expect_identifier()?;
            if self.check_token(Token::LeftBrace) {
                // Struct destructuring: let TypeName { x, y } = ...
                self.advance(); // consume '{'
                let mut fields = Vec::new();
                if !self.check_token(Token::RightBrace) {
                    loop {
                        fields.push(self.expect_identifier()?);
                        if !self.check_token(Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                self.expect_token(Token::RightBrace)?;
                LetPattern::Struct { ty_name: name, fields }
            } else {
                // Simple binding: let x = ...
                LetPattern::Name(name)
            }
        };

        let ty = if self.check_token(Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.expect_token(Token::Eq)?;
        let value = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;
        
        Ok(LetStatement {
            pattern,
            mutable,
            ty,
            value,
        })
    }
    
    fn parse_if(&mut self) -> Result<IfStatement, String> {
        self.expect_keyword(Keyword::If)?;
        // Use parse_expression_no_struct to avoid interpreting `identifier {` as a struct literal
        let condition = self.parse_expression_no_struct()?;
        let then_block = self.parse_block()?;
        let else_block = if self.check_keyword(Keyword::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(IfStatement {
            condition,
            then_block,
            else_block,
        })
    }
    
    fn parse_while(&mut self) -> Result<WhileStatement, String> {
        self.expect_keyword(Keyword::While)?;
        // Use parse_expression_no_struct to avoid interpreting `identifier {` as a struct literal
        // The `{` after the condition starts the while body block, not a struct literal
        let condition = self.parse_expression_no_struct()?;
        let body = self.parse_block()?;
        Ok(WhileStatement { condition, body })
    }
    
    fn parse_for(&mut self) -> Result<ForStatement, String> {
        self.expect_keyword(Keyword::For)?;
        let var = self.expect_identifier()?;
        self.expect_keyword(Keyword::In)?;
        // Use parse_expression_no_struct to avoid interpreting `identifier {` as a struct literal
        let iterable = self.parse_expression_no_struct()?;
        let body = self.parse_block()?;
        Ok(ForStatement { var, iterable, body })
    }
    
    fn parse_loop(&mut self) -> Result<LoopStatement, String> {
        self.expect_keyword(Keyword::Loop)?;
        let body = self.parse_block()?;
        Ok(LoopStatement { body })
    }
    
    fn parse_match(&mut self) -> Result<MatchStatement, String> {
        self.expect_keyword(Keyword::Match)?;
        // Use parse_expression_no_struct to avoid consuming the match body's { as a struct literal
        let expr = self.parse_expression_no_struct()?;
        self.expect_token(Token::LeftBrace)?;
        
        let mut arms = Vec::new();
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.expect_token(Token::FatArrow)?;
            let body = if self.check_keyword(Keyword::Return) {
                self.advance(); // consume `return`
                let ret_expr = if self.check_token(Token::Comma) || self.check_token(Token::RightBrace) {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                Expression::Block(Block {
                    statements: vec![Statement::Return(ret_expr)],
                })
            } else {
                self.parse_expression()?
            };
            if self.check_token(Token::Comma) {
                self.advance();
            }
            arms.push(MatchArm { pattern, body });
        }
        
        self.expect_token(Token::RightBrace)?;
        Ok(MatchStatement { expr: Box::new(expr), arms })
    }
    
    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        match self.peek() {
            Token::Identifier(_) => {
                let name = self.expect_identifier()?;
                if name == "_" {
                    Ok(Pattern::Wildcard)
                } else if self.check_token(Token::ColonColon) {
                    self.advance(); // consume '::'
                    let variant = self.expect_identifier()?;
                    let data = if self.check_token(Token::LeftParen) {
                        self.advance(); // consume '('
                        let inner = self.parse_pattern()?;
                        self.expect_token(Token::RightParen)?;
                        Some(Box::new(inner))
                    } else {
                        None
                    };

                    Ok(Pattern::EnumVariant {
                        ty: Type::Named(name),
                        variant,
                        data,
                    })
                } else if self.check_token(Token::LeftParen) {
                    self.advance(); // consume '('
                    let inner = self.parse_pattern()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(Pattern::EnumVariant {
                        ty: Type::Named(String::new()),
                        variant: name,
                        data: Some(Box::new(inner)),
                    })
                } else {
                    Ok(Pattern::Variable(name))
                }
            }
            Token::IntLiteral(_) => {
                if let Token::IntLiteral(n) = self.advance().clone() {
                    Ok(Pattern::Literal(Literal::Int(n)))
                } else {
                    unreachable!()
                }
            }
            Token::StringLiteral(_) => {
                if let Token::StringLiteral(s) = self.advance().clone() {
                    Ok(Pattern::Literal(Literal::String(s)))
                } else {
                    unreachable!()
                }
            }
            Token::BoolLiteral(_) => {
                if let Token::BoolLiteral(b) = self.advance().clone() {
                    Ok(Pattern::Literal(Literal::Bool(b)))
                } else {
                    unreachable!()
                }
            }
            _ => Err("Invalid pattern".to_string())
        }
    }
    
    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.expression_depth += 1;
        if self.expression_depth > 64 {
            self.expression_depth -= 1;
            return Err("Expression too deeply nested (limit: 64 levels). Simplify or break into sub-expressions.".to_string());
        }
        let result = self.parse_assignment();
        self.expression_depth -= 1;
        result
    }
    
    /// Parse an expression but don't treat `identifier {` as a struct literal.
    /// This is used in match expressions where the `{` starts the match body, not a struct literal.
    fn parse_expression_no_struct(&mut self) -> Result<Expression, String> {
        let saved = self.allow_struct_literal;
        self.allow_struct_literal = false;
        let result = self.parse_assignment();
        self.allow_struct_literal = saved;
        result
    }
    
    fn parse_assignment(&mut self) -> Result<Expression, String> {
        // Assignments are now handled at statement level (Statement::Assign)
        // parse_expression() just continues to parse_or() without handling =
        // This prevents assignment from being used as an expression
        self.parse_or()
    }
    
    fn parse_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_and()?;
        
        while self.check_token(Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            expr = Expression::BinaryOp {
                op: crate::ast::BinaryOperator::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_bitwise_or()?;
        
        while self.check_token(Token::And) {
            self.advance();
            let right = self.parse_bitwise_or()?;
            expr = Expression::BinaryOp {
                op: crate::ast::BinaryOperator::And,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_bitwise_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_bitwise_xor()?;
        
        while self.check_token(Token::Pipe) {
            self.advance();
            let right = self.parse_bitwise_xor()?;
            expr = Expression::BinaryOp {
                op: crate::ast::BinaryOperator::BitOr,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_bitwise_xor(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_bitwise_and()?;
        
        while self.check_token(Token::BitXor) {
            self.advance();
            let right = self.parse_bitwise_and()?;
            expr = Expression::BinaryOp {
                op: crate::ast::BinaryOperator::BitXor,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_bitwise_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_equality()?;
        
        while self.check_token(Token::Ampersand) {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expression::BinaryOp {
                op: crate::ast::BinaryOperator::BitAnd,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;
        
        while self.check_token(Token::EqEq) || self.check_token(Token::Ne) {
            let op = if self.check_token(Token::EqEq) {
                self.advance();
                crate::ast::BinaryOperator::Eq
            } else {
                self.advance();
                crate::ast::BinaryOperator::Ne
            };
            
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_shift()?;
        
        while self.check_token(Token::Lt) || self.check_token(Token::Le) ||
              self.check_token(Token::Gt) || self.check_token(Token::Ge) {
            let op = if self.check_token(Token::Lt) {
                self.advance();
                crate::ast::BinaryOperator::Lt
            } else if self.check_token(Token::Le) {
                self.advance();
                crate::ast::BinaryOperator::Le
            } else if self.check_token(Token::Gt) {
                self.advance();
                crate::ast::BinaryOperator::Gt
            } else {
                self.advance();
                crate::ast::BinaryOperator::Ge
            };
            
            let right = self.parse_shift()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_shift(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_range()?;
        
        while self.check_token(Token::Shl) || self.check_token(Token::Shr) {
            let op = if self.check_token(Token::Shl) {
                self.advance();
                crate::ast::BinaryOperator::Shl
            } else {
                self.advance();
                crate::ast::BinaryOperator::Shr
            };
            
            let right = self.parse_range()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_range(&mut self) -> Result<Expression, String> {
        let start = self.parse_term()?;
        
        if self.check_token(Token::DotDot) {
            self.advance();
            // Check if there's an end value
            if self.is_at_end() || 
               self.check_token(Token::LeftBrace) || 
               self.check_token(Token::RightParen) ||
               self.check_token(Token::Comma) ||
               self.check_token(Token::Semicolon) {
                // Open range like `0..`
                Ok(Expression::Range {
                    start: Some(Box::new(start)),
                    end: None,
                })
            } else {
                // Closed range like `0..5`
                let end = self.parse_term()?;
                Ok(Expression::Range {
                    start: Some(Box::new(start)),
                    end: Some(Box::new(end)),
                })
            }
        } else {
            Ok(start)
        }
    }
    
    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_factor()?;
        
        while self.check_token(Token::Plus) || self.check_token(Token::Minus) {
            let op = if self.check_token(Token::Plus) {
                self.advance();
                crate::ast::BinaryOperator::Add
            } else {
                self.advance();
                crate::ast::BinaryOperator::Sub
            };
            
            let right = self.parse_factor()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;
        
        while self.check_token(Token::Star) || self.check_token(Token::Slash) ||
              self.check_token(Token::Percent) {
            let op = if self.check_token(Token::Star) {
                self.advance();
                crate::ast::BinaryOperator::Mul
            } else if self.check_token(Token::Slash) {
                self.advance();
                crate::ast::BinaryOperator::Div
            } else {
                self.advance();
                crate::ast::BinaryOperator::Mod
            };
            
            let right = self.parse_unary()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.check_token(Token::Minus) {
            self.advance();
            let operand = self.parse_unary()?;
            Ok(Expression::UnaryOp {
                op: crate::ast::UnaryOperator::Neg,
                operand: Box::new(operand),
            })
        } else if self.check_token(Token::Exclamation) {
            self.advance();
            let operand = self.parse_unary()?;
            Ok(Expression::UnaryOp {
                op: crate::ast::UnaryOperator::Not,
                operand: Box::new(operand),
            })
        } else if self.check_token(Token::Tilde) {
            self.advance();
            let operand = self.parse_unary()?;
            Ok(Expression::UnaryOp {
                op: crate::ast::UnaryOperator::BitNot,
                operand: Box::new(operand),
            })
        } else if self.check_token(Token::Ampersand) {
            self.advance();
            let mutable = self.check_keyword(Keyword::Mut);
            if mutable {
                self.advance();
            }
            let expr = self.parse_unary()?;
            Ok(Expression::Borrow {
                mutable,
                expr: Box::new(expr),
            })
        } else {
            self.parse_call()
        }
    }
    
    fn parse_call(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;
        
        loop {
            if self.check_token(Token::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.check_token(Token::Dot) {
                self.advance();
                let field = self.expect_identifier()?;
                // Check if this is a method call (followed by '(')
                if self.check_token(Token::LeftParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();
                    
                    if !self.check_token(Token::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.check_token(Token::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }
                    
                    self.expect_token(Token::RightParen)?;
                    expr = Expression::MethodCall {
                        receiver: Box::new(expr),
                        method: field,
                        args,
                    };
                } else {
                    // Just a field access
                    expr = Expression::FieldAccess {
                        receiver: Box::new(expr),
                        field,
                    };
                }
            } else if self.check_token(Token::LeftBracket) {
                self.advance();
                let index = self.parse_expression()?;
                self.expect_token(Token::RightBracket)?;
                expr = Expression::Index {
                    receiver: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.check_token(Token::Question) {
                self.advance(); // consume '?'
                expr = Expression::Try(Box::new(expr));
            } else if self.check_keyword(Keyword::As) {
                self.advance(); // consume 'as'
                let ty = self.parse_type()?;
                expr = Expression::Cast {
                    expr: Box::new(expr),
                    ty,
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: Expression) -> Result<Expression, String> {
        self.advance(); // skip '('
        let mut args = Vec::new();
        
        if !self.check_token(Token::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.check_token(Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        self.expect_token(Token::RightParen)?;
        Ok(Expression::Call {
            callee: Box::new(callee),
            args,
            type_args: vec![],
        })
    }
    
    fn parse_primary(&mut self) -> Result<Expression, String> {
        // Guard against deep nesting — each paren level recurses through
        // parse_expression → parse_or → ... → parse_primary (10+ frames)
        self.expression_depth += 1;
        if self.expression_depth > 128 {
            self.expression_depth -= 1;
            return Err("Expression too deeply nested (limit: 128 levels). Simplify or break into sub-expressions.".to_string());
        }
        let result = self.parse_primary_inner();
        self.expression_depth -= 1;
        result
    }

    fn parse_primary_inner(&mut self) -> Result<Expression, String> {
        match self.peek() {
            Token::IntLiteral(_) => {
                if let Token::IntLiteral(n) = self.advance().clone() {
                    Ok(Expression::Literal(Literal::Int(n)))
                } else {
                    unreachable!()
                }
            }
            Token::FloatLiteral(_) => {
                if let Token::FloatLiteral(n) = self.advance().clone() {
                    Ok(Expression::Literal(Literal::Float(n)))
                } else {
                    unreachable!()
                }
            }
            Token::StringLiteral(_) => {
                if let Token::StringLiteral(s) = self.advance().clone() {
                    Ok(Expression::Literal(Literal::String(s)))
                } else {
                    unreachable!()
                }
            }
            Token::BoolLiteral(_) => {
                if let Token::BoolLiteral(b) = self.advance().clone() {
                    Ok(Expression::Literal(Literal::Bool(b)))
                } else {
                    unreachable!()
                }
            }
            Token::Identifier(_) => {
                let name = self.expect_identifier()?;
                // Check if this is an enum variant (EnumName::Variant or EnumName::Variant(value))
                if self.check_token(Token::ColonColon) {
                    self.advance(); // consume '::'
                    let method_or_variant = self.expect_identifier()?;
                    
                    // Parenthesized `Type::name(...)` is always parsed as a static call.
                    // Enum constructor disambiguation is deferred to IR lowering where we
                    // can consult the enum registry.
                    if self.check_token(Token::LeftParen) {
                        self.advance(); // consume '('
                        let mut args = Vec::new();
                        
                        if !self.check_token(Token::RightParen) {
                            loop {
                                args.push(self.parse_expression()?);
                                if !self.check_token(Token::Comma) {
                                    break;
                                }
                                self.advance();
                            }
                        }
                        
                        self.expect_token(Token::RightParen)?;

                        Ok(Expression::StaticCall {
                            type_name: name,
                            method: method_or_variant,
                            args,
                        })
                    } else {
                        // No parens = unit enum variant
                        Ok(Expression::EnumVariant {
                            enum_name: name,
                            variant: method_or_variant,
                            data: None,
                        })
                    }
                }
                // Check if this is a struct literal (Name { field: value, ... })
                // Only parse struct literals if allow_struct_literal is true
                else if self.allow_struct_literal && self.check_token(Token::LeftBrace) {
                    self.advance(); // consume '{'
                    let mut fields = Vec::new();
                    
                    while !self.check_token(Token::RightBrace) && !self.is_at_end() {
                        let field_name = self.expect_identifier()?;
                        self.expect_token(Token::Colon)?;
                        let field_value = self.parse_expression()?;
                        fields.push(FieldInit {
                            name: field_name,
                            value: field_value,
                        });
                        if self.check_token(Token::Comma) {
                            self.advance();
                        }
                    }
                    
                    self.expect_token(Token::RightBrace)?;
                    Ok(Expression::StructLiteral {
                        ty: Type::Named(name),
                        fields,
                    })
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(expr)
            }
            Token::LeftBrace => {
                Ok(Expression::Block(self.parse_block()?))
            }
            _ if self.check_keyword(Keyword::If) => {
                Ok(Expression::If(Box::new(self.parse_if_expression()?)))
            }
            _ if self.check_keyword(Keyword::Match) => {
                Ok(Expression::Match(self.parse_match()?))
            }
            _ if self.check_keyword(Keyword::Unsafe) => {
                self.advance();
                self.expect_token(Token::LeftBrace)?;
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightBrace)?;
                Ok(Expression::Unsafe(Box::new(expr)))
            }
            Token::LeftBracket => {
                self.advance(); // consume '['
                let mut elements = Vec::new();
                
                if !self.check_token(Token::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.check_token(Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                
                self.expect_token(Token::RightBracket)?;
                Ok(Expression::Array(elements))
            }
            Token::Pipe => {
                // Closure: |params| body  or |params| { block }
                self.parse_closure()
            }
            Token::Or => {
                // Empty closure: || body (|| is lexed as Or token)
                self.parse_empty_closure()
            }
            _ => Err(self.error_at_current(&format!("Unexpected token: {:?}", self.peek())))
        }
    }
    
    /// Parse a closure expression: |x| x + 1  or  |x, y: i64| { ... }
    fn parse_closure(&mut self) -> Result<Expression, String> {
        self.expect_token(Token::Pipe)?;  // Opening |
        
        // Parse parameters
        let mut params = Vec::new();
        while !self.check_token(Token::Pipe) && !self.is_at_end() {
            let name = self.expect_identifier()?;
            let ty = if self.check_token(Token::Colon) {
                self.advance();
                self.parse_type()?
            } else {
                // Default to i64 for now if no type specified
                Type::Int(IntType::I64)
            };
            params.push(Parameter { name, ty });
            
            if self.check_token(Token::Comma) {
                self.advance();
            } else if !self.check_token(Token::Pipe) {
                return Err("Expected ',' or '|' in closure parameters".to_string());
            }
        }
        
        self.expect_token(Token::Pipe)?;  // Closing |
        
        // Optional return type: |x| -> i32 { ... }
        let return_type = if self.check_token(Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Parse body: either { block } or single expression
        let body = if self.check_token(Token::LeftBrace) {
            // Block body - wrap in Block expression
            let block = self.parse_block()?;
            Expression::Block(block)
        } else {
            // Single expression body
            self.parse_expression()?
        };
        
        Ok(Expression::Closure {
            params,
            return_type,
            body: Box::new(body),
        })
    }
    
    /// Parse an empty closure: || body (where || is lexed as a single Or token)
    fn parse_empty_closure(&mut self) -> Result<Expression, String> {
        self.expect_token(Token::Or)?;  // Consume the || token
        
        // No parameters
        let params = Vec::new();
        
        // Optional return type: || -> i32 { ... }
        let return_type = if self.check_token(Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Parse body: either { block } or single expression
        let body = if self.check_token(Token::LeftBrace) {
            let block = self.parse_block()?;
            Expression::Block(block)
        } else {
            self.parse_expression()?
        };
        
        Ok(Expression::Closure {
            params,
            return_type,
            body: Box::new(body),
        })
    }
    
    fn parse_if_expression(&mut self) -> Result<IfExpression, String> {
        self.expect_keyword(Keyword::If)?;
        let condition = self.parse_expression()?;
        let then_expr = Box::new(self.parse_expression()?);
        self.expect_keyword(Keyword::Else)?;
        let else_expr = Box::new(self.parse_expression()?);
        Ok(IfExpression {
            condition: Box::new(condition),
            then_expr,
            else_expr,
        })
    }
    
    fn parse_type(&mut self) -> Result<Type, String> {
        // Pointer types: *mut T, *const T
        if self.check_token(Token::Star) {
            self.advance(); // consume '*'
            let mutable = if self.check_keyword(Keyword::Mut) {
                self.advance();
                true
            } else if self.check_keyword(Keyword::Const) {
                self.advance();
                false
            } else {
                return Err("Expected 'mut' or 'const' after '*' in pointer type".to_string());
            };
            let inner = self.parse_type()?;
            return Ok(Type::Pointer {
                mutable,
                inner: Box::new(inner),
            });
        }
        // Never type: !
        if self.check_token(Token::Exclamation) {
            self.advance();
            return Ok(Type::Never);
        }
        // Simplified type parsing
        if matches!(self.peek(), Token::Identifier(_)) {
            let name = self.expect_identifier()?;
            match name.as_str() {
                "i8" => Ok(Type::Int(IntType::I8)),
                "i16" => Ok(Type::Int(IntType::I16)),
                "i32" => Ok(Type::Int(IntType::I32)),
                "i64" => Ok(Type::Int(IntType::I64)),
                "i128" => Ok(Type::Int(IntType::I128)),
                "u8" => Ok(Type::Int(IntType::U8)),
                "u16" => Ok(Type::Int(IntType::U16)),
                "u32" => Ok(Type::Int(IntType::U32)),
                "u64" => Ok(Type::Int(IntType::U64)),
                "u128" => Ok(Type::Int(IntType::U128)),
                "isize" => Ok(Type::Int(IntType::ISize)),
                "usize" => Ok(Type::Int(IntType::USize)),
                "f32" => Ok(Type::Float(FloatType::F32)),
                "f64" => Ok(Type::Float(FloatType::F64)),
                "bool" => Ok(Type::Bool),
                "String" => Ok(Type::String),
                "str" => Ok(Type::Str),
                _ => Ok(Type::Named(name)),
            }
        } else {
            Err("Expected type".to_string())
        }
    }
    
    fn parse_struct(&mut self, is_pub: bool) -> Result<Struct, String> {
        self.expect_keyword(Keyword::Struct)?;
        let name = self.expect_identifier()?;
        
        // Parse optional generic type parameters: <T, U>
        let type_params = if self.check_token(Token::Lt) {
            self.advance(); // consume '<'
            let mut params = Vec::new();
            loop {
                params.push(self.expect_identifier()?);
                if self.check_token(Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_token(Token::Gt)?;
            params
        } else {
            Vec::new()
        };
        
        self.expect_token(Token::LeftBrace)?;
        
        let mut fields = Vec::new();
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            let field_pub = self.check_keyword(Keyword::Pub);
            if field_pub {
                self.advance();
            }
            let field_name = self.expect_identifier()?;
            self.expect_token(Token::Colon)?;
            let field_ty = self.parse_type()?;
            if self.check_token(Token::Comma) {
                self.advance();
            }
            fields.push(StructField {
                name: field_name,
                ty: field_ty,
                is_pub: field_pub,
            });
        }
        
        self.expect_token(Token::RightBrace)?;
        Ok(Struct { name, type_params, fields, is_pub })
    }
    
    fn parse_enum(&mut self, is_pub: bool) -> Result<Enum, String> {
        self.expect_keyword(Keyword::Enum)?;
        let name = self.expect_identifier()?;
        
        // Parse optional generic type parameters: <T, E>
        let type_params = if self.check_token(Token::Lt) {
            self.advance(); // consume '<'
            let mut params = Vec::new();
            loop {
                params.push(self.expect_identifier()?);
                if self.check_token(Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_token(Token::Gt)?;
            params
        } else {
            Vec::new()
        };
        
        self.expect_token(Token::LeftBrace)?;
        
        let mut variants = Vec::new();
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            let variant_name = self.expect_identifier()?;
            let data = if self.check_token(Token::LeftParen) {
                self.advance();
                let ty = self.parse_type()?;
                self.expect_token(Token::RightParen)?;
                Some(ty)
            } else {
                None
            };
            if self.check_token(Token::Comma) {
                self.advance();
            }
            variants.push(EnumVariant {
                name: variant_name,
                data,
            });
        }
        
        self.expect_token(Token::RightBrace)?;
        Ok(Enum { name, type_params, variants, is_pub })
    }
    
    /// Parse an impl block: impl TypeName { methods... } or impl Trait for TypeName { methods... }
    fn parse_impl(&mut self) -> Result<Impl, String> {
        self.expect_keyword(Keyword::Impl)?;
        let first_name = self.expect_identifier()?;
        
        // Check for `impl Trait for Type { ... }` syntax
        let (trait_name, type_name) = if self.check_keyword(Keyword::For) {
            self.advance(); // consume 'for'
            let type_name = self.expect_identifier()?;
            (Some(first_name), type_name)
        } else {
            (None, first_name)
        };
        
        self.expect_token(Token::LeftBrace)?;
        
        let mut methods = Vec::new();
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_method()?);
        }
        
        self.expect_token(Token::RightBrace)?;
        Ok(Impl { type_name, trait_name, methods })
    }
    
    /// Parse a trait definition: trait Name { func method(self) -> Type; ... }
    fn parse_trait(&mut self, is_pub: bool) -> Result<Trait, String> {
        self.expect_keyword(Keyword::Trait)?;
        let name = self.expect_identifier()?;
        self.expect_token(Token::LeftBrace)?;
        
        let mut methods = Vec::new();
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            // Parse trait method signature: func name(self, params) -> ReturnType;
            self.expect_keyword(Keyword::Func)?;
            let method_name = self.expect_identifier()?;
            self.expect_token(Token::LeftParen)?;
            
            // Parse self parameter
            let self_param = if self.check_keyword(Keyword::Mut) {
                // &mut self
                self.advance();
                if let Token::Identifier(s) = self.peek().clone() {
                    if s == "self" {
                        self.advance();
                        Some(SelfParam::RefMut)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if let Token::Identifier(s) = self.peek().clone() {
                if s == "self" {
                    self.advance();
                    Some(SelfParam::Value)
                } else {
                    None
                }
            } else {
                None
            };
            
            // Skip comma after self if present
            let mut params = Vec::new();
            if self_param.is_some() && self.check_token(Token::Comma) {
                self.advance();
            }
            
            // Parse remaining parameters
            while !self.check_token(Token::RightParen) && !self.is_at_end() {
                let param_name = self.expect_identifier()?;
                self.expect_token(Token::Colon)?;
                let param_ty = self.parse_type()?;
                params.push(Parameter { name: param_name, ty: param_ty });
                if !self.check_token(Token::Comma) {
                    break;
                }
                self.advance();
            }
            
            self.expect_token(Token::RightParen)?;
            
            let return_type = if self.check_token(Token::Arrow) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            
            self.expect_token(Token::Semicolon)?;
            
            methods.push(TraitMethod {
                name: method_name,
                self_param,
                params,
                return_type,
            });
        }
        
        self.expect_token(Token::RightBrace)?;
        Ok(Trait { name, methods, is_pub })
    }
    
    /// Parse a method definition within an impl block
    fn parse_method(&mut self) -> Result<Method, String> {
        let is_pub = self.check_keyword(Keyword::Pub);
        if is_pub {
            self.advance();
        }
        
        self.expect_keyword(Keyword::Func)?;
        let name = self.expect_identifier()?;
        self.expect_token(Token::LeftParen)?;
        
        // Check for self parameter
        let self_param = if self.check_token(Token::Ampersand) {
            // &self or &mut self
            self.advance();
            if self.check_keyword(Keyword::Mut) {
                self.advance();
                // Expect 'self' identifier
                let ident = self.expect_identifier()?;
                if ident != "self" {
                    return Err(format!("Expected 'self' after '&mut', got '{}'", ident));
                }
                Some(SelfParam::RefMut)
            } else {
                // &self
                let ident = self.expect_identifier()?;
                if ident != "self" {
                    return Err(format!("Expected 'self' after '&', got '{}'", ident));
                }
                Some(SelfParam::Ref)
            }
        } else if let Token::Identifier(ref s) = self.peek() {
            if s == "self" {
                self.advance();
                Some(SelfParam::Value)
            } else {
                None
            }
        } else {
            None
        };
        
        // If we have a self param and there are more params, expect comma
        if self_param.is_some() && !self.check_token(Token::RightParen) {
            if self.check_token(Token::Comma) {
                self.advance();
            }
        }
        
        // Parse remaining parameters (same logic as parse_parameters)
        let mut params = Vec::new();
        if !self.check_token(Token::RightParen) {
            loop {
                let param_name = self.expect_identifier()?;
                self.expect_token(Token::Colon)?;
                let param_ty = self.parse_type()?;
                params.push(Parameter { name: param_name, ty: param_ty });
                
                if !self.check_token(Token::Comma) {
                    break;
                }
                self.advance();
            }
        }
        
        self.expect_token(Token::RightParen)?;
        
        // Optional return type
        let return_type = if self.check_token(Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(Method {
            name,
            self_param,
            params,
            return_type,
            body,
            is_pub,
        })
    }
    
    fn parse_module(&mut self, is_pub: bool) -> Result<Module, String> {
        self.expect_keyword(Keyword::Mod)?;
        let name = self.expect_identifier()?;
        self.expect_token(Token::LeftBrace)?;
        
        let mut items = Vec::new();
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            items.push(self.parse_item()?);
        }
        
        self.expect_token(Token::RightBrace)?;
        Ok(Module { name, items, is_pub })
    }
    
    fn parse_import(&mut self) -> Result<Import, String> {
        if self.check_keyword(Keyword::Import) {
            self.advance();
        } else if self.check_keyword(Keyword::Use) {
            self.advance();
        }
        
        let mut path = Vec::new();
        path.push(self.expect_import_segment()?);
        
        while self.check_token(Token::ColonColon) {
            self.advance();  // consume '::'
            if self.check_token(Token::LeftBrace) {
                self.advance();  // consume '{'
                let mut selectors = Vec::new();

                if self.check_token(Token::RightBrace) {
                    return Err("Grouped import must include at least one symbol".to_string());
                }

                while !self.check_token(Token::RightBrace) && !self.is_at_end() {
                    let name = self.expect_import_segment()?;
                    let alias = if self.check_keyword(Keyword::As) {
                        self.advance();
                        Some(self.expect_identifier()?)
                    } else {
                        None
                    };
                    selectors.push(ImportSelector { name, alias });

                    if self.check_token(Token::Comma) {
                        self.advance();
                    } else if !self.check_token(Token::RightBrace) {
                        return Err("Expected ',' or '}' in grouped import list".to_string());
                    }
                }

                self.expect_token(Token::RightBrace)?;
                self.expect_token(Token::Semicolon)?;
                return Ok(Import {
                    path,
                    alias: None,
                    selectors,
                });
            }

            path.push(self.expect_import_segment()?);
        }
        
        let alias = if self.check_keyword(Keyword::As) {
            self.advance();
            Some(self.expect_identifier()?)
        } else {
            None
        };
        
        self.expect_token(Token::Semicolon)?;
        Ok(Import {
            path,
            alias,
            selectors: Vec::new(),
        })
    }
    
    fn parse_const(&mut self, is_pub: bool) -> Result<Const, String> {
        self.expect_keyword(Keyword::Const)?;
        let name = self.expect_identifier()?;
        self.expect_token(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect_token(Token::Eq)?;
        let value = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;
        Ok(Const { name, ty, value, is_pub })
    }

    fn parse_let_global(&mut self, is_pub: bool) -> Result<Const, String> {
        self.expect_keyword(Keyword::Let)?;
        // Skip optional 'mut' for top-level let (treated as const)
        if self.check_keyword(Keyword::Mut) {
            self.advance();
        }
        let name = self.expect_identifier()?;
        self.expect_token(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect_token(Token::Eq)?;
        let value = self.parse_expression()?;
        self.expect_token(Token::Semicolon)?;
        Ok(Const { name, ty, value, is_pub })
    }
    
    // Helper methods
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }
    
    fn check_token(&self, token: Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek()) == std::mem::discriminant(&token)
        }
    }
    
    fn check_keyword(&self, keyword: Keyword) -> bool {
        matches!(self.peek(), Token::Keyword(k) if *k == keyword)
    }
    
    fn expect_token(&mut self, token: Token) -> Result<(), String> {
        if self.check_token(token.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_at_current(&format!("Expected {:?}, got {:?}", token, self.peek())))
        }
    }
    
    fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), String> {
        if self.check_keyword(keyword) {
            self.advance();
            Ok(())
        } else {
            Err(self.error_at_current(&format!("Expected keyword {:?}", keyword)))
        }
    }
    
    fn expect_identifier(&mut self) -> Result<String, String> {
        if let Token::Identifier(name) = self.peek().clone() {
            self.advance();
            Ok(name)
        } else {
            Err(self.error_at_current(&format!("Expected identifier, got {:?}", self.peek())))
        }
    }

    fn expect_import_segment(&mut self) -> Result<String, String> {
        match self.peek().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            Token::Keyword(Keyword::Mod) => {
                self.advance();
                Ok("mod".to_string())
            }
            _ => Err(format!("Expected import path segment, got {:?}", self.peek())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::ast::{Expression, Item, Statement};
    use crate::lexer::Lexer;

    fn parse_single_expr_statement(expr_source: &str) -> Expression {
        let source = format!("func main() {{ {}; }}", expr_source);
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");

        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse must succeed");

        let function = match &program.items[0] {
            Item::Function(function) => function,
            other => panic!("expected function item, got {:?}", other),
        };

        match &function.body.statements[0] {
            Statement::Expr(expr) => expr.clone(),
            other => panic!("expected expression statement, got {:?}", other),
        }
    }

    #[test]
    fn single_arg_type_coloncolon_call_parses_as_static_call() {
        let expr = parse_single_expr_statement("Point::sum(1)");

        match expr {
            Expression::StaticCall {
                type_name,
                method,
                args,
            } => {
                assert_eq!(type_name, "Point");
                assert_eq!(method, "sum");
                assert_eq!(args.len(), 1);
            }
            other => panic!("expected StaticCall, got {:?}", other),
        }
    }

    #[test]
    fn non_parenthesized_type_coloncolon_stays_enum_variant() {
        let expr = parse_single_expr_statement("Status::Ok");

        match expr {
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => {
                assert_eq!(enum_name, "Status");
                assert_eq!(variant, "Ok");
                assert!(data.is_none());
            }
            other => panic!("expected EnumVariant, got {:?}", other),
        }
    }

    #[test]
    fn parses_extern_function_declaration_item() {
        let source = r#"
extern func falcon_println(s: str);
func main() { falcon_println("hi"); }
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse must succeed");

        match &program.items[0] {
            Item::ExternFunction(extern_fn) => {
                assert_eq!(extern_fn.name, "falcon_println");
                assert_eq!(extern_fn.params.len(), 1);
                assert!(extern_fn.return_type.is_none());
            }
            other => panic!("expected ExternFunction item, got {:?}", other),
        }
    }

    #[test]
    fn parses_grouped_import_with_multiple_symbols() {
        let source = r#"
import std::net::{Server, Response};
func main() {}
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse must succeed");

        match &program.items[0] {
            Item::Import(import) => {
                assert_eq!(import.path, vec!["std".to_string(), "net".to_string()]);
                assert_eq!(import.selectors.len(), 2);
                assert_eq!(import.selectors[0].name, "Server");
                assert_eq!(import.selectors[1].name, "Response");
                assert!(import.alias.is_none());
            }
            other => panic!("expected Import item, got {:?}", other),
        }
    }

    #[test]
    fn parses_grouped_import_with_aliases() {
        let source = r#"
import math::{sin as sine, cos};
func main() {}
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse must succeed");

        match &program.items[0] {
            Item::Import(import) => {
                assert_eq!(import.path, vec!["math".to_string()]);
                assert_eq!(import.selectors.len(), 2);
                assert_eq!(import.selectors[0].name, "sin");
                assert_eq!(import.selectors[0].alias.as_deref(), Some("sine"));
                assert_eq!(import.selectors[1].name, "cos");
                assert!(import.selectors[1].alias.is_none());
            }
            other => panic!("expected Import item, got {:?}", other),
        }
    }

    #[test]
    fn parses_import_path_with_mod_keyword_segment() {
        let source = r#"
import random::mod;
func main() {}
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse must succeed");

        match &program.items[0] {
            Item::Import(import) => {
                assert_eq!(import.path, vec!["random".to_string(), "mod".to_string()]);
                assert!(import.selectors.is_empty());
            }
            other => panic!("expected Import item, got {:?}", other),
        }
    }

    #[test]
    fn parses_match_expression_inside_let_initializer() {
        let source = r#"
func main() {
    let x = match 1 {
        1 => 2,
        _ => 3,
    };
}
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse must succeed");

        let function = match &program.items[0] {
            Item::Function(function) => function,
            other => panic!("expected Function item, got {:?}", other),
        };

        match &function.body.statements[0] {
            Statement::Let(let_stmt) => match &let_stmt.value {
                Expression::Match(match_expr) => {
                    assert_eq!(match_expr.arms.len(), 2);
                }
                other => panic!("expected match expression in let initializer, got {:?}", other),
            },
            other => panic!("expected let statement, got {:?}", other),
        }
    }

    #[test]
    fn parses_enum_variant_pattern_in_match_arm() {
        let source = r#"
enum Status { Ok, Err(i64) }
func main() {
    let s = Status::Err(7);
    match s {
        Status::Ok => print_int(0),
        Status::Err(code) => print_int(code),
    }
}
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse must succeed");

        let function = match &program.items[1] {
            Item::Function(function) => function,
            other => panic!("expected Function item, got {:?}", other),
        };

        match &function.body.statements[1] {
            Statement::Match(match_stmt) => {
                assert_eq!(match_stmt.arms.len(), 2);
                match &match_stmt.arms[1].pattern {
                    crate::ast::Pattern::EnumVariant { ty, variant, data } => {
                        assert_eq!(variant, "Err");
                        assert_eq!(*ty, crate::ast::Type::Named("Status".to_string()));
                        assert!(matches!(
                            data.as_deref(),
                            Some(crate::ast::Pattern::Variable(name)) if name == "code"
                        ));
                    }
                    other => panic!("expected enum variant pattern, got {:?}", other),
                }
            }
            other => panic!("expected match statement, got {:?}", other),
        }
    }

    #[test]
    fn parses_unqualified_enum_variant_pattern_with_payload() {
        let source = r#"
func main() {
    match result {
        Ok(value) => print_int(value),
        Err(_) => print_int(0),
    }
}
"#;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().expect("tokenization must succeed");
        let mut parser = Parser::new(tokens);
        let program = parser.parse().expect("parse must succeed");

        let function = match &program.items[0] {
            Item::Function(function) => function,
            other => panic!("expected Function item, got {:?}", other),
        };

        match &function.body.statements[0] {
            Statement::Match(match_stmt) => {
                match &match_stmt.arms[0].pattern {
                    crate::ast::Pattern::EnumVariant { ty, variant, data } => {
                        assert_eq!(variant, "Ok");
                        assert_eq!(*ty, crate::ast::Type::Named(String::new()));
                        assert!(matches!(
                            data.as_deref(),
                            Some(crate::ast::Pattern::Variable(name)) if name == "value"
                        ));
                    }
                    other => panic!("expected enum variant pattern, got {:?}", other),
                }
            }
            other => panic!("expected match statement, got {:?}", other),
        }
    }
}

