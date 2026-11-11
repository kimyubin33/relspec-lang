//! 0.9 쓰기 쉽게: 여러 이름 선언, 이름 없는 검사·조건·목표, 타입 생략, 단위, 문장 옆 설명.

use kpl::{CompiledProgram, compile};

fn ok(source: &str) -> CompiledProgram {
    compile(source).unwrap_or_else(|error| panic!("{error}\n{source}"))
}

fn fails(source: &str) -> (&'static str, usize) {
    let diagnostic = compile(source)
        .err()
        .unwrap_or_else(|| panic!("expected a diagnostic for:\n{source}"));
    (diagnostic.code, diagnostic.line)
}

fn value<'a>(program: &'a CompiledProgram, name: &str) -> (&'a str, &'a str, Option<&'a str>) {
    let statement = program
        .statements
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no statement named {name}"));
    (statement.value_type, &statement.value, statement.unit)
}

#[test]
fn several_names_are_declared_on_one_line() {
    let program = ok("대상 조사자, 조사범위, 보고서\n\
         행위 확인하다, 작성하다 (작성했다): 주체, 대상\n\
         조사자가 조사범위를 확인한다\n\
         조사자가 보고서를 작성했다\n");
    assert_eq!(program.entities.len(), 3);
    assert_eq!(program.verbs.len(), 2);
    assert_eq!(program.verbs[0].required, program.verbs[1].required);
    assert_eq!(program.sentences[1].verb, "작성하다");

    // 쉼표로 나눈 이름도 저마다 한 번만 선언할 수 있습니다.
    assert_eq!(fails("대상 컵, 바닥, 컵\n"), ("K301", 1));
    assert_eq!(fails("대상 컵,\n"), ("K200", 1));
    // 필수 역할은 함께 선언한 모든 행위에 적용됩니다.
    assert_eq!(
        fails("대상 컵\n행위 깨다, 닦다: 주체, 대상\n컵이 닦는다\n"),
        ("K504", 3)
    );
}

#[test]
fn checks_conditions_and_goals_need_no_name() {
    let program = ok("정의 시작: 정수 = 1\n정의 끝: 정수 = 5\n\
         검사 끝 >= 시작\n\
         미지 값: 정수\n\
         조건 값 >= 시작\n조건 값 <= 끝\n\
         목표 값 >= 0\n목표 값 > 3\n목표 넘침 = 값 > 5\n");
    // 이름이 없으면 식을 그대로 이름으로 보여 줍니다.
    assert_eq!(value(&program, "끝 >= 시작").1, "참");
    assert_eq!(program.constraints[0].name, "값 >= 시작");
    let status = |name: &str| {
        program
            .goals
            .iter()
            .find(|g| g.name == name)
            .unwrap_or_else(|| panic!("no goal {name}"))
            .status
    };
    assert_eq!(status("값 >= 0"), "보장");
    assert_eq!(status("값 > 3"), "가능");
    assert_eq!(status("넘침"), "불가능");
    assert_eq!(program.unknowns[0].min.as_deref(), Some("1"));

    // 같은 식을 두 번 적어도 이름이 겹친다고 하지 않습니다.
    ok("정의 가: 정수 = 1\n검사 가 == 1\n검사 가 == 1\n");
    // 이름 없는 검사가 정의된 이름을 가리지 않습니다.
    ok("정의 켜짐: 논리 = 참\n검사 켜짐\n검사 켜짐 == 참\n");
    assert_eq!(fails("정의 가: 정수 = 1\n검사 가 == 2\n"), ("K401", 2));
    assert_eq!(
        fails("미지 값: 정수\n조건 값 > 1\n조건 값 < 1\n"),
        ("K404", 3)
    );
}

#[test]
fn definitions_infer_their_type_and_unknowns_need_no_question_mark() {
    let program = ok(
        "정의 횟수 = 3\n정의 마감 = 2 초\n정의 켜짐 = 참\n정의 확신 = 80%\n\
         미지 가, 나: 시간\n조건 가 >= 마감\n조건 나 - 가 >= 1 초\n",
    );
    assert_eq!(value(&program, "횟수").0, "정수");
    assert_eq!(value(&program, "마감"), ("시간", "2000", Some("밀리초")));
    assert_eq!(value(&program, "켜짐").0, "논리");
    assert_eq!(value(&program, "확신").0, "정도");
    assert_eq!(program.unknowns.len(), 2);
    assert_eq!(program.unknowns[1].min.as_deref(), Some("3000"));

    // 타입을 적으면 전처럼 확인합니다.
    assert_eq!(fails("정의 마감: 정수 = 2 초\n"), ("K303", 1));
    // 기준 위의 점은 어느 기준인지 적어야 합니다.
    assert_eq!(fails("기준 시계: 시간축\n미지 도착: 시각\n"), ("K200", 2));
    ok("기준 시계: 시간축\n미지 도착: 시각 = 시계 @ ?\n");
}

#[test]
fn declared_units_are_separate_types() {
    let program = ok("단위 원, 명\n\
         정의 최저가격 = 120만 원\n정의 최고가격 = 1억 2000만원\n정의 정원 = 30 명\n\
         정의 두배 = 최저가격 * 2\n\
         검사 최저가격 < 최고가격\n\
         미지 지불의사: 원\n조건 지불의사 >= 0 원\n목표 지불의사 >= 최저가격\n");
    assert_eq!(program.units, ["원", "명"]);
    assert_eq!(value(&program, "최저가격"), ("원", "1200000", Some("원")));
    assert_eq!(value(&program, "최고가격").1, "120000000");
    assert_eq!(value(&program, "정원"), ("명", "30", Some("명")));
    assert_eq!(value(&program, "두배").1, "2400000");
    assert_eq!(program.unknowns[0].min.as_deref(), Some("0"));
    assert_eq!(program.goals[0].status, "가능");

    // 단위가 다른 양은 더하거나 비교할 수 없고, 단위 없는 수와도 섞이지 않습니다.
    assert_eq!(fails("단위 원, 명\n정의 합 = 1 원 + 1 명\n"), ("K304", 2));
    assert_eq!(fails("단위 원\n검사 100 원 > 50\n"), ("K304", 2));
    // 선언하지 않은 낱말은 단위가 아닙니다.
    assert_eq!(fails("정의 가격 = 100 원\n"), ("K315", 1));
    assert_eq!(fails("미지 가격: 원\n"), ("K315", 1));
    assert_eq!(fails("단위 원\n대상 원\n"), ("K301", 2));
}

#[test]
fn large_number_words_and_longer_time_units() {
    let program = ok("정의 가 = 3만\n정의 나 = 1억 2000만\n정의 다 = 5천\n\
         정의 회의 = 30 분\n정의 수업 = 2 시간\n정의 기간 = 3 일\n정의 길 = 2 킬로미터\n\
         검사 수업 == 120 분\n검사 회의 + 90 분 == 수업\n");
    assert_eq!(value(&program, "가").1, "30000");
    assert_eq!(value(&program, "나").1, "120000000");
    assert_eq!(value(&program, "다").1, "5000");
    assert_eq!(value(&program, "회의"), ("시간", "1800000", Some("밀리초")));
    assert_eq!(value(&program, "수업").1, "7200000");
    assert_eq!(value(&program, "기간").1, "259200000");
    assert_eq!(value(&program, "길"), ("거리", "2000000", Some("밀리미터")));

    // 선언한 단위가 같은 이름의 기본 단위보다 먼저입니다.
    let program = ok("단위 분\n정의 몫 = 3 분\n");
    assert_eq!(value(&program, "몫"), ("분", "3", Some("분")));
    // 규칙의 낱말은 단위로 읽지 않습니다.
    ok(
        "정의 수: 정수 = 5\n정의 떨어짐: 정도 = 70%\n규칙 깨짐: 떨어짐 그리고 수 > 3 이면 80% 곱으로\n",
    );
}

#[test]
fn a_sentence_carries_its_explanation() {
    let program = ok(
        "대상 조사자, 조사범위, 컵\n행위 확인하다: 주체, 대상\n행위 떨어지다, 깨지다: 주체\n\
         조사자가 조사범위를 확인한다: 조사 전에 기능·구매자(B2B)를 명시하라 # 100% 반영\n\
         첫째: 컵이 70% 떨어졌다: 실험실 바닥 기준\n\
         컵이 떨어지면 컵이 80% 깨진다: 유리컵일 때\n",
    );
    assert_eq!(
        program.sentences[0].note.as_deref(),
        Some("조사 전에 기능·구매자(B2B)를 명시하라 # 100% 반영")
    );
    assert_eq!(program.sentences[1].label.as_deref(), Some("첫째"));
    assert_eq!(
        program.sentences[1].note.as_deref(),
        Some("실험실 바닥 기준")
    );
    assert_eq!(program.conditionals[0].note.as_deref(), Some("유리컵일 때"));
    assert!(program.sentences[0].references.is_empty());

    // 설명이 붙어도 문장의 규칙은 그대로입니다.
    assert_eq!(
        fails("대상 조사범위\n행위 확인하다: 주체, 대상\n조사범위를 확인한다: 누가 하는지 없음\n"),
        ("K504", 3)
    );
    assert_eq!(
        fails("대상 조사자\n행위 확인하다: 주체\n조사자가 확인한다:\n"),
        ("K200", 3)
    );
    // 설명이 없는 문장의 중간 표현에는 설명 항목이 없습니다.
    let json = serde_json::to_string(&ok("대상 컵\n행위 떨어지다: 주체\n컵이 떨어진다\n")).unwrap();
    assert!(!json.contains("note") && !json.contains("units"));
}

#[test]
fn shipped_example_compiles() {
    let source = std::fs::read_to_string("examples/쓰기쉽게.kpl").unwrap();
    let program = ok(&source);
    let status = |name: &str| {
        program
            .goals
            .iter()
            .find(|g| g.name == name)
            .unwrap()
            .status
    };
    assert_eq!(status("참석자 <= 정원"), "보장");
    assert_eq!(status("요금 <= 최대요금"), "가능");
    assert_eq!(status("시간초과없음"), "가능");
}
