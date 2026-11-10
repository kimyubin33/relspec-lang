//! 0.3 미지값과 제약: 모순·가능·보장의 구분.

use kpl::{CompiledProgram, compile};

fn ok(source: &str) -> CompiledProgram {
    compile(source).unwrap_or_else(|error| panic!("{error}"))
}

fn fails(source: &str) -> (&'static str, usize) {
    let diagnostic = compile(source)
        .err()
        .unwrap_or_else(|| panic!("expected a diagnostic for:\n{source}"));
    (diagnostic.code, diagnostic.line)
}

fn goal<'a>(program: &'a CompiledProgram, name: &str) -> (&'a str, Vec<String>) {
    let goal = program
        .goals
        .iter()
        .find(|goal| goal.name == name)
        .expect("goal");
    (goal.status, goal.missing_observations.clone())
}

/// DEVELOPMENT.md의 예: 시작 1000 밀리초, 반응 지연 최대 200 밀리초.
const REACTION: &str = "기준 실험시계: 시간축\n\
    사건 자극: 시작 = 실험시계 @ 1000 밀리초\n\
    미지 반응: 시각 = 실험시계 @ ?\n\
    조건 뒤 = 반응 >= 자극.시작\n\
    조건 지연 = 반응 - 자극.시작 <= 200 밀리초\n\
    목표 마감 = 반응 <= 실험시계 @ 1100 밀리초\n";

#[test]
fn separates_possible_guaranteed_and_impossible() {
    let program = ok(&format!(
        "{REACTION}목표 넉넉 = 반응 <= 실험시계 @ 1200 밀리초\n\
         목표 이름 = 반응 < 자극.시작\n\
         목표 둘다 = 반응 >= 실험시계 @ 1100 밀리초 그리고 반응 <= 실험시계 @ 1100 밀리초"
    ));
    assert_eq!(goal(&program, "마감"), ("가능", vec!["반응".to_string()]));
    assert_eq!(goal(&program, "넉넉").0, "보장");
    assert_eq!(goal(&program, "이름").0, "불가능");
    assert_eq!(goal(&program, "둘다").0, "가능");

    let unknown = &program.unknowns[0];
    assert_eq!(unknown.min.as_deref(), Some("1000"));
    assert_eq!(unknown.max.as_deref(), Some("1200"));
    assert_eq!(unknown.frame.as_deref(), Some("실험시계"));

    let ir = serde_json::to_value(&program).unwrap();
    assert_eq!(ir["languageVersion"], kpl::LANGUAGE_VERSION);
    assert_eq!(ir["goals"][0]["missingObservations"][0], "반응");
    assert_eq!(ir["constraints"][1]["kind"], "조건");
}

#[test]
fn observations_decide_goals_and_conflict_with_rules() {
    let observed = ok(&format!("{REACTION}관측 반응 = 실험시계 @ 1080 밀리초"));
    assert_eq!(goal(&observed, "마감"), ("보장", vec![]));
    assert_eq!(observed.unknowns[0].observed.as_deref(), Some("1080"));

    let late = ok(&format!("{REACTION}관측 반응 = 실험시계 @ 1150 밀리초"));
    assert_eq!(goal(&late, "마감").0, "불가능");

    // 규칙(지연 최대 200 밀리초)과 관측이 충돌합니다.
    let conflict = format!("{REACTION}관측 반응 = 실험시계 @ 1300 밀리초");
    assert_eq!(fails(&conflict), ("K404", 7));
}

#[test]
fn contradictory_conditions_are_reported_at_the_later_condition() {
    let source = "미지 가: 시간 = ?\n\
                  미지 나: 시간 = ?\n\
                  조건 하나 = 가 - 나 >= 5 초\n\
                  조건 둘 = 나 - 가 >= 1 초";
    let diagnostic = compile(source).err().expect("contradiction");
    assert_eq!((diagnostic.code, diagnostic.line), ("K404", 4));
    assert!(
        diagnostic.message.contains("최소 5000 밀리초"),
        "{}",
        diagnostic.message
    );

    assert_eq!(fails("조건 항상틀림 = 1 > 2"), ("K404", 1));
}

#[test]
fn chained_differences_propagate_bounds() {
    let program = ok("기준 시계: 시간축\n\
         미지 가: 시각 = 시계 @ ?\n\
         미지 나: 시각 = 시계 @ ?\n\
         미지 다: 시각 = 시계 @ ?\n\
         조건 시작 = 가 == 시계 @ 0 초\n\
         조건 첫구간 = 나 - 가 <= 3 초 그리고 나 > 가\n\
         조건 둘째구간 = 다 - 나 <= 4 초 그리고 다 - 나 >= 1 초\n\
         목표 전체 = 다 - 가 <= 7 초\n\
         목표 빠듯 = 다 - 가 <= 6 초");
    assert_eq!(goal(&program, "전체").0, "보장");
    assert_eq!(
        goal(&program, "빠듯"),
        ("가능", vec!["가".into(), "다".into()])
    );
    // `나 > 가`는 정수 밀리초에서 `나 - 가 >= 1`입니다.
    let c = &program.unknowns[2];
    assert_eq!(
        (c.min.as_deref(), c.max.as_deref()),
        (Some("1001"), Some("7000"))
    );
}

#[test]
fn unbounded_unknowns_and_other_types() {
    let program = ok("기준 복도: 위치축\n\
         미지 위치값: 위치 = 복도 @ ?\n\
         미지 횟수: 정수 = ?\n\
         조건 앞쪽 = 위치값 >= 복도 @ 2 미터\n\
         조건 횟수상한 = 2 * 3 >= 횟수\n\
         목표 횟수제한 = 횟수 <= 6");
    let position = &program.unknowns[0];
    assert_eq!(
        (position.min.as_deref(), position.max.as_deref()),
        (Some("2000"), None)
    );
    assert_eq!(position.unit, Some("밀리미터"));
    assert_eq!(program.unknowns[1].min, None);
    assert_eq!(goal(&program, "횟수제한").0, "보장");
}

#[test]
fn checks_with_unknowns_must_be_guaranteed() {
    let passing = ok(&format!(
        "{REACTION}검사 상한 = 반응 <= 실험시계 @ 1200 밀리초"
    ));
    let check = passing.statements.last().unwrap();
    assert_eq!((check.kind, check.value.as_str()), ("검사", "참"));

    let only_possible = format!("{REACTION}검사 엄격 = 반응 <= 실험시계 @ 1100 밀리초");
    let diagnostic = compile(&only_possible).err().expect("not guaranteed");
    assert_eq!(diagnostic.code, "K403");
    assert!(
        diagnostic.message.contains("반응"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn rejects_constraints_outside_difference_logic() {
    let base = "미지 가: 시간 = ?\n미지 나: 시간 = ?\n미지 다: 시간 = ?\n";
    for line in [
        "조건 같지않음 = 가 != 1 초",
        "조건 둘중하나 = 가 < 1 초 또는 나 < 1 초",
        "조건 셋 = 가 + 나 - 다 <= 1 초",
        "조건 합 = 가 + 나 <= 1 초",
        "조건 배수 = 2 * 가 <= 1 초",
    ] {
        assert_eq!(fails(&format!("{base}{line}")), ("K405", 4), "{line}");
    }
    // 시간 × 시간은 타입 오류가 먼저이고, 정수 미지값끼리의 곱은 비선형이라 K405입니다.
    assert_eq!(
        fails(&format!("{base}조건 곱 = 가 * 나 <= 1 초")),
        ("K304", 4)
    );
    assert_eq!(
        fails("미지 가: 정수 = ?\n미지 나: 정수 = ?\n조건 곱 = 가 * 나 <= 1"),
        ("K405", 3)
    );
}

#[test]
fn unknowns_stay_out_of_fixed_values() {
    let base = "기준 시계: 시간축\n미지 가: 시각 = 시계 @ ?\n";
    assert_eq!(
        fails(&format!("{base}정의 나: 시각 = 가 + 1 초")),
        ("K406", 3)
    );
    // 0.5부터 사건 역할에는 미지값을 쓸 수 있습니다(tests/linking.rs).
    assert_eq!(fails(&format!("{base}관측 가 = 가")), ("K406", 3));
}

#[test]
fn observation_rules() {
    let base = "기준 시계: 시간축\n기준 영상: 시간축\n미지 가: 시각 = 시계 @ ?\n";
    assert_eq!(
        fails(&format!(
            "{base}관측 가 = 시계 @ 1 초\n관측 가 = 시계 @ 1 초"
        )),
        ("K407", 5)
    );
    assert_eq!(
        fails(&format!("{base}정의 나: 정수 = 1\n관측 나 = 1")),
        ("K407", 5)
    );
    assert_eq!(fails(&format!("{base}관측 가 = 영상 @ 1 초")), ("K306", 4));
    assert_eq!(fails(&format!("{base}관측 가 = 1 초")), ("K307", 4));
    assert_eq!(fails(&format!("{base}관측 없음 = 1 초")), ("K302", 4));
}

#[test]
fn unknown_declarations_need_the_right_form() {
    assert_eq!(fails("미지 가: 시각 = ?"), ("K200", 1));
    assert_eq!(
        fails("기준 시계: 시간축\n미지 가: 시간 = 시계 @ ?"),
        ("K200", 2)
    );
    assert_eq!(
        fails("기준 복도: 위치축\n미지 가: 시각 = 복도 @ ?"),
        ("K308", 2)
    );
    assert_eq!(fails("미지 가: 논리 = ?"), ("K200", 1));
    assert_eq!(
        fails("기준 시계: 시간축\n미지 가: 시각 = 시계 @ ?\n조건 나 = 가 > 시계"),
        ("K309", 3)
    );
}

#[test]
fn reaction_example_ships() {
    let source = std::fs::read_to_string("examples/반응마감.kpl").unwrap();
    let program = ok(&source);
    assert_eq!(goal(&program, "마감준수").0, "가능");
    assert_eq!(goal(&program, "넉넉한마감").0, "보장");
    assert_eq!(goal(&program, "너무이른마감").0, "불가능");
}
