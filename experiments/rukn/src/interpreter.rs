use std::collections::BTreeMap;

use crate::ast::{BinaryOp, Expr, ExprKind, Literal, Stmt, StmtKind, UnaryOp};
use crate::diagnostic::{Diagnostic, Phase, Span};

pub const DEFAULT_STEP_LIMIT: u64 = 1_000_000;
const MAX_TEXT_BYTES: usize = 256 * 1024;
const MAX_OUTPUT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Integer(i64),
    Text(String),
    Boolean(bool),
}

impl Value {
    fn display(&self) -> String {
        match self {
            Self::Integer(value) => value.to_string(),
            Self::Text(value) => value.clone(),
            Self::Boolean(true) => "صح".to_string(),
            Self::Boolean(false) => "خطأ".to_string(),
        }
    }

    fn into_boolean(self, span: Span) -> Result<bool, Diagnostic> {
        match self {
            Self::Boolean(value) => Ok(value),
            _ => Err(runtime_error(
                span,
                "يتطلب الشرط قيمة منطقية: «صح» أو «خطأ»",
            )),
        }
    }
}

#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    mutable: bool,
}

pub fn execute_with_step_limit(
    statements: &[Stmt],
    step_limit: u64,
) -> Result<Vec<String>, Diagnostic> {
    let mut interpreter = Interpreter {
        scopes: vec![BTreeMap::new()],
        output: Vec::new(),
        output_bytes: 0,
        steps: 0,
        step_limit,
    };
    interpreter.execute_block(statements)?;
    Ok(interpreter.output)
}

struct Interpreter {
    scopes: Vec<BTreeMap<String, Binding>>,
    output: Vec<String>,
    output_bytes: usize,
    steps: u64,
    step_limit: u64,
}

impl Interpreter {
    fn execute_block(&mut self, statements: &[Stmt]) -> Result<(), Diagnostic> {
        for statement in statements {
            self.consume_step(statement.span)?;
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Stmt) -> Result<(), Diagnostic> {
        match &statement.kind {
            StmtKind::Declare {
                name,
                mutable,
                value,
            } => {
                let evaluated = self.evaluate(value)?;
                let scope = self
                    .scopes
                    .last_mut()
                    .expect("a global scope always exists");
                if scope.contains_key(name) {
                    return Err(runtime_error(
                        statement.span,
                        format!("المعرّف «{name}» معلن مسبقًا في هذا النطاق"),
                    ));
                }
                scope.insert(
                    name.clone(),
                    Binding {
                        value: evaluated,
                        mutable: *mutable,
                    },
                );
                Ok(())
            }
            StmtKind::Assign { name, value } => {
                let evaluated = self.evaluate(value)?;
                for scope in self.scopes.iter_mut().rev() {
                    if let Some(binding) = scope.get_mut(name) {
                        if !binding.mutable {
                            return Err(runtime_error(
                                statement.span,
                                format!("لا يمكن تغيير الثابت «{name}»"),
                            ));
                        }
                        binding.value = evaluated;
                        return Ok(());
                    }
                }
                Err(runtime_error(
                    statement.span,
                    format!("المتغير «{name}» غير معرّف"),
                ))
            }
            StmtKind::Print(value) => self.print_value(value, statement.span),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let result = self.evaluate(condition)?.into_boolean(condition.span)?;
                self.with_scope(if result { then_branch } else { else_branch })
            }
            StmtKind::While { condition, body } => {
                loop {
                    self.consume_step(statement.span)?;
                    let result = self.evaluate(condition)?.into_boolean(condition.span)?;
                    if !result {
                        break;
                    }
                    self.with_scope(body)?;
                }
                Ok(())
            }
        }
    }

    fn with_scope(&mut self, statements: &[Stmt]) -> Result<(), Diagnostic> {
        self.scopes.push(BTreeMap::new());
        let result = self.execute_block(statements);
        self.scopes.pop();
        result
    }

    fn evaluate(&self, expression: &Expr) -> Result<Value, Diagnostic> {
        match &expression.kind {
            ExprKind::Literal(literal) => Ok(match literal {
                Literal::Integer(value) => Value::Integer(*value),
                Literal::Text(value) => Value::Text(value.clone()),
                Literal::Boolean(value) => Value::Boolean(*value),
            }),
            ExprKind::Variable(name) => self
                .scopes
                .iter()
                .rev()
                .find_map(|scope| scope.get(name))
                .map(|binding| binding.value.clone())
                .ok_or_else(|| {
                    runtime_error(expression.span, format!("المتغير «{name}» غير معرّف"))
                }),
            ExprKind::Unary { operator, right } => {
                let value = self.evaluate(right)?;
                match (operator, value) {
                    (UnaryOp::Negate, Value::Integer(number)) => number
                        .checked_neg()
                        .map(Value::Integer)
                        .ok_or_else(|| runtime_error(expression.span, "تجاوز عددي عند النفي")),
                    (UnaryOp::Negate, _) => Err(runtime_error(
                        expression.span,
                        "العامل «-» يتطلب عددًا صحيحًا",
                    )),
                    (UnaryOp::Not, Value::Boolean(value)) => Ok(Value::Boolean(!value)),
                    (UnaryOp::Not, _) => Err(runtime_error(
                        expression.span,
                        "العامل «ليس» يتطلب قيمة منطقية",
                    )),
                }
            }
            ExprKind::Binary {
                left,
                operator: BinaryOp::And,
                right,
            } => {
                let left = self.evaluate(left)?.into_boolean(expression.span)?;
                if !left {
                    return Ok(Value::Boolean(false));
                }
                Ok(Value::Boolean(
                    self.evaluate(right)?.into_boolean(expression.span)?,
                ))
            }
            ExprKind::Binary {
                left,
                operator: BinaryOp::Or,
                right,
            } => {
                let left = self.evaluate(left)?.into_boolean(expression.span)?;
                if left {
                    return Ok(Value::Boolean(true));
                }
                Ok(Value::Boolean(
                    self.evaluate(right)?.into_boolean(expression.span)?,
                ))
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_value = self.evaluate(left)?;
                let right_value = self.evaluate(right)?;
                self.evaluate_binary(left_value, *operator, right_value, expression.span)
            }
        }
    }

    fn evaluate_binary(
        &self,
        left: Value,
        operator: BinaryOp,
        right: Value,
        span: Span,
    ) -> Result<Value, Diagnostic> {
        use BinaryOp as Op;
        use Value as Val;

        match (left, operator, right) {
            (Val::Integer(a), Op::Add, Val::Integer(b)) => {
                checked_integer(a.checked_add(b), span, "الجمع")
            }
            (Val::Integer(a), Op::Subtract, Val::Integer(b)) => {
                checked_integer(a.checked_sub(b), span, "الطرح")
            }
            (Val::Integer(a), Op::Multiply, Val::Integer(b)) => {
                checked_integer(a.checked_mul(b), span, "الضرب")
            }
            (Val::Integer(_), Op::Divide | Op::Remainder, Val::Integer(0)) => {
                Err(runtime_error(span, "القسمة على صفر غير مسموحة"))
            }
            (Val::Integer(a), Op::Divide, Val::Integer(b)) => {
                checked_integer(a.checked_div(b), span, "القسمة")
            }
            (Val::Integer(a), Op::Remainder, Val::Integer(b)) => {
                checked_integer(a.checked_rem(b), span, "باقي القسمة")
            }
            (Val::Text(a), Op::Add, Val::Text(b)) => {
                let length = a.len().saturating_add(b.len());
                if length > MAX_TEXT_BYTES {
                    Err(runtime_error(
                        span,
                        format!("تجاوز النص حد الحجم البالغ {MAX_TEXT_BYTES} بايت"),
                    ))
                } else {
                    Ok(Val::Text(a + &b))
                }
            }
            (Val::Integer(a), Op::Equal, Val::Integer(b)) => Ok(Val::Boolean(a == b)),
            (Val::Integer(a), Op::NotEqual, Val::Integer(b)) => Ok(Val::Boolean(a != b)),
            (Val::Boolean(a), Op::Equal, Val::Boolean(b)) => Ok(Val::Boolean(a == b)),
            (Val::Boolean(a), Op::NotEqual, Val::Boolean(b)) => Ok(Val::Boolean(a != b)),
            (Val::Text(a), Op::Equal, Val::Text(b)) => Ok(Val::Boolean(a == b)),
            (Val::Text(a), Op::NotEqual, Val::Text(b)) => Ok(Val::Boolean(a != b)),
            (Val::Integer(a), Op::Less, Val::Integer(b)) => Ok(Val::Boolean(a < b)),
            (Val::Integer(a), Op::LessEqual, Val::Integer(b)) => Ok(Val::Boolean(a <= b)),
            (Val::Integer(a), Op::Greater, Val::Integer(b)) => Ok(Val::Boolean(a > b)),
            (Val::Integer(a), Op::GreaterEqual, Val::Integer(b)) => Ok(Val::Boolean(a >= b)),
            (_, Op::Equal, _) => Ok(Val::Boolean(false)),
            (_, Op::NotEqual, _) => Ok(Val::Boolean(true)),
            _ => Err(runtime_error(
                span,
                format!("أنواع القيم غير متوافقة مع العامل {operator:?}"),
            )),
        }
    }

    fn print_value(&mut self, expression: &Expr, span: Span) -> Result<(), Diagnostic> {
        let rendered = self.evaluate(expression)?.display();
        let next_size = self
            .output_bytes
            .checked_add(rendered.len())
            .and_then(|size| size.checked_add(1))
            .unwrap_or(usize::MAX);
        if next_size > MAX_OUTPUT_BYTES {
            return Err(runtime_error(
                span,
                format!("تجاوز إخراج البرنامج الحد البالغ {MAX_OUTPUT_BYTES} بايت"),
            ));
        }
        self.output_bytes = next_size;
        self.output.push(rendered);
        Ok(())
    }

    fn consume_step(&mut self, span: Span) -> Result<(), Diagnostic> {
        if self.steps >= self.step_limit {
            return Err(runtime_error(
                span,
                format!("تجاوز البرنامج حد التنفيذ البالغ {} خطوة", self.step_limit),
            ));
        }
        self.steps += 1;
        Ok(())
    }
}

fn checked_integer(value: Option<i64>, span: Span, operation: &str) -> Result<Value, Diagnostic> {
    value
        .map(Value::Integer)
        .ok_or_else(|| runtime_error(span, format!("تجاوز عددي أثناء عملية {operation}")))
}

fn runtime_error(span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(Phase::Runtime, span, message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn run(source: &str) -> Result<Vec<String>, Diagnostic> {
        execute_with_step_limit(&parse(lex(source)?)?, DEFAULT_STEP_LIMIT)
    }

    #[test]
    fn runs_arabic_variables_and_arithmetic() {
        let output = run("دع العدد = 6\nاطبع(العدد * 7)").unwrap();
        assert_eq!(output, vec!["42"]);
    }

    #[test]
    fn runs_if_else_and_while_blocks() {
        let source = "دع س = 0\nطالما س < 3\nاطبع(س)\nس = س + 1\nهاية\nإذا س == 3\nاطبع(\"انتهى\")\nوإلا\nاطبع(\"خطأ\")\nهاية";
        assert_eq!(run(source).unwrap(), vec!["0", "1", "2", "انتهى"]);
    }

    #[test]
    fn constants_cannot_be_reassigned() {
        let error = run("ثابت حد = 5\nحد = 6").unwrap_err();
        assert!(error.message.contains("الثابت"));
    }

    #[test]
    fn undefined_variables_are_reported() {
        let error = run("اطبع(مفقود)").unwrap_err();
        assert!(error.message.contains("غير معرّف"));
    }

    #[test]
    fn division_by_zero_is_reported() {
        let error = run("اطبع(10 / 0)").unwrap_err();
        assert!(error.message.contains("صفر"));
    }

    #[test]
    fn boolean_operators_short_circuit() {
        let source = "اطبع(خطأ و (1 / 0 == 0))\nاطبع(صح أو (1 / 0 == 0))";
        assert_eq!(run(source).unwrap(), vec!["خطأ", "صح"]);
    }

    #[test]
    fn string_concatenation_has_a_size_limit() {
        let source = "دع س = \"x\"\nدع عدد = 0\nطالما عدد < 20\nس = س + س\nعدد = عدد + 1\nنهاية";
        let error = run(source).unwrap_err();
        assert!(error.message.contains("حد الحجم"));
    }

    #[test]
    fn total_printed_output_has_a_size_limit() {
        let large = "x".repeat(120_000);
        let source = format!(
            "دع نص = \"{large}\"\nدع عدد = 0\nطالما عدد < 10\nاطبع(نص)\nعدد = عدد + 1\nنهاية"
        );
        let error = run(&source).unwrap_err();
        assert!(error.message.contains("إخراج البرنامج"));
    }

    #[test]
    fn loop_step_limit_stops_infinite_loops() {
        let statements = parse(lex("طالما صح\nهاية").unwrap()).unwrap();
        let error = execute_with_step_limit(&statements, 5).unwrap_err();
        assert!(error.message.contains("حد التنفيذ"));
    }
}
