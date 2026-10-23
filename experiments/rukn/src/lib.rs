mod ast;
mod diagnostic;
mod interpreter;
mod lexer;
mod parser;
mod token;

pub use diagnostic::{Diagnostic, Phase, Span};
pub use interpreter::DEFAULT_STEP_LIMIT;

/// Parse a source file without executing it.
pub fn check(source: &str) -> Result<(), Diagnostic> {
    let tokens = lexer::lex(source)?;
    parser::parse(tokens).map(|_| ())
}

/// Execute a Rukn source file with the default interpreter step limit.
pub fn run(source: &str) -> Result<Vec<String>, Diagnostic> {
    run_with_step_limit(source, DEFAULT_STEP_LIMIT)
}

/// Execute source with a caller-selected step limit, useful for tests and embedding.
pub fn run_with_step_limit(source: &str, step_limit: u64) -> Result<Vec<String>, Diagnostic> {
    let tokens = lexer::lex(source)?;
    let statements = parser::parse(tokens)?;
    interpreter::execute_with_step_limit(&statements, step_limit)
}

#[cfg(test)]
mod tests {
    use super::*;

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
