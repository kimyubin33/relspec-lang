mod integer;
mod parser;
mod sentence;
mod solver;
mod syntax;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;

use integer::Integer;
use serde::Serialize;
use solver::{Closure, ORIGIN};
use syntax::{Axis, Expression, Kind, Role, Statement, Token, Type};

pub const LANGUAGE_VERSION: &str = "0.4";

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
pub struct IrUnknown {
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    /// 모든 조건을 만족하는 해에서의 최솟값. `null`이면 하한이 없습니다.
    pub min: Option<String>,
    /// 모든 조건을 만족하는 해에서의 최댓값. `null`이면 상한이 없습니다.
    pub max: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed: Option<String>,
}

#[derive(Serialize)]
pub struct IrConstraint {
    /// `조건` 또는 `관측`
    pub kind: &'static str,
    pub name: String,
    pub expression: IrExpression,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrGoal {
    pub name: String,
    /// `보장`(모든 해에서 성립), `가능`(일부 해에서 성립), `불가능`(어떤 해에서도 성립하지 않음)
    pub status: &'static str,
    /// `가능`일 때 판정을 확정하려면 관측해야 하는 미지값.
    pub missing_observations: Vec<String>,
    pub expression: IrExpression,
}

#[derive(Serialize)]
pub struct IrVerb {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub forms: Vec<String>,
    pub required: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct IrReference {
    pub number: u32,
    pub text: String,
}

#[derive(Serialize)]
pub struct IrSentenceRole {
    pub role: String,
    pub particle: String,
    #[serde(rename = "type")]
    pub value_type: &'static str,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<String>,
    /// 지시어(이렇게, 이것)가 가리키는 참조 번호.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<u32>,
}

#[derive(Serialize)]
pub struct IrSentence {
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// 행위의 기본형
    pub verb: String,
    /// 문장에 쓰인 활용형
    pub form: String,
    /// 문장이 참인 정도(퍼센트). 적지 않으면 100입니다.
    pub degree: u32,
    pub roles: Vec<IrSentenceRole>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<u32>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unknowns: Vec<IrUnknown>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<IrConstraint>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<IrGoal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verbs: Vec<IrVerb>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sentences: Vec<IrSentence>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<IrReference>,
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
        unknowns: Vec::new(),
        constraints: Vec::new(),
        goals: Vec::new(),
        verbs: Vec::new(),
        sentences: Vec::new(),
        references: Vec::new(),
    };
    let mut pending = Vec::new();
    let mut verbs: Vec<sentence::Verb> = Vec::new();
    let mut citations: Vec<(u32, Token)> = Vec::new();

    for statement in statements {
        let name = match &statement {
            Statement::Value { name, .. }
            | Statement::Frame { name, .. }
            | Statement::Entity { name }
            | Statement::Event { name, .. }
            | Statement::Unknown { name, .. }
            | Statement::Constraint { name, .. } => Some(name),
            Statement::Sentence { label, .. } => label.as_ref(),
            // 관측은 새 이름을 만들지 않고 미지값에 값을 기록합니다.
            // 행위와 참조는 값과 다른 이름 공간을 씁니다.
            Statement::Observe { .. }
            | Statement::Verb { .. }
            | Statement::ReferenceDefinition { .. } => None,
        };
        if let Some(name) = name
            && !names.insert(name.text.clone())
        {
            return Err(name.error(
                "K301",
                format!("'{}' 이름이 이미 선언되어 있습니다.", name.text),
            ));
        }

        match statement {
            Statement::Value {
                name,
                declared_type: None,
                expression,
            } if mentions_unknown(&expression, &scope) => {
                // 미지값이 들어간 검사는 모든 해에서 보장되어야 하며, 판정은 모든 조건을 읽은 뒤에 합니다.
                let judged = constrain(expression, &scope)?;
                check_declared_type(&name, Type::Boolean, &judged.value, &scope)?;
                pending.push(Pending {
                    name,
                    is_check: true,
                    judged,
                });
            }
            Statement::Value {
                name,
                declared_type,
                expression,
            } => {
                let Bound { value, ir } = bind(expression, &scope)?;
                if value.is_symbolic() {
                    return Err(name.error(
                        "K406",
                        format!(
                            "정의 '{}'에 미지값이 들어 있습니다. 정의에는 값이 확정된 식만 쓸 수 있습니다.",
                            name.text
                        ),
                    ));
                }
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
                    terms: BTreeMap::new(),
                };
                scope.values.insert(name.text.clone(), value);
                program.entities.push(IrEntity { name: name.text });
            }
            Statement::Event { name, roles } => {
                let (record, ir) = bind_event(&name, roles, &scope)?;
                scope.events.insert(name.text, record);
                program.events.push(ir);
            }
            Statement::Unknown { name, kind, frame } => {
                let frame = match frame {
                    None => None,
                    Some(token) => {
                        let Some(&id) = scope.frame_ids.get(&token.text) else {
                            return Err(token.error(
                                "K308",
                                format!(
                                    "'@' 앞에는 '기준'으로 선언한 이름이 필요합니다. '{}'은 기준이 아닙니다.",
                                    token.text
                                ),
                            ));
                        };
                        if scope.frames[id].1.point() != kind {
                            return Err(token.error(
                                "K308",
                                format!(
                                    "{} 기준 '{}'에는 {}을 놓을 수 없습니다.",
                                    scope.frames[id].1.name(),
                                    token.text,
                                    kind.name(),
                                ),
                            ));
                        }
                        Some(id)
                    }
                };
                let node = scope.closure.add_node();
                let mut value = Value::point(kind, frame, Integer::from_small(0));
                value.terms.insert(node, Integer::from_small(1));
                scope.values.insert(name.text.clone(), value);
                scope.unknown_ids.insert(name.text.clone(), node);
                scope.unknowns.push(UnknownInfo {
                    name: name.text,
                    kind,
                    frame,
                    observed: None,
                });
            }
            Statement::Constraint {
                name,
                goal,
                expression,
            } => {
                let judged = constrain(expression, &scope)?;
                check_declared_type(&name, Type::Boolean, &judged.value, &scope)?;
                if goal {
                    pending.push(Pending {
                        name,
                        is_check: false,
                        judged,
                    });
                    continue;
                }
                if !judged.fixed {
                    return Err(
                        name.error("K404", format!("조건 '{}'은 항상 거짓입니다.", name.text))
                    );
                }
                for atom in &judged.atoms {
                    assume(&mut scope, atom, &name, "조건")?;
                }
                program.constraints.push(IrConstraint {
                    kind: "조건",
                    name: name.text,
                    expression: judged.ir,
                });
            }
            Statement::Verb {
                name,
                forms,
                required,
            } => {
                if verbs.iter().any(|v| v.name == name.text) {
                    return Err(name.error(
                        "K301",
                        format!("행위 '{}'이 이미 선언되어 있습니다.", name.text),
                    ));
                }
                let verb = sentence::Verb::new(&name, &forms, &required)?;
                program.verbs.push(IrVerb {
                    name: name.text,
                    forms: forms.into_iter().map(|f| f.text).collect(),
                    required: verb.required.clone(),
                });
                verbs.push(verb);
            }
            Statement::ReferenceDefinition { number, text } => {
                let Ok(n) = number.text.parse::<u32>() else {
                    return Err(number.error("K511", "참조 번호가 너무 큽니다."));
                };
                if program.references.iter().any(|r| r.number == n) {
                    return Err(number.error("K512", format!("참조 {n}이 이미 정의되어 있습니다.")));
                }
                program.references.push(IrReference {
                    number: n,
                    text: text.text,
                });
            }
            Statement::Sentence { label, tokens } => {
                let bound = sentence::bind_sentence(label.as_ref(), &tokens, &scope, &verbs)?;
                citations.extend(bound.citations);
                if let Some(label) = label {
                    scope.values.insert(
                        label.text,
                        Value::plain(Type::Degree, Integer::from_small(bound.degree)),
                    );
                }
                program.sentences.push(bound.ir);
            }
            Statement::Observe { name, expression } => {
                let Some(&node) = scope.unknown_ids.get(&name.text) else {
                    if scope.values.contains_key(&name.text) {
                        return Err(name.error(
                            "K407",
                            format!("'{}'은 미지값이 아니므로 관측할 수 없습니다.", name.text),
                        ));
                    }
                    return Err(unbound(&name, &scope));
                };
                let info = &scope.unknowns[node - 1];
                if info.observed.is_some() {
                    return Err(name.error(
                        "K407",
                        format!("미지값 '{}'은 이미 관측되었습니다.", name.text),
                    ));
                }
                let Bound { value, ir } = bind(expression, &scope)?;
                if value.is_symbolic() {
                    return Err(name.error(
                        "K406",
                        format!("'{}'의 관측값에는 미지값을 쓸 수 없습니다.", name.text),
                    ));
                }
                check_declared_type(&name, info.kind, &value, &scope)?;
                if value.frame != info.frame {
                    let expected = Value::point(info.kind, info.frame, Integer::from_small(0));
                    return Err(name.error(
                        "K306",
                        format!(
                            "미지값 '{}'은 {}인데 관측값은 {}입니다.",
                            name.text,
                            scope.describe(&expected),
                            scope.describe(&value),
                        ),
                    ));
                }
                let upper = Atom {
                    from: ORIGIN,
                    to: node,
                    bound: value.number.clone(),
                };
                let lower = Atom {
                    from: node,
                    to: ORIGIN,
                    bound: value.number.clone().negate(),
                };
                for atom in [upper, lower] {
                    assume(&mut scope, &atom, &name, "관측")?;
                }
                scope.unknowns[node - 1].observed = Some(value.number);
                program.constraints.push(IrConstraint {
                    kind: "관측",
                    name: name.text,
                    expression: ir,
                });
            }
        }
    }

    for (number, token) in &citations {
        if !program.references.iter().any(|r| r.number == *number) {
            return Err(token.error(
                "K511",
                format!("참조 {number}이 정의되지 않았습니다. '참조 {number}: 내용'을 적으세요."),
            ));
        }
    }

    for Pending {
        name,
        is_check,
        judged,
    } in pending
    {
        let (status, missing) = judge(&scope, &judged);
        if is_check {
            if status != "보장" {
                let reason = if status == "불가능" {
                    "어떤 해에서도 성립하지 않습니다".to_string()
                } else if missing.is_empty() {
                    "일부 해에서만 성립합니다".to_string()
                } else {
                    format!(
                        "일부 해에서만 성립합니다. 보장 여부를 확정하려면 관측이 필요합니다: {}",
                        missing.join(", ")
                    )
                };
                return Err(name.error(
                    "K403",
                    format!(
                        "검사 '{}'은 모든 해에서 보장되지 않습니다: {reason}.",
                        name.text
                    ),
                ));
            }
            program.statements.push(IrStatement {
                kind: "검사",
                name: name.text,
                value_type: Type::Boolean.name(),
                value: "참".into(),
                unit: None,
                frame: None,
                expression: judged.ir,
            });
        } else {
            program.goals.push(IrGoal {
                name: name.text,
                status,
                missing_observations: missing,
                expression: judged.ir,
            });
        }
    }

    for (index, info) in scope.unknowns.iter().enumerate() {
        let node = index + 1;
        let probe = Value::point(info.kind, info.frame, Integer::from_small(0));
        program.unknowns.push(IrUnknown {
            name: info.name.clone(),
            value_type: info.kind.name(),
            unit: info.kind.base_unit(),
            frame: scope.frame_name(&probe),
            min: scope
                .closure
                .upper(node, ORIGIN)
                .map(|bound| bound.clone().negate().to_string()),
            max: scope.closure.upper(ORIGIN, node).map(ToString::to_string),
            observed: info.observed.as_ref().map(ToString::to_string),
        });
    }

    Ok(program)
}

/// 미지값 하나 또는 두 미지값의 차이에 대한 상한: `to - from <= bound`.
/// 꼭짓점 `ORIGIN`은 값 0(기준 원점)입니다.
struct Atom {
    from: usize,
    to: usize,
    bound: Integer,
}

/// 조건·목표·검사의 식을 비교식들의 '그리고'로 풀어 둔 결과.
struct Judged {
    /// 타입 확인용 값. 미지값이 없으면 실제 결과입니다.
    value: Value,
    /// 미지값이 없는 부분들의 결과를 모두 '그리고'로 묶은 값.
    fixed: bool,
    atoms: Vec<Atom>,
    ir: IrExpression,
}

struct Pending {
    name: Token,
    is_check: bool,
    judged: Judged,
}

struct UnknownInfo {
    name: String,
    kind: Type,
    frame: Option<usize>,
    observed: Option<Integer>,
}

fn mentions_unknown(expression: &Expression, scope: &Scope) -> bool {
    match expression {
        Expression::Reference(name) => scope.unknown_ids.contains_key(&name.text),
        Expression::Unary(_, operand) | Expression::Anchor(_, _, operand) => {
            mentions_unknown(operand, scope)
        }
        Expression::Binary(_, left, right) => {
            mentions_unknown(left, scope) || mentions_unknown(right, scope)
        }
        Expression::Literal(..) | Expression::Member(..) => false,
    }
}

fn constrain(expression: Expression, scope: &Scope) -> Result<Judged, Diagnostic> {
    match expression {
        Expression::Binary(op, left, right) if op.kind == Kind::And => {
            let left = constrain(*left, scope)?;
            let right = constrain(*right, scope)?;
            let value = evaluate_binary(&op, &left.value, &right.value, scope)?;
            let mut atoms = left.atoms;
            atoms.extend(right.atoms);
            Ok(Judged {
                value,
                fixed: left.fixed && right.fixed,
                atoms,
                ir: IrExpression::Binary {
                    kind: Type::Boolean.name(),
                    operator: op.text,
                    operands: [Box::new(left.ir), Box::new(right.ir)],
                },
            })
        }
        Expression::Binary(op, left, right)
            if matches!(
                op.kind,
                Kind::Less
                    | Kind::LessEqual
                    | Kind::Greater
                    | Kind::GreaterEqual
                    | Kind::Equal
                    | Kind::NotEqual
            ) =>
        {
            let left = bind(*left, scope)?;
            let right = bind(*right, scope)?;
            // 타입과 기준은 미지값을 뗀 값으로 확인합니다.
            let value = evaluate_binary(&op, &left.value.fixed(), &right.value.fixed(), scope)?;
            let symbolic = left.value.is_symbolic() || right.value.is_symbolic();
            let ir = IrExpression::Binary {
                kind: Type::Boolean.name(),
                operator: op.text.clone(),
                operands: [Box::new(left.ir), Box::new(right.ir)],
            };
            if !symbolic {
                let fixed = !value.number.is_zero();
                return Ok(Judged {
                    value,
                    fixed,
                    atoms: Vec::new(),
                    ir,
                });
            }

            let difference = left.value.minus(&right.value);
            let one = Integer::from_small(1);
            let mut atoms = Vec::new();
            match op.kind {
                Kind::LessEqual => atoms.push(atom(&op, &difference, Integer::from_small(0))?),
                Kind::Less => atoms.push(atom(&op, &difference, one.negate())?),
                Kind::GreaterEqual => {
                    atoms.push(atom(&op, &difference.negated(), Integer::from_small(0))?)
                }
                Kind::Greater => atoms.push(atom(&op, &difference.negated(), one.negate())?),
                Kind::Equal => {
                    atoms.push(atom(&op, &difference, Integer::from_small(0))?);
                    atoms.push(atom(&op, &difference.negated(), Integer::from_small(0))?);
                }
                _ => return Err(unsupported(&op)),
            }
            Ok(Judged {
                value,
                fixed: true,
                atoms,
                ir,
            })
        }
        expression => {
            let Bound { value, ir } = bind(expression, scope)?;
            let fixed = value.kind == Type::Boolean && !value.number.is_zero();
            Ok(Judged {
                value,
                fixed,
                atoms: Vec::new(),
                ir,
            })
        }
    }
}

fn unsupported(op: &Token) -> Diagnostic {
    op.error(
        "K405",
        format!(
            "미지값이 들어간 '{}' 비교는 지원하지 않습니다. 미지값은 '그리고'로 묶은 <, <=, >, >=, == 비교에서 쓸 수 있습니다.",
            op.text
        ),
    )
}

/// `expression <= limit`을 `to - from <= bound` 꼴로 바꿉니다.
fn atom(op: &Token, expression: &Value, limit: Integer) -> Result<Atom, Diagnostic> {
    let bound = limit.sub(&expression.number);
    let one = Integer::from_small(1);
    let minus_one = one.clone().negate();
    let terms: Vec<(usize, Integer)> = expression
        .terms
        .iter()
        .map(|(&node, c)| (node, c.clone()))
        .collect();
    let (from, to) = match terms.as_slice() {
        [(x, c)] if *c == one => (ORIGIN, *x),
        [(x, c)] if *c == minus_one => (*x, ORIGIN),
        [(x, a), (y, b)] if *a == one && *b == minus_one => (*y, *x),
        [(x, a), (y, b)] if *a == minus_one && *b == one => (*x, *y),
        [] => unreachable!("상수 비교는 미지값 없는 경로에서 처리합니다"),
        _ => {
            return Err(op.error(
                "K405",
                "지원하는 제약은 미지값 하나의 상·하한 또는 두 미지값의 차이(가 - 나)에 대한 상·하한입니다.",
            ));
        }
    };
    Ok(Atom { from, to, bound })
}

/// 조건이나 관측을 해 집합에 더합니다. 모순이면 앞선 조건이 정한 범위와 함께 보고합니다.
fn assume(scope: &mut Scope, atom: &Atom, name: &Token, label: &str) -> Result<(), Diagnostic> {
    let Err(back) = scope.closure.add(atom.from, atom.to, &atom.bound) else {
        return Ok(());
    };
    // 앞선 조건: from - to <= back, 즉 to - from >= -back. 새 조건: to - from <= bound.
    let (subject, earlier, requested) = if atom.to == ORIGIN {
        // to - from = -x 이므로 부호를 뒤집어 x에 대해 말합니다.
        let x = atom.from;
        (
            scope.unknowns[x - 1].name.clone(),
            format!("최대 {}", scope.quantity(x, true, &back)),
            format!(
                "최소 {}",
                scope.quantity(x, true, &atom.bound.clone().negate())
            ),
        )
    } else if atom.from == ORIGIN {
        let x = atom.to;
        (
            scope.unknowns[x - 1].name.clone(),
            format!("최소 {}", scope.quantity(x, true, &back.clone().negate())),
            format!("최대 {}", scope.quantity(x, true, &atom.bound)),
        )
    } else {
        (
            format!(
                "{} - {}",
                scope.unknowns[atom.to - 1].name,
                scope.unknowns[atom.from - 1].name
            ),
            format!(
                "최소 {}",
                scope.quantity(atom.to, false, &back.clone().negate())
            ),
            format!("최대 {}", scope.quantity(atom.to, false, &atom.bound)),
        )
    };
    Err(name.error(
        "K404",
        format!(
            "{label} '{}'이 앞선 조건·관측과 모순됩니다. 앞선 조건: '{subject}' {earlier}. 이 {label}: '{subject}' {requested}.",
            name.text
        ),
    ))
}

/// 모든 조건 아래에서 목표가 보장·가능·불가능 중 무엇인지 판정합니다.
fn judge(scope: &Scope, judged: &Judged) -> (&'static str, Vec<String>) {
    if !judged.fixed {
        return ("불가능", Vec::new());
    }
    let entailed = judged.atoms.iter().all(|atom| {
        scope
            .closure
            .upper(atom.from, atom.to)
            .is_some_and(|upper| *upper <= atom.bound)
    });
    if entailed {
        return ("보장", Vec::new());
    }
    let mut trial = scope.closure.clone();
    if judged
        .atoms
        .iter()
        .any(|atom| trial.add(atom.from, atom.to, &atom.bound).is_err())
    {
        return ("불가능", Vec::new());
    }
    let mut nodes: Vec<usize> = judged
        .atoms
        .iter()
        .flat_map(|atom| [atom.from, atom.to])
        .filter(|&node| node != ORIGIN && scope.unknowns[node - 1].observed.is_none())
        .collect();
    nodes.sort_unstable();
    nodes.dedup();
    let missing = nodes
        .into_iter()
        .map(|node| scope.unknowns[node - 1].name.clone())
        .collect();
    ("가능", missing)
}

#[derive(Default)]
struct Scope {
    values: HashMap<String, Value>,
    frame_ids: HashMap<String, usize>,
    frames: Vec<(String, Axis)>,
    events: HashMap<String, EventRecord>,
    unknown_ids: HashMap<String, usize>,
    /// 꼭짓점 `n`의 미지값은 `unknowns[n - 1]`입니다.
    unknowns: Vec<UnknownInfo>,
    closure: Closure,
}

impl Default for Closure {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    fn frame_name(&self, value: &Value) -> Option<String> {
        value.frame.map(|id| self.frames[id].0.clone())
    }

    /// 미지값 `node`에 관한 양을 사람이 읽을 수 있게 씁니다.
    /// `absolute`이면 미지값 자체의 값, 아니면 두 미지값의 차이입니다.
    fn quantity(&self, node: usize, absolute: bool, number: &Integer) -> String {
        let info = &self.unknowns[node - 1];
        let kind = if absolute {
            info.kind
        } else {
            info.kind.displacement().unwrap_or(info.kind)
        };
        let amount = match kind.base_unit() {
            Some(unit) => format!("{number} {unit}"),
            None => number.to_string(),
        };
        match (absolute, info.frame) {
            (true, Some(id)) => format!("{} @ {amount}", self.frames[id].0),
            _ => amount,
        }
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
    /// 미지값 꼭짓점별 계수. 값은 `number + Σ 계수 × 미지값`입니다.
    terms: BTreeMap<usize, Integer>,
}

impl Value {
    fn plain(kind: Type, number: Integer) -> Self {
        Self {
            kind,
            frame: None,
            number,
            label: None,
            terms: BTreeMap::new(),
        }
    }

    fn point(kind: Type, frame: Option<usize>, number: Integer) -> Self {
        Self {
            kind,
            frame,
            number,
            label: None,
            terms: BTreeMap::new(),
        }
    }

    fn is_symbolic(&self) -> bool {
        !self.terms.is_empty()
    }

    /// 미지값 부분을 뗀 값. 타입·기준 확인에 씁니다.
    fn fixed(&self) -> Self {
        Self {
            terms: BTreeMap::new(),
            ..self.clone()
        }
    }

    fn negated(&self) -> Self {
        Self {
            number: self.number.clone().negate(),
            terms: self
                .terms
                .iter()
                .map(|(&node, c)| (node, c.clone().negate()))
                .collect(),
            ..self.clone()
        }
    }

    /// 계수를 포함한 뺄셈. 결과의 타입은 호출한 쪽에서 정합니다.
    fn minus(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.number = self.number.sub(&other.number);
        result.terms = combine(&self.terms, &other.terms, false);
        result
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
            Type::Degree => format!("{}%", self.number),
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
        if value.is_symbolic() {
            return Err(role.error(
                "K406",
                format!(
                    "사건 '{}'의 '{}' 역할에 미지값이 들어 있습니다. 사건 역할에는 값이 확정된 식만 쓸 수 있습니다.",
                    name.text, role_name
                ),
            ));
        }
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
            let mut value = Value::point(axis.point(), Some(id), operand.value.number);
            value.terms = operand.value.terms;
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
            Ok(value.negated())
        }
        Kind::Not if value.kind == Type::Boolean => Ok(Value::logical(value.number.is_zero())),
        // 퍼지 부정: 100% - 정도
        Kind::Not if value.kind == Type::Degree => Ok(Value::plain(
            Type::Degree,
            Integer::from_small(100).sub(&value.number),
        )),
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

fn as_degree(value: &Value) -> Integer {
    if value.kind == Type::Boolean {
        Integer::from_small(if value.number.is_zero() { 0 } else { 100 })
    } else {
        value.number.clone()
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

fn combine(
    left: &BTreeMap<usize, Integer>,
    right: &BTreeMap<usize, Integer>,
    add: bool,
) -> BTreeMap<usize, Integer> {
    let mut terms = left.clone();
    for (&node, coefficient) in right {
        let current = terms
            .remove(&node)
            .unwrap_or_else(|| Integer::from_small(0));
        let next = if add {
            current.add(coefficient)
        } else {
            current.sub(coefficient)
        };
        if !next.is_zero() {
            terms.insert(node, next);
        }
    }
    terms
}

fn scale(terms: &BTreeMap<usize, Integer>, factor: &Integer) -> BTreeMap<usize, Integer> {
    if factor.is_zero() {
        return BTreeMap::new();
    }
    terms
        .iter()
        .map(|(&node, c)| (node, c.mul(factor)))
        .collect()
}

/// 미지값 계수까지 함께 계산합니다. 비교·논리 연산의 결과는 미지값을 담을 수 없습니다.
fn evaluate_binary(
    op: &Token,
    left: &Value,
    right: &Value,
    scope: &Scope,
) -> Result<Value, Diagnostic> {
    let mut result = evaluate_fixed(op, &left.fixed(), &right.fixed(), scope)?;
    if !left.is_symbolic() && !right.is_symbolic() {
        return Ok(result);
    }
    result.terms = match op.kind {
        Kind::Plus => combine(&left.terms, &right.terms, true),
        Kind::Minus => combine(&left.terms, &right.terms, false),
        Kind::Star if left.is_symbolic() && right.is_symbolic() => {
            return Err(op.error(
                "K405",
                "미지값끼리 곱할 수 없습니다. 제약은 미지값에 대해 일차식이어야 합니다.",
            ));
        }
        Kind::Star if left.is_symbolic() => scale(&left.terms, &right.number),
        Kind::Star => scale(&right.terms, &left.number),
        _ => return Err(unsupported(op)),
    };
    Ok(result)
}

fn evaluate_fixed(
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
            } else if !(vector || (l == r && l == Degree)) {
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
        // 퍼지 논리(Zadeh): 그리고 = 최솟값, 또는 = 최댓값. 논리값은 참 100%, 거짓 0%로 봅니다.
        Kind::And | Kind::Or
            if matches!(l, Boolean | Degree)
                && matches!(r, Boolean | Degree)
                && (l == Degree || r == Degree) =>
        {
            let (x, y) = (as_degree(left), as_degree(right));
            let pick_left = if op.kind == Kind::And { x <= y } else { x >= y };
            Value::plain(Degree, if pick_left { x } else { y })
        }
        Kind::And if l == Boolean && r == Boolean => Value::logical(!a.is_zero() && !b.is_zero()),
        Kind::Or if l == Boolean && r == Boolean => Value::logical(!a.is_zero() || !b.is_zero()),
        _ => return Err(mismatch()),
    };
    Ok(result)
}
