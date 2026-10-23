use crate::ast::{BinaryOp, Expr, ExprKind, Literal, Stmt, StmtKind, UnaryOp};
use crate::diagnostic::{Diagnostic, Phase, Span};
use crate::token::{Token, TokenKind};

const MAX_SYNTAX_DEPTH: usize = 128;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, Diagnostic> {
    Parser {
        tokens,
        current: 0,
        block_depth: 0,
        expression_depth: 0,
        unary_depth: 0,
    }
    .program()
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    block_depth: usize,
    expression_depth: usize,
    unary_depth: usize,
}

impl Parser {
    fn program(mut self) -> Result<Vec<Stmt>, Diagnostic> {
        let mut statements = Vec::new();
        self.skip_newlines();
        while !self.check(&TokenKind::Eof) {
            statements.push(self.statement()?);
            self.finish_statement()?;
            self.skip_newlines();
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, Diagnostic> {
        let span = self.peek().span;
        let kind = match self.peek().kind.clone() {
            TokenKind::Let | TokenKind::Const => {
                let mutable = self.advance().kind == TokenKind::Let;
                let name = self.consume_identifier("اسم المتغير")?;
                self.consume(&TokenKind::Equal, "علامة '=' بعد اسم المتغير")?;
                let value = self.expression()?;
                StmtKind::Declare {
                    name,
                    mutable,
                    value,
                }
            }
            TokenKind::Identifier(name) => {
                self.advance();
                self.consume(&TokenKind::Equal, "علامة '=' في الإسناد")?;
                let value = self.expression()?;
                StmtKind::Assign { name, value }
            }
            TokenKind::Print => {
                self.advance();
                self.consume(&TokenKind::LeftParen, "قوس فتح بعد «اطبع»")?;
                let value = self.expression()?;
                self.consume(&TokenKind::RightParen, "قوس إغلاق بعد قيمة الطباعة")?;
                StmtKind::Print(value)
            }
            TokenKind::If => self.if_statement()?,
            TokenKind::While => self.while_statement()?,
            TokenKind::End | TokenKind::Else => {
                return Err(self.error("نهاية كتلة دون بداية مطابقة"));
            }
            _ => return Err(self.error("تعليمة غير معروفة")),
        };
        Ok(Stmt { kind, span })
    }

    fn if_statement(&mut self) -> Result<StmtKind, Diagnostic> {
        self.advance();
        let condition = self.expression()?;
        self.consume(&TokenKind::Newline, "سطر جديد بعد شرط «إذا»")?;
        let then_branch = self.block(true)?;
        let else_branch = if self.match_kind(&TokenKind::Else) {
            self.consume(&TokenKind::Newline, "سطر جديد بعد «وإلا»")?;
            self.block(false)?
        } else {
            Vec::new()
        };
        self.consume(&TokenKind::End, "«نهاية» لإغلاق كتلة «إذا»")?;
        Ok(StmtKind::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<StmtKind, Diagnostic> {
        self.advance();
        let condition = self.expression()?;
        self.consume(&TokenKind::Newline, "سطر جديد بعد شرط «طالما»")?;
        let body = self.block(false)?;
        self.consume(&TokenKind::End, "«نهاية» لإغلاق كتلة «طالما»")?;
        Ok(StmtKind::While { condition, body })
    }

    fn block(&mut self, stop_on_else: bool) -> Result<Vec<Stmt>, Diagnostic> {
        if self.block_depth >= MAX_SYNTAX_DEPTH {
            return Err(self.error(format!(
                "تجاوز تداخل الكتل الحد المسموح ({MAX_SYNTAX_DEPTH})"
            )));
        }
        self.block_depth += 1;
        let result = self.block_inner(stop_on_else);
        self.block_depth -= 1;
        result
    }

    fn block_inner(&mut self, stop_on_else: bool) -> Result<Vec<Stmt>, Diagnostic> {
        let mut statements = Vec::new();
        self.skip_newlines();
        loop {
            if self.check(&TokenKind::Eof)
                || self.check(&TokenKind::End)
                || (stop_on_else && self.check(&TokenKind::Else))
            {
                return Ok(statements);
            }
            statements.push(self.statement()?);
            self.finish_statement()?;
            self.skip_newlines();
        }
    }

    fn expression(&mut self) -> Result<Expr, Diagnostic> {
        if self.expression_depth >= MAX_SYNTAX_DEPTH {
            return Err(self.error(format!(
                "تجاوز تداخل التعبيرات الحد المسموح ({MAX_SYNTAX_DEPTH})"
            )));
        }
        self.expression_depth += 1;
        let result = self.or();
        self.expression_depth -= 1;
        result
    }

    fn or(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.and()?;
        while self.match_kind(&TokenKind::Or) {
            let span = expr.span;
            let right = self.and()?;
            expr = binary(expr, BinaryOp::Or, right, span);
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.equality()?;
        while self.match_kind(&TokenKind::And) {
            let span = expr.span;
            let right = self.equality()?;
            expr = binary(expr, BinaryOp::And, right, span);
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.comparison()?;
        loop {
            let operator = if self.match_kind(&TokenKind::EqualEqual) {
                Some(BinaryOp::Equal)
            } else if self.match_kind(&TokenKind::BangEqual) {
                Some(BinaryOp::NotEqual)
            } else {
                None
            };
            let Some(operator) = operator else { break };
            let span = expr.span;
            let right = self.comparison()?;
            expr = binary(expr, operator, right, span);
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.term()?;
        loop {
            let operator = if self.match_kind(&TokenKind::Less) {
                Some(BinaryOp::Less)
            } else if self.match_kind(&TokenKind::LessEqual) {
                Some(BinaryOp::LessEqual)
            } else if self.match_kind(&TokenKind::Greater) {
                Some(BinaryOp::Greater)
            } else if self.match_kind(&TokenKind::GreaterEqual) {
                Some(BinaryOp::GreaterEqual)
            } else {
                None
            };
            let Some(operator) = operator else { break };
            let span = expr.span;
            let right = self.term()?;
            expr = binary(expr, operator, right, span);
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.factor()?;
        loop {
            let operator = if self.match_kind(&TokenKind::Plus) {
                Some(BinaryOp::Add)
            } else if self.match_kind(&TokenKind::Minus) {
                Some(BinaryOp::Subtract)
            } else {
                None
            };
            let Some(operator) = operator else { break };
            let span = expr.span;
            let right = self.factor()?;
            expr = binary(expr, operator, right, span);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.unary()?;
        loop {
            let operator = if self.match_kind(&TokenKind::Star) {
                Some(BinaryOp::Multiply)
            } else if self.match_kind(&TokenKind::Slash) {
                Some(BinaryOp::Divide)
            } else if self.match_kind(&TokenKind::Percent) {
                Some(BinaryOp::Remainder)
            } else {
                None
            };
            let Some(operator) = operator else { break };
            let span = expr.span;
            let right = self.unary()?;
            expr = binary(expr, operator, right, span);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Diagnostic> {
        let operator = if self.match_kind(&TokenKind::Minus) {
            Some((UnaryOp::Negate, self.previous().span))
        } else if self.match_kind(&TokenKind::Not) {
            Some((UnaryOp::Not, self.previous().span))
        } else {
            None
        };

        let Some((operator, span)) = operator else {
            return self.primary();
        };

        if self.unary_depth >= MAX_SYNTAX_DEPTH {
            return Err(Diagnostic::new(
                Phase::Syntax,
                span,
                format!("تجاوز تداخل العوامل الأحادية الحد المسموح ({MAX_SYNTAX_DEPTH})"),
            ));
        }
        self.unary_depth += 1;
        let right = self.unary();
        self.unary_depth -= 1;
        let right = right?;

        Ok(Expr {
            kind: ExprKind::Unary {
                operator,
                right: Box::new(right),
            },
            span,
        })
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        if self.check(&TokenKind::Eof) {
            return Err(self.error("انتهى الملف بينما كان التعبير ينتظر قيمة"));
        }
        let token = self.advance().clone();
        let kind = match token.kind {
            TokenKind::Integer(value) => ExprKind::Literal(Literal::Integer(value)),
            TokenKind::String(value) => ExprKind::Literal(Literal::Text(value)),
            TokenKind::True => ExprKind::Literal(Literal::Boolean(true)),
            TokenKind::False => ExprKind::Literal(Literal::Boolean(false)),
            TokenKind::Identifier(name) => ExprKind::Variable(name),
            TokenKind::LeftParen => {
                let expr = self.expression()?;
                self.consume(&TokenKind::RightParen, "قوس إغلاق ')'")?;
                return Ok(expr);
            }
            _ => {
                return Err(Diagnostic::new(
                    Phase::Syntax,
                    token.span,
                    format!("تعبير غير متوقع؛ وصل {} بدل قيمة", token.kind.description()),
                ));
            }
        };
        Ok(Expr {
            kind,
            span: token.span,
        })
    }

    fn consume_identifier(&mut self, expected: &str) -> Result<String, Diagnostic> {
        match self.peek().kind.clone() {
            TokenKind::Identifier(name) => {
                self.advance();
                Ok(name)
            }
            _ => Err(self.error(format!("كان متوقعًا {expected}"))),
        }
    }

    fn consume(&mut self, expected: &TokenKind, description: &str) -> Result<Token, Diagnostic> {
        if self.check(expected) {
            Ok(self.advance().clone())
        } else {
            Err(self.error(format!("كان متوقعًا {description}")))
        }
    }

    fn match_kind(&mut self, expected: &TokenKind) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(expected)
    }

    fn finish_statement(&mut self) -> Result<(), Diagnostic> {
        if self.check(&TokenKind::Newline)
            || self.check(&TokenKind::Eof)
            || self.check(&TokenKind::End)
            || self.check(&TokenKind::Else)
        {
            Ok(())
        } else {
            Err(self.error("يجب أن تنتهي التعليمة بنهاية السطر"))
        }
    }

    fn skip_newlines(&mut self) {
        while self.match_kind(&TokenKind::Newline) {}
    }

    fn advance(&mut self) -> &Token {
        if !self.check(&TokenKind::Eof) {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current.min(self.tokens.len() - 1)]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }

    fn error(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Phase::Syntax, self.peek().span, message)
    }
}

fn binary(left: Expr, operator: BinaryOp, right: Expr, span: Span) -> Expr {
    Expr {
        kind: ExprKind::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        },
        span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    #[test]
    fn parses_nested_blocks_and_operator_precedence() {
        let tokens = lex("دع س = 2 + 3 * 4\nإذا س > 10\nاطبع(س)\nنهاية").unwrap();
        assert!(parse(tokens).is_ok());
    }

    #[test]
    fn rejects_unclosed_parentheses_without_recursing_forever() {
        let tokens = lex("اطبع(").unwrap();
        assert!(parse(tokens).is_err());
    }

    #[test]
    fn rejects_excessively_nested_parentheses() {
        let depth = MAX_SYNTAX_DEPTH + 1;
        let source = format!("اطبع({}1{})", "(".repeat(depth), ")".repeat(depth));
        let tokens = lex(&source).unwrap();
        let error = parse(tokens).unwrap_err();
        assert!(error.message.contains("تجاوز تداخل التعبيرات"));
    }

    #[test]
    fn rejects_excessively_nested_unary_operators() {
        let source = format!("اطبع({}صح)", "ليس ".repeat(MAX_SYNTAX_DEPTH + 1));
        let tokens = lex(&source).unwrap();
        let error = parse(tokens).unwrap_err();
        assert!(error.message.contains("تجاوز تداخل العوامل الأحادية"));
    }

    #[test]
    fn rejects_unclosed_blocks() {
        let tokens = lex("إذا صح\nاطبع(1)").unwrap();
        assert!(parse(tokens).is_err());
    }

    #[test]
    fn rejects_assignment_without_equals() {
        let tokens = lex("دع س ١").unwrap();
        assert!(parse(tokens).is_err());
    }
}
