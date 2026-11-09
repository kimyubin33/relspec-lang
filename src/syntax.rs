use icu_normalizer::ComposingNormalizerBorrowed;
use icu_properties::{CodePointMapData, props::GeneralCategory};

use crate::Diagnostic;
use crate::integer::Integer;

const MAX_TOKENS_PER_LINE: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Type {
    Integer,
    Boolean,
    Time,
    Distance,
}

impl Type {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Integer => "정수",
            Self::Boolean => "논리",
            Self::Time => "시간",
            Self::Distance => "거리",
        }
    }

    pub(crate) fn base_unit(self) -> Option<&'static str> {
        match self {
            Self::Time => Some("밀리초"),
            Self::Distance => Some("밀리미터"),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    End,
    NewLine,
    Identifier,
    Number,
    Unit(Type, u32),
    Type(Type),
    Define,
    Check,
    True,
    False,
    And,
    Or,
    Not,
    Colon,
    Assign,
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Star,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Debug)]
pub(crate) struct Token {
    pub(crate) kind: Kind,
    pub(crate) text: String,
    line: usize,
    column: usize,
}

impl Token {
    pub(crate) fn error(&self, code: &'static str, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            code,
            line: self.line,
            column: self.column,
            message: message.into(),
        }
    }
}

fn classify_name(name: &str) -> Kind {
    match name {
        "정의" => Kind::Define,
        "검사" => Kind::Check,
        "참" => Kind::True,
        "거짓" => Kind::False,
        "그리고" => Kind::And,
        "또는" => Kind::Or,
        "아님" => Kind::Not,
        "정수" => Kind::Type(Type::Integer),
        "논리" => Kind::Type(Type::Boolean),
        "시간" => Kind::Type(Type::Time),
        "거리" => Kind::Type(Type::Distance),
        "밀리초" => Kind::Unit(Type::Time, 1),
        "초" => Kind::Unit(Type::Time, 1000),
        "밀리미터" => Kind::Unit(Type::Distance, 1),
        "센티미터" => Kind::Unit(Type::Distance, 10),
        "미터" => Kind::Unit(Type::Distance, 1000),
        _ => Kind::Identifier,
    }
}

fn is_name_start(ch: char) -> bool {
    // The first implementation used UTF-16 code units for identifier classification.
    ch == '_'
        || (ch.len_utf16() == 1
            && matches!(
                CodePointMapData::<GeneralCategory>::new().get(ch),
                GeneralCategory::UppercaseLetter
                    | GeneralCategory::LowercaseLetter
                    | GeneralCategory::TitlecaseLetter
                    | GeneralCategory::ModifierLetter
                    | GeneralCategory::OtherLetter
            ))
}

fn is_name_part(ch: char) -> bool {
    is_name_start(ch)
        || (ch.len_utf16() == 1
            && matches!(
                CodePointMapData::<GeneralCategory>::new().get(ch),
                GeneralCategory::DecimalNumber
                    | GeneralCategory::NonspacingMark
                    | GeneralCategory::SpacingMark
            ))
}

pub(crate) fn scan(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let normalizer = ComposingNormalizerBorrowed::new_nfc();
    let mut tokens = Vec::new();
    let mut lines = normalized.split('\n').enumerate().peekable();

    while let Some((i, text)) = lines.next() {
        let line = i + 1;
        let mut characters = text.char_indices().peekable();
        let mut column = 1;
        let mut count = 0;

        while let Some((start, ch)) = characters.next() {
            let token_column = column;
            column += ch.len_utf16();

            if ch.is_whitespace() {
                continue;
            }
            if ch == '#' {
                break;
            }
            count += 1;
            if count > MAX_TOKENS_PER_LINE {
                return Err(Diagnostic {
                    code: "K101",
                    line,
                    column: token_column,
                    message: format!("한 줄에 기호를 {MAX_TOKENS_PER_LINE}개까지 쓸 수 있습니다."),
                });
            }

            let kind;
            let token_text;
            if ch.is_ascii_digit() || is_name_start(ch) {
                let is_part = if ch.is_ascii_digit() {
                    |c: char| c.is_ascii_digit()
                } else {
                    is_name_part
                };
                while let Some((_, next)) = characters.next_if(|&(_, next)| is_part(next)) {
                    column += next.len_utf16();
                }
                let end = characters
                    .peek()
                    .map_or(text.len(), |&(position, _)| position);
                if ch.is_ascii_digit() {
                    kind = Kind::Number;
                    token_text = text[start..end].into();
                } else {
                    token_text = normalizer.normalize(&text[start..end]).into_owned();
                    kind = classify_name(&token_text);
                }
            } else {
                let followed_by_equals = matches!(ch, '=' | '!' | '<' | '>')
                    && characters.next_if(|&(_, next)| next == '=').is_some();
                if followed_by_equals {
                    column += 1;
                }
                kind = match (ch, followed_by_equals) {
                    (':', _) => Kind::Colon,
                    ('(', _) => Kind::LeftParen,
                    (')', _) => Kind::RightParen,
                    ('+', _) => Kind::Plus,
                    ('-', _) => Kind::Minus,
                    ('*', _) => Kind::Star,
                    ('=', false) => Kind::Assign,
                    ('=', true) => Kind::Equal,
                    ('!', true) => Kind::NotEqual,
                    ('<', false) => Kind::Less,
                    ('<', true) => Kind::LessEqual,
                    ('>', false) => Kind::Greater,
                    ('>', true) => Kind::GreaterEqual,
                    _ => {
                        return Err(Diagnostic {
                            code: "K100",
                            line,
                            column: token_column,
                            message: format!("알 수 없는 기호 '{ch}'입니다."),
                        });
                    }
                };
                let end = characters
                    .peek()
                    .map_or(text.len(), |&(position, _)| position);
                token_text = text[start..end].into();
            }

            tokens.push(Token {
                kind,
                text: token_text,
                line,
                column: token_column,
            });
        }

        let kind = if lines.peek().is_some() {
            Kind::NewLine
        } else {
            Kind::End
        };
        tokens.push(Token {
            kind,
            text: String::new(),
            line,
            column: text.encode_utf16().count() + 1,
        });
    }
    Ok(tokens)
}

pub(crate) enum Expression {
    Literal(Type, Integer),
    Reference(Token),
    Unary(Token, Box<Expression>),
    Binary(Token, Box<Expression>, Box<Expression>),
}

pub(crate) struct Statement {
    pub(crate) name: Token,
    pub(crate) declared_type: Option<Type>,
    pub(crate) expression: Expression,
}
