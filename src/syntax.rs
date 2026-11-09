use icu_normalizer::ComposingNormalizerBorrowed;
use icu_properties::{CodePointMapData, props::GeneralCategory};

use crate::Diagnostic;
use crate::integer::Integer;

const MAX_TOKENS_PER_LINE: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Type {
    Integer,
    Boolean,
    /// 지속 시간(두 시각의 차이). 0.1의 `시간` 타입과 같은 값입니다.
    Time,
    /// 길이(두 위치의 차이).
    Distance,
    /// 시간축 기준 위의 한 점.
    Instant,
    /// 위치축 기준 위의 한 점.
    Position,
    /// 사건에 참여하는 이름 있는 대상.
    Entity,
    /// 퍼지 논리의 참 정도(0%~100%).
    Degree,
}

impl Type {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Integer => "정수",
            Self::Boolean => "논리",
            Self::Time => "시간",
            Self::Distance => "거리",
            Self::Instant => "시각",
            Self::Position => "위치",
            Self::Entity => "대상",
            Self::Degree => "정도",
        }
    }

    pub(crate) fn base_unit(self) -> Option<&'static str> {
        match self {
            Self::Time | Self::Instant => Some("밀리초"),
            Self::Distance | Self::Position => Some("밀리미터"),
            _ => None,
        }
    }

    /// 기준 위의 점이면 그 점들의 차이에 해당하는 타입을 돌려줍니다.
    pub(crate) fn displacement(self) -> Option<Type> {
        match self {
            Self::Instant => Some(Self::Time),
            Self::Position => Some(Self::Distance),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axis {
    Time,
    Space,
}

impl Axis {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Time => "시간축",
            Self::Space => "위치축",
        }
    }

    pub(crate) fn point(self) -> Type {
        match self {
            Self::Time => Type::Instant,
            Self::Space => Type::Position,
        }
    }

    pub(crate) fn offset(self) -> Type {
        match self {
            Self::Time => Type::Time,
            Self::Space => Type::Distance,
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
    Frame,
    Event,
    Unknown,
    Assume,
    Goal,
    Observe,
    Verb,
    Reference,
    /// `참조 N:` 뒤의 본문. 낱말로 나누지 않고 그대로 보관합니다.
    Text,
    Axis(Axis),
    True,
    False,
    And,
    Or,
    Not,
    Colon,
    Comma,
    Dot,
    At,
    Question,
    Percent,
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
    pub(crate) fn line(&self) -> usize {
        self.line
    }

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
        "기준" => Kind::Frame,
        "사건" => Kind::Event,
        "미지" => Kind::Unknown,
        "조건" => Kind::Assume,
        "목표" => Kind::Goal,
        "관측" => Kind::Observe,
        "행위" => Kind::Verb,
        "참조" => Kind::Reference,
        "정도" => Kind::Type(Type::Degree),
        "시간축" => Kind::Axis(Axis::Time),
        "위치축" => Kind::Axis(Axis::Space),
        "시각" => Kind::Type(Type::Instant),
        "위치" => Kind::Type(Type::Position),
        "대상" => Kind::Type(Type::Entity),
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
                    (',', _) => Kind::Comma,
                    ('.', _) => Kind::Dot,
                    ('@', _) => Kind::At,
                    ('?', _) => Kind::Question,
                    ('%', _) => Kind::Percent,
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

            // `참조 N:` 뒤는 자유 본문입니다. 주석 기호 `#`도 본문에 포함합니다.
            let line_tokens = &tokens[tokens.len() - count..];
            if count == 3
                && matches!(
                    line_tokens
                        .iter()
                        .map(|t| t.kind)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    [Kind::Reference, Kind::Number, Kind::Colon]
                )
            {
                let rest = characters
                    .peek()
                    .map_or(text.len(), |&(position, _)| position);
                let body = &text[rest..];
                let leading = body.len() - body.trim_start().len();
                let body_column = column + body[..leading].encode_utf16().count();
                tokens.push(Token {
                    kind: Kind::Text,
                    text: normalizer.normalize(body.trim()).into_owned(),
                    line,
                    column: body_column,
                });
                break;
            }
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
    /// `기준 @ 값`: 기준 위의 점을 만듭니다.
    Anchor(Token, Token, Box<Expression>),
    /// `사건.역할`: 사건에 연결된 역할 값을 읽습니다.
    Member(Token, Token),
}

pub(crate) struct Role {
    pub(crate) name: Token,
    pub(crate) expression: Expression,
}

pub(crate) enum Statement {
    /// `정의`(타입 있음)와 `검사`(타입 없음).
    Value {
        name: Token,
        declared_type: Option<Type>,
        expression: Expression,
    },
    Frame {
        name: Token,
        axis: Axis,
    },
    Entity {
        name: Token,
    },
    Event {
        name: Token,
        roles: Vec<Role>,
    },
    /// `미지 이름: 시간 = ?`, `미지 이름: 시각 = 기준 @ ?`
    Unknown {
        name: Token,
        kind: Type,
        frame: Option<Token>,
    },
    /// `조건`(모든 해가 따라야 하는 가정)과 `목표`(가능·보장 여부를 판정할 식).
    Constraint {
        name: Token,
        goal: bool,
        expression: Expression,
    },
    /// `관측 미지이름 = 값`
    Observe {
        name: Token,
        expression: Expression,
    },
    /// `행위 설계하다 (설계했다): 주체, 대상`
    Verb {
        name: Token,
        forms: Vec<Token>,
        required: Vec<Token>,
    },
    /// `참조 1: 본문`
    ReferenceDefinition {
        number: Token,
        text: Token,
    },
    /// 키워드로 시작하지 않는 줄. `이름: 문장`처럼 이름을 붙일 수 있습니다.
    Sentence {
        label: Option<Token>,
        tokens: Vec<Token>,
    },
}
