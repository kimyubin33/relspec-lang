use crate::Diagnostic;
use crate::integer::Integer;
use crate::syntax::{Expression, Kind, Statement, Token, Type};

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

            if !matches!(self.current().kind, Kind::Define | Kind::Check) {
                return Err(self
                    .current()
                    .error("K200", "'정의' 또는 '검사'로 줄을 시작하세요."));
            }

            let declaration = self.take().kind;
            let name = self.expect(Kind::Identifier, "선언할 이름을 적으세요.")?;
            let declared_type = if declaration == Kind::Define {
                self.expect(Kind::Colon, "이름 뒤에 ':'와 타입을 적으세요.")?;
                let Kind::Type(kind) = self.current().kind else {
                    return Err(self
                        .current()
                        .error("K200", "타입은 정수, 논리, 시간, 거리 중 하나입니다."));
                };
                self.take();
                Some(kind)
            } else {
                None
            };

            self.expect(Kind::Assign, "식 앞에 '='를 적으세요.")?;
            let expression = self.expression(1)?;
            statements.push(Statement {
                name,
                declared_type,
                expression,
            });

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
            Kind::Identifier => Ok(Expression::Reference(self.take())),
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
