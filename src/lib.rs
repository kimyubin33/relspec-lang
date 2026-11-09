mod integer;
mod parser;
mod syntax;

use std::collections::{HashMap, HashSet};
use std::fmt;

use integer::Integer;
use serde::Serialize;
use syntax::{Axis, Expression, Kind, Role, Statement, Token, Type};

pub const LANGUAGE_VERSION: &str = "0.2";

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
    /// `기준 @ 값`
    Anchor {
        #[serde(rename = "type")]
        kind: &'static str,
        frame: String,
        operands: [Box<IrExpression>; 1],
    },
    /// `사건.역할`
    Member {
        #[serde(rename = "type")]
        kind: &'static str,
        event: String,
        role: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    pub expression: IrExpression,
}

#[derive(Serialize)]
pub struct IrFrame {
    pub name: String,
    pub axis: &'static str,
}

#[derive(Serialize)]
pub struct IrEntity {
    pub name: String,
}

#[derive(Serialize)]
pub struct IrRole {
    pub role: String,
    #[serde(rename = "type")]
    pub value_type: &'static str,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    /// 역할을 생략해 기본값이 들어간 경우(`종료` = `시작`)에는 식이 없습니다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<IrExpression>,
}

#[derive(Serialize)]
pub struct IrEvent {
    pub name: String,
    pub roles: Vec<IrRole>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompiledProgram {
    pub language_version: &'static str,
    pub statements: Vec<IrStatement>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<IrFrame>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<IrEntity>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<IrEvent>,
}

/// 사건이 받을 수 있는 역할과 각 역할의 값 종류.
const ROLES: &[(&str, Type)] = &[
    ("주체", Type::Entity),
    ("대상", Type::Entity),
    ("관측자", Type::Entity),
    ("시작", Type::Instant),
    ("종료", Type::Instant),
    ("출발", Type::Position),
    ("도착", Type::Position),
];

pub fn compile(source: &str) -> Result<CompiledProgram, Diagnostic> {
    let tokens = syntax::scan(source)?;
    let statements = parser::parse(&tokens)?;
    let mut scope = Scope::default();
    let mut names = HashSet::new();
    let mut program = CompiledProgram {
        language_version: LANGUAGE_VERSION,
        statements: Vec::new(),
        frames: Vec::new(),
        entities: Vec::new(),
        events: Vec::new(),
    };

    for statement in statements {
        let name = match &statement {
            Statement::Value { name, .. }
            | Statement::Frame { name, .. }
            | Statement::Entity { name }
            | Statement::Event { name, .. } => name,
        };
        if !names.insert(name.text.clone()) {
            return Err(name.error(
                "K301",
                format!("'{}' 이름이 이미 선언되어 있습니다.", name.text),
            ));
        }

        match statement {
            Statement::Value {
                name,
                declared_type,
                expression,
            } => {
                let Bound { value, ir } = bind(expression, &scope)?;
                let expected = declared_type.unwrap_or(Type::Boolean);
                check_declared_type(&name, expected, &value, &scope)?;

                let is_definition = declared_type.is_some();
                if !is_definition && value.number.is_zero() {
                    return Err(
                        name.error("K401", format!("검사 '{}'의 결과가 거짓입니다.", name.text))
                    );
                }

                program.statements.push(IrStatement {
                    kind: if is_definition { "정의" } else { "검사" },
                    name: name.text.clone(),
                    value_type: value.kind.name(),
                    value: value.display(),
                    unit: value.kind.base_unit(),
                    frame: scope.frame_name(&value),
                    expression: ir,
                });
                if is_definition {
                    scope.values.insert(name.text, value);
                }
            }
            Statement::Frame { name, axis } => {
                scope
                    .frame_ids
                    .insert(name.text.clone(), scope.frames.len());
                scope.frames.push((name.text.clone(), axis));
                program.frames.push(IrFrame {
                    name: name.text,
                    axis: axis.name(),
                });
            }
            Statement::Entity { name } => {
                let value = Value {
                    kind: Type::Entity,
                    frame: None,
                    number: Integer::from_small(program.entities.len() as u32),
                    label: Some(name.text.clone()),
                };
                scope.values.insert(name.text.clone(), value);
                program.entities.push(IrEntity { name: name.text });
            }
            Statement::Event { name, roles } => {
                let (record, ir) = bind_event(&name, roles, &scope)?;
                scope.events.insert(name.text, record);
                program.events.push(ir);
            }
        }
    }

    Ok(program)
}

#[derive(Default)]
struct Scope {
    values: HashMap<String, Value>,
    frame_ids: HashMap<String, usize>,
    frames: Vec<(String, Axis)>,
    events: HashMap<String, EventRecord>,
}

impl Scope {
    fn frame_name(&self, value: &Value) -> Option<String> {
        value.frame.map(|id| self.frames[id].0.clone())
    }

    /// 진단 메시지용 타입 이름. 기준 위의 점이면 기준 이름을 덧붙입니다.
    fn describe(&self, value: &Value) -> String {
        match value.frame {
            Some(id) => format!("{}({})", value.kind.name(), self.frames[id].0),
            None => value.kind.name().into(),
        }
    }
}

struct EventRecord {
    roles: HashMap<&'static str, Value>,
}

#[derive(Clone)]
struct Value {
    kind: Type,
    /// 시각·위치가 놓인 기준. 다른 타입에서는 항상 `None`입니다.
    frame: Option<usize>,
    number: Integer,
    /// 대상의 이름.
    label: Option<String>,
}

impl Value {
    fn plain(kind: Type, number: Integer) -> Self {
        Self {
            kind,
            frame: None,
            number,
            label: None,
        }
    }

    fn point(kind: Type, frame: Option<usize>, number: Integer) -> Self {
        Self {
            kind,
            frame,
            number,
            label: None,
        }
    }

    fn logical(result: bool) -> Self {
        Self::plain(Type::Boolean, Integer::from_small(u32::from(result)))
    }

    fn display(&self) -> String {
        match self.kind {
            Type::Boolean => if self.number.is_zero() {
                "거짓"
            } else {
                "참"
            }
            .into(),
            Type::Entity => self.label.clone().unwrap_or_default(),
            _ => self.number.to_string(),
        }
    }
}

struct Bound {
    value: Value,
    ir: IrExpression,
}

/// 지속 시간과 시각, 길이와 위치처럼 차원은 같지만 역할이 다른 조합인지 확인합니다.
fn point_vector_pair(a: Type, b: Type) -> bool {
    a.displacement() == Some(b) || b.displacement() == Some(a)
}

fn check_declared_type(
    name: &Token,
    expected: Type,
    value: &Value,
    scope: &Scope,
) -> Result<(), Diagnostic> {
    if value.kind == expected {
        return Ok(());
    }
    if point_vector_pair(expected, value.kind) {
        return Err(name.error(
            "K307",
            format!(
                "'{}'에는 {} 값이 필요하지만 식은 {}입니다. {}",
                name.text,
                expected.name(),
                scope.describe(value),
                point_hint(expected),
            ),
        ));
    }
    Err(name.error(
        "K303",
        format!(
            "'{}'에는 {} 타입이 필요하지만 식은 {} 타입입니다.",
            name.text,
            expected.name(),
            scope.describe(value),
        ),
    ))
}

fn point_hint(expected: Type) -> &'static str {
    match expected {
        Type::Instant => "시각은 '기준 @ 1 초'처럼 시간축 기준 위에 놓아야 합니다.",
        Type::Position => "위치는 '기준 @ 1 미터'처럼 위치축 기준 위에 놓아야 합니다.",
        Type::Time => "두 시각의 차이(종료 - 시작)가 지속 시간입니다.",
        Type::Distance => "두 위치의 차이(도착 - 출발)가 거리입니다.",
        _ => "",
    }
}

fn bind_event(
    name: &Token,
    roles: Vec<Role>,
    scope: &Scope,
) -> Result<(EventRecord, IrEvent), Diagnostic> {
    let mut values: HashMap<&'static str, Value> = HashMap::new();
    let mut tokens: HashMap<&'static str, Token> = HashMap::new();
    let mut ir_roles = Vec::new();

    for Role {
        name: role,
        expression,
    } in roles
    {
        let Some(&(role_name, expected)) = ROLES.iter().find(|(r, _)| *r == role.text) else {
            let known: Vec<_> = ROLES.iter().map(|(r, _)| *r).collect();
            return Err(role.error(
                "K311",
                format!(
                    "'{}'은 사건 역할이 아닙니다. 사용할 수 있는 역할: {}",
                    role.text,
                    known.join(", "),
                ),
            ));
        };
        if values.contains_key(role_name) {
            return Err(role.error(
                "K310",
                format!(
                    "사건 '{}'에 '{}' 역할이 두 번 선언되었습니다.",
                    name.text, role_name
                ),
            ));
        }

        let Bound { value, ir } = bind(expression, scope)?;
        if value.kind != expected {
            let (code, hint) = if point_vector_pair(expected, value.kind) {
                ("K307", point_hint(expected))
            } else {
                ("K313", "")
            };
            return Err(role.error(
                code,
                format!(
                    "'{}' 역할에는 {} 값이 필요하지만 식은 {}입니다. {}",
                    role_name,
                    expected.name(),
                    scope.describe(&value),
                    hint,
                )
                .trim_end()
                .to_string(),
            ));
        }

        ir_roles.push(IrRole {
            role: role_name.into(),
            value_type: value.kind.name(),
            value: value.display(),
            unit: value.kind.base_unit(),
            frame: scope.frame_name(&value),
            expression: Some(ir),
        });
        values.insert(role_name, value);
        tokens.insert(role_name, role);
    }

    let Some(start) = values.get("시작").cloned() else {
        return Err(name.error(
            "K312",
            format!("사건 '{}'에는 '시작' 역할이 필요합니다.", name.text),
        ));
    };

    match values.get("종료") {
        Some(end) => {
            if end.frame != start.frame {
                return Err(tokens["종료"].error(
                    "K306",
                    format!(
                        "사건 '{}'의 시작은 {}, 종료는 {}입니다. 같은 기준을 사용하세요.",
                        name.text,
                        scope.describe(&start),
                        scope.describe(end),
                    ),
                ));
            }
            if end.number < start.number {
                return Err(name.error(
                    "K402",
                    format!(
                        "사건 '{}'의 종료({} 밀리초)가 시작({} 밀리초)보다 앞섭니다.",
                        name.text, end.number, start.number,
                    ),
                ));
            }
        }
        None => {
            // 종료를 생략한 사건은 한 시각에 일어난 사건으로 봅니다.
            ir_roles.push(IrRole {
                role: "종료".into(),
                value_type: start.kind.name(),
                value: start.display(),
                unit: start.kind.base_unit(),
                frame: scope.frame_name(&start),
                expression: None,
            });
            values.insert("종료", start.clone());
        }
    }

    match (values.get("출발"), values.get("도착")) {
        (Some(from), Some(to)) if from.frame != to.frame => {
            return Err(tokens["도착"].error(
                "K306",
                format!(
                    "사건 '{}'의 출발은 {}, 도착은 {}입니다. 같은 기준을 사용하세요.",
                    name.text,
                    scope.describe(from),
                    scope.describe(to),
                ),
            ));
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(name.error(
                "K312",
                format!(
                    "사건 '{}'에는 '출발'과 '도착'을 함께 적어야 합니다.",
                    name.text
                ),
            ));
        }
        _ => {}
    }

    Ok((
        EventRecord { roles: values },
        IrEvent {
            name: name.text.clone(),
            roles: ir_roles,
        },
    ))
}

fn member(event_token: &Token, field: &Token, scope: &Scope) -> Result<Value, Diagnostic> {
    let Some(event) = scope.events.get(&event_token.text) else {
        return Err(event_token.error(
            "K302",
            format!(
                "'{}'을 사용하기 전에 '사건'으로 선언하세요.",
                event_token.text
            ),
        ));
    };
    let roles = &event.roles;
    let derived = |a: &str, b: &str, kind: Type| {
        let (Some(first), Some(last)) = (roles.get(a), roles.get(b)) else {
            return None;
        };
        Some(Value::plain(kind, last.number.sub(&first.number)))
    };
    let value = match field.text.as_str() {
        "소요" => derived("시작", "종료", Type::Time),
        "이동" => derived("출발", "도착", Type::Distance),
        role => roles.get(role).cloned(),
    };
    value.ok_or_else(|| {
        field.error(
            "K314",
            format!(
                "사건 '{}'에서 '{}'을 읽을 수 없습니다. 선언한 역할 또는 '소요'·'이동'을 사용하세요.",
                event_token.text, field.text,
            ),
        )
    })
}

fn bind(expression: Expression, scope: &Scope) -> Result<Bound, Diagnostic> {
    let (value, ir) = match expression {
        Expression::Literal(kind, number) => {
            let value = Value::plain(kind, number);
            let ir = IrExpression::Literal {
                kind: kind.name(),
                value: value.display(),
                unit: kind.base_unit(),
            };
            (value, ir)
        }
        Expression::Reference(name) => {
            let Some(value) = scope.values.get(&name.text).cloned() else {
                return Err(unbound(&name, scope));
            };
            let ir = IrExpression::Reference {
                kind: value.kind.name(),
                name: name.text,
            };
            (value, ir)
        }
        Expression::Member(event, field) => {
            let value = member(&event, &field, scope)?;
            let ir = IrExpression::Member {
                kind: value.kind.name(),
                event: event.text,
                role: field.text,
            };
            (value, ir)
        }
        Expression::Anchor(frame, at, operand) => {
            let Some(&id) = scope.frame_ids.get(&frame.text) else {
                return Err(frame.error(
                    "K308",
                    format!(
                        "'@' 앞에는 '기준'으로 선언한 이름이 필요합니다. '{}'은 기준이 아닙니다.",
                        frame.text
                    ),
                ));
            };
            let axis = scope.frames[id].1;
            let operand = bind(*operand, scope)?;
            if operand.value.kind != axis.offset() {
                return Err(at.error(
                    "K308",
                    format!(
                        "{} 기준 '{}'에는 {} 값을 놓아야 하지만 {}이 주어졌습니다.",
                        axis.name(),
                        frame.text,
                        axis.offset().name(),
                        scope.describe(&operand.value),
                    ),
                ));
            }
            let value = Value::point(axis.point(), Some(id), operand.value.number);
            let ir = IrExpression::Anchor {
                kind: value.kind.name(),
                frame: frame.text,
                operands: [Box::new(operand.ir)],
            };
            (value, ir)
        }
        Expression::Unary(op, operand) => {
            let operand = bind(*operand, scope)?;
            let value = evaluate_unary(&op, operand.value, scope)?;
            let ir = IrExpression::Unary {
                kind: value.kind.name(),
                operator: op.text,
                operands: [Box::new(operand.ir)],
            };
            (value, ir)
        }
        Expression::Binary(op, left, right) => {
            let left = bind(*left, scope)?;
            let right = bind(*right, scope)?;
            let value = evaluate_binary(&op, &left.value, &right.value, scope)?;
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

fn unbound(name: &Token, scope: &Scope) -> Diagnostic {
    if let Some(&id) = scope.frame_ids.get(&name.text) {
        let unit = match scope.frames[id].1 {
            Axis::Time => "0 초",
            Axis::Space => "0 미터",
        };
        return name.error(
            "K309",
            format!(
                "'{}'은 기준이므로 값으로 쓸 수 없습니다. '{} @ {unit}'처럼 기준 위의 점을 만드세요.",
                name.text, name.text,
            ),
        );
    }
    if scope.events.contains_key(&name.text) {
        return name.error(
            "K309",
            format!(
                "'{}'은 사건이므로 값으로 쓸 수 없습니다. '{}.시작'처럼 역할을 고르세요.",
                name.text, name.text,
            ),
        );
    }
    name.error(
        "K302",
        format!("'{}'을 사용하기 전에 '정의'로 선언하세요.", name.text),
    )
}

fn evaluate_unary(op: &Token, value: Value, scope: &Scope) -> Result<Value, Diagnostic> {
    match op.kind {
        Kind::Minus if matches!(value.kind, Type::Integer | Type::Time | Type::Distance) => {
            Ok(Value::plain(value.kind, value.number.negate()))
        }
        Kind::Not if value.kind == Type::Boolean => Ok(Value::logical(value.number.is_zero())),
        _ => Err(op.error(
            "K304",
            format!(
                "'{}' 연산은 {} 타입에 사용할 수 없습니다.",
                op.text,
                scope.describe(&value),
            ),
        )),
    }
}

/// 두 점이 같은 기준에 있는지 확인합니다.
fn same_frame(op: &Token, left: &Value, right: &Value, scope: &Scope) -> Result<(), Diagnostic> {
    if left.frame == right.frame {
        return Ok(());
    }
    Err(op.error(
        "K306",
        format!(
            "'{}' 연산의 두 값이 서로 다른 기준에 있습니다. 왼쪽은 {}, 오른쪽은 {}입니다.",
            op.text,
            scope.describe(left),
            scope.describe(right),
        ),
    ))
}

fn evaluate_binary(
    op: &Token,
    left: &Value,
    right: &Value,
    scope: &Scope,
) -> Result<Value, Diagnostic> {
    use Type::*;

    let (l, r) = (left.kind, right.kind);
    let (a, b) = (&left.number, &right.number);
    let vector = l == r && matches!(l, Integer | Time | Distance);
    let point = l == r && matches!(l, Instant | Position);
    let mismatch = || {
        let (code, hint) = if point_vector_pair(l, r) {
            (
                "K307",
                " 시각과 지속 시간(위치와 거리)은 비교할 수 없습니다. 점끼리 빼서 차이를 구하세요.",
            )
        } else {
            ("K304", "")
        };
        op.error(
            code,
            format!(
                "'{}' 연산에 사용할 수 없는 조합입니다. 왼쪽은 {}, 오른쪽은 {}입니다.{hint}",
                op.text,
                scope.describe(left),
                scope.describe(right),
            ),
        )
    };
    let adds_points = || {
        op.error(
            "K305",
            format!(
                "'{}' 연산으로 {}과 {}을 계산할 수 없습니다. 기준 위의 점에는 차이(시간·거리)만 더하거나 뺄 수 있습니다.",
                op.text,
                scope.describe(left),
                scope.describe(right),
            ),
        )
    };

    let result = match op.kind {
        Kind::Plus => match (l, r) {
            _ if vector => Value::plain(l, a.add(b)),
            (Instant, Time) | (Position, Distance) => Value::point(l, left.frame, a.add(b)),
            (Time, Instant) | (Distance, Position) => Value::point(r, right.frame, a.add(b)),
            _ if point => return Err(adds_points()),
            _ => return Err(mismatch()),
        },
        Kind::Minus => match (l, r) {
            _ if vector => Value::plain(l, a.sub(b)),
            (Instant, Time) | (Position, Distance) => Value::point(l, left.frame, a.sub(b)),
            _ if point => {
                same_frame(op, left, right, scope)?;
                Value::plain(l.displacement().unwrap_or(l), a.sub(b))
            }
            (Time, Instant) | (Distance, Position) => return Err(adds_points()),
            _ => return Err(mismatch()),
        },
        Kind::Star => match (l, r) {
            (Integer, Integer | Time | Distance) => Value::plain(r, a.mul(b)),
            (Time | Distance, Integer) => Value::plain(l, a.mul(b)),
            _ => return Err(mismatch()),
        },
        Kind::Less | Kind::LessEqual | Kind::Greater | Kind::GreaterEqual => {
            if point {
                same_frame(op, left, right, scope)?;
            } else if !vector {
                return Err(mismatch());
            }
            Value::logical(match op.kind {
                Kind::Less => a < b,
                Kind::LessEqual => a <= b,
                Kind::Greater => a > b,
                _ => a >= b,
            })
        }
        Kind::Equal | Kind::NotEqual => {
            if l != r {
                return Err(mismatch());
            }
            same_frame(op, left, right, scope)?;
            Value::logical((a == b) == (op.kind == Kind::Equal))
        }
        Kind::And if l == Boolean && r == Boolean => Value::logical(!a.is_zero() && !b.is_zero()),
        Kind::Or if l == Boolean && r == Boolean => Value::logical(!a.is_zero() || !b.is_zero()),
        _ => return Err(mismatch()),
    };
    Ok(result)
}
