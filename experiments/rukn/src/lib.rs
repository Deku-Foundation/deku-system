mod ast;
mod diagnostic;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod typechecker;

pub use diagnostic::{Diagnostic, Phase, Span};
pub use interpreter::DEFAULT_STEP_LIMIT;
pub use lexer::MAX_SOURCE_BYTES;

/// Parse a source file without executing it.
pub fn check(source: &str) -> Result<(), Diagnostic> {
    let tokens = lexer::lex(source)?;
    let statements = parser::parse(tokens)?;
    typechecker::check(&statements)
}

/// Execute a Rukn source file with the default interpreter step limit.
pub fn run(source: &str) -> Result<Vec<String>, Diagnostic> {
    run_with_step_limit(source, DEFAULT_STEP_LIMIT)
}

/// Execute source with a caller-selected step limit, useful for tests and embedding.
pub fn run_with_step_limit(source: &str, step_limit: u64) -> Result<Vec<String>, Diagnostic> {
    let tokens = lexer::lex(source)?;
    let statements = parser::parse(tokens)?;
    typechecker::check(&statements)?;
    interpreter::execute_with_step_limit(&statements, step_limit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_malformed_sources_return_errors_without_panicking() {
        let atoms = [
            "دع", "ثابت", "اطبع", "إذا", "وإلا", "طالما", "نهاية", "صح", "خطأ",
            "ليس", "و", "أو", "=", "==", "!=", "<", ">", "+", "-", "*", "/", "%",
            "(", ")", "\n", "# تعليق", "\"", "٠", "۷", "name", "_", "،", "\u{200D}",
        ];
        let mut state = 0x524B_4E31_u64;
        for _ in 0..512 {
            let mut source = String::new();
            let count = (state as usize % 48) + 1;
            for _ in 0..count {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                source.push_str(atoms[(state as usize >> 17) % atoms.len()]);
                if state & 3 == 0 {
                    source.push(' ');
                }
            }
            let result = std::panic::catch_unwind(|| check(&source));
            assert!(result.is_ok(), "compiler panicked on source: {source:?}");
        }
    }

    #[test]
    fn check_does_not_execute_program() {
        let source = "اطبع(10 / 0)";
        assert!(check(source).is_ok());
        assert!(run(source).is_err());
    }

    #[test]
    fn reports_source_location_for_syntax_errors() {
        let error = check("دع س = 1\nاطبع(").unwrap_err();
        assert_eq!(error.phase, Phase::Syntax);
        assert_eq!(error.span.line, 2);
    }
}
