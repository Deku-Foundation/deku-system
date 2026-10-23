use crate::diagnostic::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Identifier(String),
    Integer(i64),
    String(String),
    Newline,
    LeftParen,
    RightParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Let,
    Const,
    Print,
    If,
    Else,
    End,
    While,
    True,
    False,
    And,
    Or,
    Not,
    Eof,
}

impl TokenKind {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Identifier(_) => "اسم",
            Self::Integer(_) => "عدد صحيح",
            Self::String(_) => "نص",
            Self::Newline => "نهاية سطر",
            Self::LeftParen => "'('",
            Self::RightParen => "')'",
            Self::Comma => "','",
            Self::Plus => "'+'",
            Self::Minus => "'-'",
            Self::Star => "'*'",
            Self::Slash => "'/'",
            Self::Percent => "'%'",
            Self::Equal => "'='",
            Self::EqualEqual => "'=='",
            Self::BangEqual => "'!='",
            Self::Less => "'<'",
            Self::LessEqual => "'<='",
            Self::Greater => "'>'",
            Self::GreaterEqual => "'>='",
            Self::Let => "'دع'",
            Self::Const => "'ثابت'",
            Self::Print => "'اطبع'",
            Self::If => "'إذا'",
            Self::Else => "'وإلا'",
            Self::End => "'نهاية'",
            Self::While => "'طالما'",
            Self::True => "'صح'",
            Self::False => "'خطأ'",
            Self::And => "'و'",
            Self::Or => "'أو'",
            Self::Not => "'ليس'",
            Self::Eof => "نهاية الملف",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
