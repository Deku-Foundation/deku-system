use std::collections::BTreeMap;

use crate::ast::{BinaryOp, Expr, ExprKind, Literal, Stmt, StmtKind, UnaryOp};
use crate::diagnostic::{Diagnostic, Phase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Integer,
    Text,
    Boolean,
}

impl Type {
    fn arabic(self) -> &'static str {
        match self {
            Self::Integer => "عدد صحيح",
            Self::Text => "نص",
            Self::Boolean => "قيمة منطقية",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Binding {
    value_type: Type,
    mutable: bool,
}

pub fn check(statements: &[Stmt]) -> Result<(), Diagnostic> {
    let mut checker = Checker {
        scopes: vec![BTreeMap::new()],
    };
    checker.check_block(statements)
}

struct Checker {
    scopes: Vec<BTreeMap<String, Binding>>,
}

impl Checker {
    fn check_block(&mut self, statements: &[Stmt]) -> Result<(), Diagnostic> {
        for statement in statements {
            self.check_statement(statement)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &Stmt) -> Result<(), Diagnostic> {
        match &statement.kind {
            StmtKind::Declare {
                name,
                mutable,
                value,
            } => {
                let value_type = self.expression_type(value)?;
                let scope = self.scopes.last_mut().expect("global scope exists");
                if scope.contains_key(name) {
                    return Err(type_error(
                        statement.span,
                        format!("المعرّف «{name}» معلن مسبقًا في هذا النطاق"),
                    ));
                }
                scope.insert(
                    name.clone(),
                    Binding {
                        value_type,
                        mutable: *mutable,
                    },
                );
                Ok(())
            }
            StmtKind::Assign { name, value } => {
                let value_type = self.expression_type(value)?;
                let binding = self
                    .scopes
                    .iter()
                    .rev()
                    .find_map(|scope| scope.get(name))
                    .copied()
                    .ok_or_else(|| {
                        type_error(statement.span, format!("المتغير «{name}» غير معرّف"))
                    })?;
                if !binding.mutable {
                    return Err(type_error(
                        statement.span,
                        format!("لا يمكن تغيير الثابت «{name}»"),
                    ));
                }
                if value_type != binding.value_type {
                    return Err(type_error(
                        statement.span,
                        format!(
                            "لا يمكن إسناد قيمة من نوع {} إلى «{name}» من نوع {}",
                            value_type.arabic(),
                            binding.value_type.arabic()
                        ),
                    ));
                }
                Ok(())
            }
            StmtKind::Print(value) => {
                self.expression_type(value)?;
                Ok(())
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.require_type(condition, Type::Boolean, "شرط «إذا»")?;
                self.with_scope(then_branch)?;
                self.with_scope(else_branch)
            }
            StmtKind::While { condition, body } => {
                self.require_type(condition, Type::Boolean, "شرط «طالما»")?;
                self.with_scope(body)
            }
        }
    }

    fn with_scope(&mut self, statements: &[Stmt]) -> Result<(), Diagnostic> {
        self.scopes.push(BTreeMap::new());
        let result = self.check_block(statements);
        self.scopes.pop();
        result
    }

    fn require_type(
        &self,
        expression: &Expr,
        expected: Type,
        context: &str,
    ) -> Result<(), Diagnostic> {
        let actual = self.expression_type(expression)?;
        if actual == expected {
            Ok(())
        } else {
            Err(type_error(
                expression.span,
                format!(
                    "{context} يتطلب {}، لكن التعبير من نوع {}",
                    expected.arabic(),
                    actual.arabic()
                ),
            ))
        }
    }

    fn expression_type(&self, expression: &Expr) -> Result<Type, Diagnostic> {
        match &expression.kind {
            ExprKind::Literal(literal) => Ok(match literal {
                Literal::Integer(_) => Type::Integer,
                Literal::Text(_) => Type::Text,
                Literal::Boolean(_) => Type::Boolean,
            }),
            ExprKind::Variable(name) => self
                .scopes
                .iter()
                .rev()
                .find_map(|scope| scope.get(name))
                .map(|binding| binding.value_type)
                .ok_or_else(|| type_error(expression.span, format!("المتغير «{name}» غير معرّف")),
            ExprKind::Unary { operator, right } => {
                let actual = self.expression_type(right)?;
                let (expected, result) = match operator {
                    UnaryOp::Negate => (Type::Integer, Type::Integer),
                    UnaryOp::Not => (Type::Boolean, Type::Boolean),
                };
                if actual != expected {
                    let label = match operator {
                        UnaryOp::Negate => "النفي العددي",
                        UnaryOp::Not => "العامل «ليس»",
                    };
                    return Err(type_error(
                        expression.span,
                        format!(
                            "{label} يتطلب {}، لكن القيمة من نوع {}",
                            expected.arabic(),
                            actual.arabic()
                        ),
                    ));
                }
                Ok(result)
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = self.expression_type(left)?;
                let right_type = self.expression_type(right)?;
                use BinaryOp as Op;
                match operator {
                    Op::Add if left_type == Type::Integer && right_type == Type::Integer => {
                        Ok(Type::Integer)
                    }
                    Op::Add if left_type == Type::Text && right_type == Type::Text => {
                        Ok(Type::Text)
                    }
                    Op::Subtract | Op::Multiply | Op::Divide | Op::Remainder
                        if left_type == Type::Integer && right_type == Type::Integer =>
                    {
                        Ok(Type::Integer)
                    }
                    Op::Less | Op::LessEqual | Op::Greater | Op::GreaterEqual
                        if left_type == Type::Integer && right_type == Type::Integer =>
                    {
                        Ok(Type::Boolean)
                    }
                    Op::Equal | Op::NotEqual if left_type == right_type => Ok(Type::Boolean),
                    Op::And | Op::Or
                        if left_type == Type::Boolean && right_type == Type::Boolean =>
                    {
                        Ok(Type::Boolean)
                    }
                    _ => Err(type_error(
                        expression.span,
                        format!(
                            "العامل «{}» لا يقبل نوعي {} و{}",
                            operator_description(*operator),
                            left_type.arabic(),
                            right_type.arabic()
                        ),
                    )),
                }
            }
        }
    }
}

fn operator_description(operator: BinaryOp) -> &'static str {
    match operator {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Remainder => "%",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::Greater => ">",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::And => "و",
        BinaryOp::Or => "أو",
    }
}

fn type_error(span: crate::diagnostic::Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(Phase::Semantic, span, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn check_source(source: &str) -> Result<(), Diagnostic> {
        check(&parse(lex(source)?)?)
    }

    #[test]
    fn infers_declaration_types_and_checks_operators() {
        assert!(check_source("دع العدد = 3\nاطبع(العدد + 2)").is_ok());
        assert!(check_source("دع العدد = 3\nاطبع(العدد + \"نص\")").is_err());
    }

    #[test]
    fn conditions_must_be_boolean() {
        assert!(check_source("إذا 5\nاطبع(1)\nهاية").is_err());
        assert!(check_source("إذا 5 > 3\nاطبع(1)\nهاية").is_ok());
    }

    #[test]
    fn mutable_variables_keep_their_inferred_type() {
        assert!(check_source("دع القيمة = 3\nالقيمة = 4").is_ok());
        assert!(check_source("دع القيمة = 3\nالقيمة = \"أربعة\"").is_err());
    }

    #[test]
    fn immutable_bindings_and_undefined_names_fail_during_check() {
        assert!(check_source("ثابت الحد = 5\nالحد = 6").is_err());
        assert!(check_source("اطبع(اسم_غير_موجود)").is_err());
    }

    #[test]
    fn block_bindings_do_not_escape_scope() {
        let source = "إذا صح\nدع محلي = 1\nنهاية\nاطبع(محلي)";
        assert!(check_source(source).is_err());
    }
}
