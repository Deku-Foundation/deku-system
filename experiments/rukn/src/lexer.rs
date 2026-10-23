use unicode_ident::{is_xid_continue, is_xid_start};
use unicode_normalization::UnicodeNormalization;

use crate::diagnostic::{Diagnostic, Phase, Span};
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DigitSet {
    Ascii,
    ArabicIndic,
    ExtendedArabicIndic,
}

pub fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    Scanner::new(source).scan()
}

struct Scanner {
    chars: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            current: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    fn scan(mut self) -> Result<Vec<Token>, Diagnostic> {
        while !self.is_at_end() {
            let start = Span::new(self.line, self.column);
            let ch = self.advance();

            if is_forbidden_control(ch) {
                return Err(Diagnostic::new(
                    Phase::Lexical,
                    start,
                    "محارف التحكم باتجاه النص غير مسموحة في الشيفرة المصدرية",
                ));
            }

            match ch {
                ' ' | '\t' | '\r' => {}
                '\n' => self.push(TokenKind::Newline, start),
                '#' => self.skip_comment(start)?,
                '(' => self.push(TokenKind::LeftParen, start),
                ')' => self.push(TokenKind::RightParen, start),
                ',' | '،' => self.push(TokenKind::Comma, start),
                '+' => self.push(TokenKind::Plus, start),
                '-' => self.push(TokenKind::Minus, start),
                '*' => self.push(TokenKind::Star, start),
                '/' => self.push(TokenKind::Slash, start),
                '%' => self.push(TokenKind::Percent, start),
                '=' => {
                    let kind = if self.take('=') {
                        TokenKind::EqualEqual
                    } else {
                        TokenKind::Equal
                    };
                    self.push(kind, start);
                }
                '!' => {
                    if self.take('=') {
                        self.push(TokenKind::BangEqual, start);
                    } else {
                        return Err(Diagnostic::new(
                            Phase::Lexical,
                            start,
                            "استخدم «ليس» للنفي؛ علامة «!» منفردة غير مدعومة",
                        ));
                    }
                }
                '<' => {
                    let kind = if self.take('=') {
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    };
                    self.push(kind, start);
                }
                '>' => {
                    let kind = if self.take('=') {
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    };
                    self.push(kind, start);
                }
                '"' => self.string(start)?,
                c if digit_set(c).is_some() => self.number(c, start)?,
                c if c == '_' || is_xid_start(c) => self.identifier(c, start),
                _ => {
                    return Err(Diagnostic::new(
                        Phase::Lexical,
                        start,
                        format!("محرف غير معروف: {ch:?}"),
                    ));
                }
            }
        }

        self.tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(self.line, self.column),
        ));
        Ok(self.tokens)
    }

    fn identifier(&mut self, first: char, span: Span) {
        let mut text = String::new();
        text.push(first);
        while let Some(ch) = self.peek() {
            if ch == '_' || is_xid_continue(ch) {
                text.push(self.advance());
            } else {
                break;
            }
        }

        let normalized: String = text.nfc().collect();
        let kind = match normalized.as_str() {
            "دع" => TokenKind::Let,
            "ثابت" => TokenKind::Const,
            "اطبع" => TokenKind::Print,
            "إذا" => TokenKind::If,
            "وإلا" => TokenKind::Else,
            "نهاية" => TokenKind::End,
            "طالما" => TokenKind::While,
            "صح" => TokenKind::True,
            "خطأ" => TokenKind::False,
            "و" => TokenKind::And,
            "أو" => TokenKind::Or,
            "ليس" => TokenKind::Not,
            _ => TokenKind::Identifier(normalized),
        };
        self.push(kind, span);
    }

    fn number(&mut self, first: char, span: Span) -> Result<(), Diagnostic> {
        let initial_set = digit_set(first).expect("number starts with a decimal digit");
        let mut text = String::new();
        text.push(normalize_digit(first));
        while let Some(ch) = self.peek() {
            if let Some(set) = digit_set(ch) {
                if set != initial_set {
                    return Err(Diagnostic::new(
                        Phase::Lexical,
                        Span::new(self.line, self.column),
                        "لا تخلط أنظمة الأرقام داخل العدد الواحد",
                    ));
                }
                text.push(normalize_digit(self.advance()));
            } else if ch == '_' {
                self.advance();
                if self.peek().and_then(digit_set).is_none() {
                    return Err(Diagnostic::new(
                        Phase::Lexical,
                        Span::new(self.line, self.column.saturating_sub(1)),
                        "يجب أن يفصل كل تسطير سفلي بين رقمين",
                    ));
                }
            } else {
                break;
            }
        }

        match text.parse::<i64>() {
            Ok(value) => self.push(TokenKind::Integer(value), span),
            Err(_) => {
                return Err(Diagnostic::new(
                    Phase::Lexical,
                    span,
                    "العدد الصحيح خارج المجال المدعوم",
                ));
            }
        }
        Ok(())
    }

    fn string(&mut self, span: Span) -> Result<(), Diagnostic> {
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance();
                self.push(TokenKind::String(value), span);
                return Ok(());
            }
            if ch == '\n' || ch == '\r' {
                return Err(Diagnostic::new(
                    Phase::Lexical,
                    span,
                    "النص غير مغلق قبل نهاية السطر",
                ));
            }
            if is_bidi_control(ch) {
                return Err(Diagnostic::new(
                    Phase::Lexical,
                    Span::new(self.line, self.column),
                    "محارف التحكم باتجاه النص غير مسموحة داخل النصوص في الإصدار الأول",
                ));
            }
            if ch == '\\' {
                self.advance();
                let escape_span = Span::new(self.line, self.column.saturating_sub(1));
                let escaped = self.advance_or_error(escape_span)?;
                match escaped {
                    'n' => value.push('\n'),
                    'r' => value.push('\r'),
                    't' => value.push('\t'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    _ => {
                        return Err(Diagnostic::new(
                            Phase::Lexical,
                            escape_span,
                            format!("تسلسل هروب غير مدعوم: \\{escaped}"),
                        ));
                    }
                }
            } else {
                value.push(self.advance());
            }
        }

        Err(Diagnostic::new(
            Phase::Lexical,
            span,
            "النص غير مغلق؛ أضف علامة اقتباس مزدوجة",
        ))
    }

    fn skip_comment(&mut self, comment_span: Span) -> Result<(), Diagnostic> {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            if is_forbidden_control(ch) {
                return Err(Diagnostic::new(
                    Phase::Lexical,
                    Span::new(self.line, self.column),
                    "محارف اتجاه النص أو التنسيق غير المرئية ممنوعة حتى داخل التعليقات",
                ));
            }
            self.advance();
        }
        let _ = comment_span;
        Ok(())
    }

    fn push(&mut self, kind: TokenKind, span: Span) {
        self.tokens.push(Token::new(kind, span));
    }

    fn advance(&mut self) -> char {
        let ch = self.chars[self.current];
        self.current += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        ch
    }

    fn advance_or_error(&mut self, span: Span) -> Result<char, Diagnostic> {
        if self.is_at_end() {
            Err(Diagnostic::new(
                Phase::Lexical,
                span,
                "تسلسل هروب غير مكتمل",
            ))
        } else {
            Ok(self.advance())
        }
    }

    fn take(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.current).copied()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }
}

fn digit_set(ch: char) -> Option<DigitSet> {
    match ch {
        '0'..='9' => Some(DigitSet::Ascii),
        '٠'..='٩' => Some(DigitSet::ArabicIndic),
        '۰'..='۹' => Some(DigitSet::ExtendedArabicIndic),
        _ => None,
    }
}

fn normalize_digit(ch: char) -> char {
    match ch {
        '٠'..='٩' => char::from_u32(ch as u32 - '٠' as u32 + '0' as u32).unwrap(),
        '۰'..='۹' => char::from_u32(ch as u32 - '۰' as u32 + '0' as u32).unwrap(),
        _ => ch,
    }
}

fn is_forbidden_control(ch: char) -> bool {
    matches!(
        ch as u32,
        0x061C | 0x200B..=0x200F | 0x202A..=0x202E | 0x2060 | 0x2066..=0x2069 | 0xFEFF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_arabic_keywords_and_identifiers() {
        let tokens = lex("دع العدد = ١٢\nاطبع(العدد)").unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Let));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("العدد".into()));
        assert_eq!(tokens[3].kind, TokenKind::Integer(12));
    }

    #[test]
    fn accepts_each_digit_system_without_mixing() {
        for (input, expected) in [("17", 17), ("١٧", 17), ("۱۷", 17)] {
            let tokens = lex(input).unwrap();
            assert_eq!(tokens[0].kind, TokenKind::Integer(expected));
        }
        assert!(lex("1٧").is_err());
    }

    #[test]
    fn normalizes_identifiers_to_nfc() {
        let tokens = lex("دع ا\u{0654} = 1\nدع أ = 2").unwrap();
        assert_eq!(tokens[1].kind, TokenKind::Identifier("أ".into()));
        assert_eq!(tokens[6].kind, TokenKind::Identifier("أ".into()));
    }

    #[test]
    fn rejects_bidi_controls() {
        let error = lex("دع x = 1\u{202E}").unwrap_err();
        assert_eq!(error.phase, Phase::Lexical);
    }

    #[test]
    fn parses_escaped_strings_and_comments() {
        let tokens = lex("اطبع(\"سطر\\nثانٍ\") # تعليق").unwrap();
        assert_eq!(tokens[2].kind, TokenKind::String("سطر\nثانٍ".into()));
    }

    #[test]
    fn rejects_unclosed_strings() {
        assert!(lex("اطبع(\"غير مكتمل").is_err());
    }
}
