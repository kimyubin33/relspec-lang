use crate::Diagnostic;
use crate::integer::Integer;
use crate::syntax::{Expression, Kind, Role, Statement, Token, Type};

pub(crate) fn parse(tokens: &[Token]) -> Result<Vec<Statement>, Diagnostic> {
    Parser { tokens, index: 0 }.parse()
}

struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
}

impl Parser<'_> {
    fn parse(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        let mut statements = Vec::new();

        while self.current().kind != Kind::End {
            if self.current().kind == Kind::NewLine {
                self.take();
                continue;
            }

            let statement = match self.current().kind {
                Kind::Define | Kind::Check => self.value_statement()?,
                Kind::Frame => self.frame_statement()?,
                Kind::Type(Type::Entity) => {
                    self.take();
                    let name = self.expect(Kind::Identifier, "선언할 대상의 이름을 적으세요.")?;
                    Statement::Entity { name }
                }
                Kind::Event => self.event_statement()?,
                Kind::Unknown => self.unknown_statement()?,
                Kind::Assume | Kind::Goal => {
                    let goal = self.take().kind == Kind::Goal;
                    let name = self.expect(Kind::Identifier, "선언할 이름을 적으세요.")?;
                    self.expect(Kind::Assign, "식 앞에 '='를 적으세요.")?;
                    let expression = self.expression(1)?;
                    Statement::Constraint {
                        name,
                        goal,
                        expression,
                    }
                }
                Kind::Observe => {
                    self.take();
                    let name = self.expect(Kind::Identifier, "관측한 미지값의 이름을 적으세요.")?;
                    self.expect(Kind::Assign, "이름 뒤에 '='와 관측값을 적으세요.")?;
                    let expression = self.expression(1)?;
                    Statement::Observe { name, expression }
                }
                _ => {
                    return Err(self.current().error(
                        "K200",
                        "'정의', '검사', '기준', '대상', '사건', '미지', '조건', '목표', '관측' 중 하나로 줄을 시작하세요.",
                    ));
                }
            };
            statements.push(statement);

            if !matches!(self.current().kind, Kind::NewLine | Kind::End) {
                return Err(self.current().error(
                    "K200",
                    "한 줄에는 문장 하나를 적으세요. 비교에는 '=='를 사용합니다.",
                ));
            }
        }

        if statements.is_empty() {
            return Err(self.current().error("K201", "적어도 한 문장을 작성하세요."));
        }
        Ok(statements)
    }

    fn value_statement(&mut self) -> Result<Statement, Diagnostic> {
        let declaration = self.take().kind;
        let name = self.expect(Kind::Identifier, "선언할 이름을 적으세요.")?;
        let declared_type = if declaration == Kind::Define {
            self.expect(Kind::Colon, "이름 뒤에 ':'와 타입을 적으세요.")?;
            let Kind::Type(kind) = self.current().kind else {
                return Err(self.current().error(
                    "K200",
                    "타입은 정수, 논리, 시간, 거리, 시각, 위치, 대상 중 하나입니다.",
                ));
            };
            self.take();
            Some(kind)
        } else {
            None
        };

        self.expect(Kind::Assign, "식 앞에 '='를 적으세요.")?;
        let expression = self.expression(1)?;
        Ok(Statement::Value {
            name,
            declared_type,
            expression,
        })
    }

    fn frame_statement(&mut self) -> Result<Statement, Diagnostic> {
        self.take();
        let name = self.expect(Kind::Identifier, "선언할 기준의 이름을 적으세요.")?;
        self.expect(
            Kind::Colon,
            "기준 이름 뒤에 ':'와 '시간축' 또는 '위치축'을 적으세요.",
        )?;
        let Kind::Axis(axis) = self.current().kind else {
            return Err(self
                .current()
                .error("K200", "기준의 종류는 '시간축' 또는 '위치축'입니다."));
        };
        self.take();
        Ok(Statement::Frame { name, axis })
    }

    fn unknown_statement(&mut self) -> Result<Statement, Diagnostic> {
        self.take();
        let name = self.expect(Kind::Identifier, "선언할 미지값의 이름을 적으세요.")?;
        self.expect(Kind::Colon, "이름 뒤에 ':'와 타입을 적으세요.")?;
        let Kind::Type(kind) = self.current().kind else {
            return Err(self.current().error(
                "K200",
                "미지값의 타입은 정수, 시간, 거리, 시각, 위치 중 하나입니다.",
            ));
        };
        let type_token = self.take();
        self.expect(
            Kind::Assign,
            "타입 뒤에 '= ?' 또는 '= 기준 @ ?'를 적으세요.",
        )?;
        let frame = if self.current().kind == Kind::Identifier {
            let frame = self.take();
            self.expect(Kind::At, "기준 이름 뒤에 '@ ?'를 적으세요.")?;
            Some(frame)
        } else {
            None
        };
        self.expect(Kind::Question, "알 수 없는 값은 '?'로 적으세요.")?;

        let needs_frame = matches!(kind, Type::Instant | Type::Position);
        let allowed = matches!(
            kind,
            Type::Integer | Type::Time | Type::Distance | Type::Instant | Type::Position
        );
        if !allowed {
            return Err(type_token.error(
                "K200",
                "미지값의 타입은 정수, 시간, 거리, 시각, 위치 중 하나입니다.",
            ));
        }
        if needs_frame != frame.is_some() {
            let message = if needs_frame {
                format!(
                    "{}은 기준 위의 점이므로 '= 기준 @ ?'로 적으세요.",
                    kind.name()
                )
            } else {
                format!("{}은 기준이 필요 없으므로 '= ?'로 적으세요.", kind.name())
            };
            return Err(type_token.error("K200", message));
        }
        Ok(Statement::Unknown { name, kind, frame })
    }

    fn event_statement(&mut self) -> Result<Statement, Diagnostic> {
        self.take();
        let name = self.expect(Kind::Identifier, "선언할 사건의 이름을 적으세요.")?;
        self.expect(Kind::Colon, "사건 이름 뒤에 ':'와 역할을 적으세요.")?;
        let mut roles = Vec::new();
        loop {
            let role = match self.current().kind {
                Kind::Identifier | Kind::Type(Type::Entity) => self.take(),
                _ => {
                    return Err(self
                        .current()
                        .error("K200", "'시작 = 값'처럼 역할 이름과 값을 적으세요."));
                }
            };
            self.expect(Kind::Assign, "역할 이름 뒤에 '='를 적으세요.")?;
            let expression = self.expression(1)?;
            roles.push(Role {
                name: role,
                expression,
            });
            if self.current().kind != Kind::Comma {
                break;
            }
            self.take();
        }
        Ok(Statement::Event { name, roles })
    }

    fn expression(&mut self, minimum: u8) -> Result<Expression, Diagnostic> {
        let mut left = self.prefix()?;
        while precedence(self.current().kind) >= minimum {
            let op = self.take();
            let right = self.expression(precedence(op.kind) + 1)?;
            left = Expression::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn prefix(&mut self) -> Result<Expression, Diagnostic> {
        match self.current().kind {
            Kind::Number => {
                let number = Integer::from_decimal(&self.take().text);
                if let Kind::Unit(kind, factor) = self.current().kind {
                    self.take();
                    return Ok(Expression::Literal(
                        kind,
                        number.mul(&Integer::from_small(factor)),
                    ));
                }
                Ok(Expression::Literal(Type::Integer, number))
            }
            Kind::True | Kind::False => {
                let bit = u32::from(self.take().kind == Kind::True);
                Ok(Expression::Literal(Type::Boolean, Integer::from_small(bit)))
            }
            Kind::Identifier => {
                let name = self.take();
                match self.current().kind {
                    Kind::Dot => {
                        self.take();
                        let field = match self.current().kind {
                            Kind::Identifier | Kind::Type(Type::Entity) => self.take(),
                            _ => {
                                return Err(self
                                    .current()
                                    .error("K200", "'.' 뒤에 역할 이름을 적으세요."));
                            }
                        };
                        Ok(Expression::Member(name, field))
                    }
                    Kind::At => {
                        let at = self.take();
                        Ok(Expression::Anchor(name, at, Box::new(self.prefix()?)))
                    }
                    _ => Ok(Expression::Reference(name)),
                }
            }
            Kind::Minus | Kind::Not => {
                let op = self.take();
                Ok(Expression::Unary(op, Box::new(self.prefix()?)))
            }
            Kind::LeftParen => {
                self.take();
                let expression = self.expression(1)?;
                self.expect(Kind::RightParen, "닫는 괄호 ')'가 필요합니다.")?;
                Ok(expression)
            }
            _ => Err(self
                .current()
                .error("K200", "숫자, 참·거짓, 이름 또는 괄호로 식을 작성하세요.")),
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn take(&mut self) -> Token {
        let token = self.current().clone();
        self.index += 1;
        token
    }

    fn expect(&mut self, kind: Kind, message: &str) -> Result<Token, Diagnostic> {
        if self.current().kind == kind {
            Ok(self.take())
        } else {
            Err(self.current().error("K200", message))
        }
    }
}

fn precedence(kind: Kind) -> u8 {
    match kind {
        Kind::Or => 1,
        Kind::And => 2,
        Kind::Equal | Kind::NotEqual => 3,
        Kind::Less | Kind::LessEqual | Kind::Greater | Kind::GreaterEqual => 4,
        Kind::Plus | Kind::Minus => 5,
        Kind::Star => 6,
        _ => 0,
    }
}
