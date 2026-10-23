use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Lexical,
    Syntax,
    Runtime,
    Io,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lexical => "التحليل المعجمي",
            Self::Syntax => "التحليل النحوي",
            Self::Runtime => "وقت التشغيل",
            Self::Io => "الإدخال والإخراج",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub phase: Phase,
    pub span: Span,
    pub message: String,
}

impl Diagnostic {
    pub fn new(phase: Phase, span: Span, message: impl Into<String>) -> Self {
        Self {
            phase,
            span,
            message: message.into(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "خطأ في {}، السطر {}، العمود {}: {}",
            self.phase, self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for Diagnostic {}
