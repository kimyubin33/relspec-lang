//! 0.7 문장으로 쓰는 규칙: `-면` 조건문, 사실 찾기, 결론의 사슬.

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

fn rule<'a>(program: &'a CompiledProgram, name: &str) -> &'a kpl::IrConditional {
    program
        .conditionals
        .iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("no conditional named {name}"))
}

fn goal<'a>(program: &'a CompiledProgram, name: &str) -> (&'a str, Vec<String>) {
    let goal = program.goals.iter().find(|g| g.name == name).expect("goal");
    (goal.status, goal.undecided_rules.clone())
}

const BASE: &str = "대상 컵\n대상 바닥\n대상 유빈\n대상 물\n\
    행위 떨어지다: 주체\n행위 깨지다: 주체\n행위 젖다: 주체\n행위 다치다: 주체\n\
    행위 먹다: 주체, 대상\n";

#[test]
fn conditional_sentences_propagate_degrees() {
    let program = ok(&format!(
        "{BASE}컵이 70% 떨어졌다\n\
         깨짐: 컵이 떨어지면 컵이 80% 깨진다\n\
         컵이 깨지면 바닥이 90% 젖는다 곱으로\n\
         다침: 바닥이 젖으면 유빈이 50% 다친다\n\
         목표 깨짐심함 = 깨짐 >= 60%\n\
         목표 다침있음 = 다침 >= 25%"
    ));
    let broken = rule(&program, "깨짐");
    assert!(broken.labeled);
    assert_eq!(broken.premise, "70%");
    assert_eq!(broken.strength, "80%");
    assert_eq!(broken.value, "56%~70%");
    assert_eq!(broken.matched_lines, [10]);
    assert_eq!(
        (
            broken.condition.verb.as_str(),
            broken.condition.form.as_str()
        ),
        ("떨어지다", "떨어지면")
    );
    assert_eq!(broken.conclusion.verb, "깨지다");

    // 이름을 붙이지 않은 조건문은 줄 번호로 부릅니다.
    let wet = rule(&program, "12행");
    assert!(!wet.labeled);
    assert_eq!(wet.method, Some("곱"));
    assert_eq!(wet.premise, "56%~70%");
    assert_eq!(wet.value, "50.4%~63%");

    let hurt = rule(&program, "다침");
    assert_eq!(hurt.value, "25.2%~50%");
    assert_eq!(hurt.outcomes.as_ref().unwrap().len(), 4);

    assert_eq!(
        goal(&program, "깨짐심함"),
        ("가능", vec!["깨짐".to_string()])
    );
    assert_eq!(goal(&program, "다침있음"), ("보장", vec![]));

    let ir = serde_json::to_value(&program).unwrap();
    assert_eq!(ir["languageVersion"], kpl::LANGUAGE_VERSION);
    assert_eq!(ir["conditionals"][0]["matchedLines"][0], 10);
    assert_eq!(
        ir["conditionals"][0]["condition"]["roles"][0]["value"],
        "컵"
    );
}

#[test]
fn strength_defaults_to_full_and_methods_can_be_inside_or_after() {
    let program = ok(&format!(
        "{BASE}컵이 70% 떨어졌다\n\
         가: 컵이 떨어지면 컵이 깨진다\n\
         나: 컵이 떨어지면 바닥이 80% 젖는다 최소로\n\
         다: 컵이 떨어지면 유빈이 곱으로 80% 다친다\n\
         검사 값 = 가 == 70% 그리고 나 == 70% 그리고 다 == 56%"
    ));
    // 강도를 적지 않으면 100%이고, 이때 두 방식의 결과가 같습니다.
    assert_eq!(rule(&program, "가").strength, "100%");
    assert!(rule(&program, "가").outcomes.is_none());
    assert_eq!(rule(&program, "나").method, Some("최소"));
    assert_eq!(rule(&program, "다").method, Some("곱"));
}

#[test]
fn premises_match_facts_by_verb_and_roles() {
    // 사실에 역할이 더 있어도 전제에 해당합니다: "유빈이 물을 먹었다"는 "유빈이 먹었다"를 함의합니다.
    let program = ok(&format!(
        "{BASE}행위 마시다: 주체\n\
         유빈이 물을 60% 먹었다\n\
         가: 유빈이 물을 먹으면 바닥이 젖는다\n\
         검사 값 = 가 == 60%"
    ));
    assert_eq!(rule(&program, "가").matched_lines, [11]);

    // 역할이 다르면 해당하지 않습니다.
    assert_eq!(
        fails(&format!(
            "{BASE}유빈이 물을 60% 먹었다\n유빈이 컵을 먹으면 바닥이 젖는다"
        )),
        ("K551", 11)
    );
    assert_eq!(
        fails(&format!(
            "{BASE}컵이 70% 떨어졌다\n바닥이 떨어지면 컵이 깨진다"
        )),
        ("K551", 11)
    );
}

#[test]
fn several_matching_facts_combine_by_maximum() {
    let program = ok(&format!(
        "{BASE}행위 밟히다: 주체\n\
         컵이 70% 떨어졌다\n컵이 40% 밟혔다\n\
         컵이 떨어지면 컵이 50% 깨진다 곱으로\n\
         컵이 밟히면 컵이 90% 깨진다 곱으로\n\
         부서짐: 컵이 깨지면 바닥이 젖는다\n\
         검사 합침 = 부서짐 == 36%"
    ));
    // 두 조건문이 같은 결론을 냅니다: 35%와 36%. 다음 전제는 둘 중 큰 값을 씁니다.
    let combined = rule(&program, "부서짐");
    assert_eq!(combined.matched_lines, [13, 14]);
    assert_eq!(combined.premise, "36%");

    // 같은 사실을 두 번 말해도 큰 값을 씁니다.
    let twice = ok(&format!(
        "{BASE}컵이 70% 떨어졌다\n컵이 40% 떨어졌다\n가: 컵이 떨어지면 컵이 깨진다\n검사 값 = 가 == 70%"
    ));
    assert_eq!(rule(&twice, "가").matched_lines, [10, 11]);
}

#[test]
fn undecided_methods_stay_consistent_across_combined_facts() {
    // 두 사실이 같은 규칙에서 갈라졌다면 최댓값도 세계마다 따로 구합니다.
    let program = ok(&format!(
        "{BASE}컵이 70% 떨어졌다\n\
         깨짐: 컵이 떨어지면 컵이 80% 깨진다\n\
         컵이 60% 깨졌다\n\
         결과: 컵이 깨지면 바닥이 젖는다"
    ));
    // 깨짐이 최소(70%)인 세계에서는 max(70, 60) = 70, 곱(56%)인 세계에서는 max(56, 60) = 60.
    let result = rule(&program, "결과");
    assert_eq!(result.premise, "60%~70%");
    let mut values: Vec<_> = result
        .outcomes
        .as_ref()
        .unwrap()
        .iter()
        .map(|o| o.value.clone())
        .collect();
    values.sort();
    assert_eq!(values, ["60%", "70%"]);
}

#[test]
fn a_zero_degree_fact_is_different_from_a_missing_fact() {
    // 사실이 없으면 짐작하지 않고 오류입니다. 일어나지 않았다면 0%로 적습니다.
    assert_eq!(
        fails(&format!("{BASE}컵이 떨어지면 컵이 깨진다")),
        ("K551", 10)
    );
    let program = ok(&format!(
        "{BASE}컵이 0% 떨어졌다\n가: 컵이 떨어지면 컵이 80% 깨진다\n검사 영 = 가 == 0%"
    ));
    assert_eq!(rule(&program, "가").value, "0%");
}

#[test]
fn facts_must_come_before_the_conditionals_that_use_them() {
    assert_eq!(
        fails(&format!(
            "{BASE}컵이 70% 떨어졌다\n컵이 떨어지면 컵이 깨진다\n컵이 30% 떨어졌다"
        )),
        ("K552", 12)
    );
    // 이끌어 낸 사실도 앞선 조건문의 전제에 해당하면 안 됩니다.
    assert_eq!(
        fails(&format!(
            "{BASE}컵이 70% 떨어졌다\n바닥이 50% 젖었다\n\
             컵이 떨어지면 컵이 깨진다\n바닥이 젖으면 컵이 떨어진다"
        )),
        ("K552", 13)
    );
    // 결론이 자기 전제가 되는 순환.
    let diagnostic = compile(&format!(
        "{BASE}컵이 70% 떨어졌다\n컵이 떨어지면 컵이 떨어진다"
    ))
    .err()
    .unwrap();
    assert_eq!((diagnostic.code, diagnostic.line), ("K552", 11));
    assert!(
        diagnostic.message.contains("순환"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn conditional_verb_forms() {
    let program = ok("대상 유빈\n대상 컵\n\
         행위 설계하다: 주체\n행위 듣다 (들었다, 들으면): 주체\n행위 살다: 주체\n\
         행위 구르다: 주체\n행위 깨지다: 주체\n\
         유빈이 설계했다\n유빈이 들었다\n유빈이 살았다\n유빈이 굴렀다\n\
         유빈이 설계하면 컵이 깨진다\n\
         유빈이 설계를 하면 컵이 깨진다\n\
         유빈이 들으면 컵이 깨진다\n\
         유빈이 살면 컵이 깨진다\n\
         유빈이 구르면 컵이 깨진다");
    let verbs: Vec<_> = program
        .conditionals
        .iter()
        .map(|r| r.condition.verb.as_str())
        .collect();
    assert_eq!(verbs, ["설계하다", "설계하다", "듣다", "살다", "구르다"]);

    let base = format!("{BASE}컵이 70% 떨어졌다\n유빈이 물을 먹었다\n");
    // 받침 없는 어간에는 '-면', 받침 있는 어간에는 '-으면'을 씁니다.
    assert_eq!(
        fails(&format!("{base}컵이 떨어지으면 컵이 깨진다")),
        ("K503", 12)
    );
    assert_eq!(
        fails(&format!("{base}유빈이 물을 먹면 컵이 깨진다")),
        ("K503", 12)
    );
    ok(&format!("{base}유빈이 물을 먹으면 컵이 깨진다"));
}

#[test]
fn conditional_structure_errors() {
    let base = format!("{BASE}컵이 70% 떨어졌다\n");
    assert_eq!(fails(&format!("{base}컵이 떨어지면")), ("K550", 11));
    assert_eq!(
        fails(&format!("{base}컵이 70% 떨어지면 컵이 깨진다")),
        ("K550", 11)
    );
    assert_eq!(
        fails(&format!("{base}컵이 떨어지면 컵이 깨지면 바닥이 젖는다")),
        ("K550", 11)
    );
    assert_eq!(fails(&format!("{base}컵이 곱으로 깨졌다")), ("K550", 11));
    assert_eq!(
        fails(&format!("{base}컵이 떨어지면 컵이 곱으로 깨진다 최소로")),
        ("K505", 11)
    );
    // 전제와 결론 모두 필수 역할을 지켜야 합니다.
    assert_eq!(fails(&format!("{base}떨어지면 컵이 깨진다")), ("K504", 11));
    assert_eq!(fails(&format!("{base}컵이 떨어지면 깨진다")), ("K504", 11));
    assert_eq!(
        fails(&format!("{base}컵이 떨어지면 컵이 120% 깨진다")),
        ("K520", 11)
    );
    // '-면'으로 끝나는 선언된 이름은 전제로 읽지 않습니다.
    assert_eq!(
        fails("대상 화면\n행위 켜지다: 주체\n화면 켜졌다"),
        ("K501", 3)
    );
    ok("대상 화면\n행위 켜지다: 주체\n화면이 켜졌다");
}

#[test]
fn labeled_conclusions_are_degrees_and_events_and_feed_keyword_rules() {
    let program = ok(&format!(
        "{BASE}컵이 70% 떨어졌다\n\
         깨짐: 컵이 떨어지면 컵이 80% 깨진다 곱으로\n\
         규칙 위험: 깨짐 이면 50% 최소로\n\
         검사 값 = 깨짐 == 56% 그리고 위험 == 50% 그리고 깨짐.주체 == 컵"
    ));
    assert_eq!(program.rules[0].value, "50%");
    assert_eq!(program.conditionals.len(), 1);
}

#[test]
fn checks_on_undecided_conditionals_must_hold_both_ways() {
    let base = format!("{BASE}컵이 70% 떨어졌다\n깨짐: 컵이 떨어지면 컵이 80% 깨진다\n");
    ok(&format!("{base}검사 충분 = 깨짐 >= 50%"));
    assert_eq!(
        fails(&format!("{base}검사 엄격 = 깨짐 >= 60%")),
        ("K540", 12)
    );
}

#[test]
fn demonstratives_and_references_span_both_clauses() {
    let program = ok(&format!(
        "{BASE}컵이 이렇게 70% 떨어졌다 (참조 1)\n\
         컵이 이렇게 떨어지면 컵이 그렇게 깨진다 (참조 1, 참조 2)\n\
         참조 1: 식탁 모서리에서 미끄러져\n참조 2: 손잡이가 떨어져 나가며"
    ));
    let rule = &program.conditionals[0];
    assert_eq!(rule.references, [1, 2]);
    assert_eq!(rule.conclusion.roles[1].reference, Some(2));
    // 전제의 '이렇게'가 다른 참조를 가리키면 사실에 해당하지 않습니다.
    assert_eq!(
        fails(&format!(
            "{BASE}컵이 이렇게 70% 떨어졌다 (참조 1)\n\
             컵이 이렇게 떨어지면 컵이 깨진다 (참조 2)\n참조 1: 가\n참조 2: 나"
        )),
        ("K551", 11)
    );
    assert_eq!(
        fails(&format!(
            "{BASE}컵이 70% 떨어졌다\n컵이 떨어지면 컵이 이렇게 깨진다"
        )),
        ("K510", 11)
    );
}

#[test]
fn conditional_example_ships() {
    let source = std::fs::read_to_string("examples/조건문.kpl").unwrap();
    let program = ok(&source);
    assert!(program.conditionals.len() >= 3);
}
