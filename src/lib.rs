mod integer;
mod parser;
mod syntax;

use std::collections::{HashMap, HashSet};
use std::fmt;

use integer::Integer;
use serde::Serialize;
use syntax::{Expression, Kind, Token, Type};

#[derive(Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({},{}): error {}: {}",
            self.line, self.column, self.code, self.message
        )
    }
}

impl std::error::Error for Diagnostic {}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IrExpression {
    Literal {
        #[serde(rename = "type")]
        kind: &'static str,
        value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        unit: Option<&'static str>,
    },
    Reference {
        #[serde(rename = "type")]
        kind: &'static str,
        name: String,
    },
    Unary {
        #[serde(rename = "type")]
        kind: &'static str,
        operator: String,
        operands: [Box<IrExpression>; 1],
    },
    Binary {
        #[serde(rename = "type")]
        kind: &'static str,
        operator: String,
        operands: [Box<IrExpression>; 2],
    },
}

#[derive(Serialize)]
pub struct IrStatement {
    pub kind: &'static str,
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: &'static str,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<&'static str>,
    pub expression: IrExpression,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompiledProgram {
    pub language_version: &'static str,
    pub statements: Vec<IrStatement>,
}

pub fn compile(source: &str) -> Result<CompiledProgram, Diagnostic> {
    let tokens = syntax::scan(source)?;
    let statements = parser::parse(&tokens)?;
    let mut values = HashMap::new();
    let mut names = HashSet::new();
    let mut output = Vec::new();

    for statement in statements {
        let name = statement.name;
        if !names.insert(name.text.clone()) {
            return Err(name.error(
                "K301",
                format!("'{}' 이름이 이미 선언되어 있습니다.", name.text),
            ));
        }

        let Bound { value, ir } = bind(statement.expression, &values)?;
        let expected = statement.declared_type.unwrap_or(Type::Boolean);
        if value.kind != expected {
            return Err(name.error(
                "K303",
                format!(
                    "'{}'에는 {} 타입이 필요하지만 식은 {} 타입입니다.",
                    name.text,
                    expected.name(),
                    value.kind.name(),
                ),
            ));
        }

        let is_definition = statement.declared_type.is_some();
        if !is_definition && value.number.is_zero() {
            return Err(name.error("K401", format!("검사 '{}'의 결과가 거짓입니다.", name.text)));
        }

        output.push(IrStatement {
            kind: if is_definition { "정의" } else { "검사" },
            name: name.text.clone(),
            value_type: value.kind.name(),
            value: value.display(),
            unit: value.kind.base_unit(),
            expression: ir,
        });
        if is_definition {
            values.insert(name.text, value);
        }
    }

    Ok(CompiledProgram {
        language_version: "0.1",
        statements: output,
    })
}

#[derive(Clone)]
struct Value {
    kind: Type,
    number: Integer,
}

impl Value {
    fn logical(result: bool) -> Self {
        Self {
            kind: Type::Boolean,
            number: Integer::from_small(u32::from(result)),
        }
    }

    fn display(&self) -> String {
        if self.kind == Type::Boolean {
            return if self.number.is_zero() {
                "거짓"
            } else {
                "참"
            }
            .into();
        }
        self.number.to_string()
    }
}

struct Bound {
    value: Value,
    ir: IrExpression,
}

fn bind(expression: Expression, values: &HashMap<String, Value>) -> Result<Bound, Diagnostic> {
    let (value, ir) = match expression {
        Expression::Literal(kind, number) => {
            let value = Value { kind, number };
            let ir = IrExpression::Literal {
                kind: kind.name(),
                value: value.display(),
                unit: kind.base_unit(),
            };
            (value, ir)
        }
        Expression::Reference(name) => {
            let value = values
                .get(&name.text)
                .ok_or_else(|| {
                    name.error(
                        "K302",
                        format!("'{}'을 사용하기 전에 '정의'로 선언하세요.", name.text),
                    )
                })?
                .clone();
            let ir = IrExpression::Reference {
                kind: value.kind.name(),
                name: name.text,
            };
            (value, ir)
        }
        Expression::Unary(op, operand) => {
            let operand = bind(*operand, values)?;
            let value = evaluate_unary(&op, operand.value)?;
            let ir = IrExpression::Unary {
                kind: value.kind.name(),
                operator: op.text,
                operands: [Box::new(operand.ir)],
            };
            (value, ir)
        }
        Expression::Binary(op, left, right) => {
            let left = bind(*left, values)?;
            let right = bind(*right, values)?;
            let value = evaluate_binary(&op, &left.value, &right.value)?;
            let ir = IrExpression::Binary {
                kind: value.kind.name(),
                operator: op.text,
                operands: [Box::new(left.ir), Box::new(right.ir)],
            };
            (value, ir)
        }
    };
    Ok(Bound { value, ir })
}

fn evaluate_unary(op: &Token, value: Value) -> Result<Value, Diagnostic> {
    match op.kind {
        Kind::Minus if value.kind != Type::Boolean => Ok(Value {
            kind: value.kind,
            number: value.number.negate(),
        }),
        Kind::Not if value.kind == Type::Boolean => Ok(Value::logical(value.number.is_zero())),
        _ => Err(op.error(
            "K304",
            format!(
                "'{}' 연산은 {} 타입에 사용할 수 없습니다.",
                op.text,
                value.kind.name(),
            ),
        )),
    }
}

fn evaluate_binary(op: &Token, left: &Value, right: &Value) -> Result<Value, Diagnostic> {
    let same_type = left.kind == right.kind;
    let numeric = same_type && left.kind != Type::Boolean;
    let boolean = same_type && left.kind == Type::Boolean;
    let a = &left.number;
    let b = &right.number;

    let result = match op.kind {
        Kind::Plus if numeric => Value {
            kind: left.kind,
            number: a.add(b),
        },
        Kind::Minus if numeric => Value {
            kind: left.kind,
            number: a.sub(b),
        },
        Kind::Star if left.kind == Type::Integer && right.kind != Type::Boolean => Value {
            kind: right.kind,
            number: a.mul(b),
        },
        Kind::Star if right.kind == Type::Integer && left.kind != Type::Boolean => Value {
            kind: left.kind,
            number: a.mul(b),
        },
        Kind::Equal if same_type => Value::logical(a == b),
        Kind::NotEqual if same_type => Value::logical(a != b),
        Kind::Less if numeric => Value::logical(a < b),
        Kind::LessEqual if numeric => Value::logical(a <= b),
        Kind::Greater if numeric => Value::logical(a > b),
        Kind::GreaterEqual if numeric => Value::logical(a >= b),
        Kind::And if boolean => Value::logical(!a.is_zero() && !b.is_zero()),
        Kind::Or if boolean => Value::logical(!a.is_zero() || !b.is_zero()),
        _ => {
            return Err(op.error(
                "K304",
                format!(
                    "'{}' 연산에 사용할 수 없는 조합입니다. 왼쪽은 {}, 오른쪽은 {}입니다.",
                    op.text,
                    left.kind.name(),
                    right.kind.name(),
                ),
            ));
        }
    };
    Ok(result)
}
