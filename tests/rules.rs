//! 0.6 정도의 전파: 규칙, 최소(Mamdani)·곱(Larsen) 방식, 방식에 따른 세계 나누기.

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

fn rule<'a>(program: &'a CompiledProgram, name: &str) -> &'a kpl::IrRule {
    program.rules.iter().find(|r| r.name == name).expect("rule")
}

fn goal<'a>(program: &'a CompiledProgram, name: &str) -> (&'a str, Vec<String>) {
    let goal = program.goals.iter().find(|g| g.name == name).expect("goal");
    (goal.status, goal.undecided_rules.clone())
}

const CUP: &str = "대상 컵\n대상 바닥\n행위 떨어지다: 주체\n행위 젖다: 주체\n\
                   떨어짐: 컵이 70% 떨어졌다\n젖음: 바닥이 40% 젖었다\n";

#[test]
fn decided_rules_use_their_method() {
    let program = ok(&format!(
        "{CUP}규칙 최소: 떨어짐 이면 80% 최소로\n규칙 곱: 떨어짐 이면 80% 곱으로\n\
         검사 값 = 최소 == 70% 그리고 곱 == 56%"
    ));
    assert_eq!(rule(&program, "최소").method, Some("최소"));
    assert_eq!(rule(&program, "곱").value, "56%");
    assert!(rule(&program, "곱").outcomes.is_none());
}

#[test]
fn products_are_exact_decimals() {
    let program = ok("대상 컵\n행위 떨어지다: 주체\n가: 컵이 33% 떨어졌다\n\
         규칙 반: 가 이면 50% 곱으로\n\
         규칙 작음: 1% 이면 1% 곱으로\n규칙 더작음: 작음 이면 1% 곱으로\n\
         검사 사이 = 반 > 16% 그리고 반 < 17%\n\
         정의 보수: 정도 = 아님 반");
    assert_eq!(rule(&program, "반").value, "16.5%");
    assert_eq!(rule(&program, "더작음").value, "0.0001%");
    let complement = program
        .statements
        .iter()
        .find(|s| s.name == "보수")
        .unwrap();
    assert_eq!(complement.value, "83.5%");
}

#[test]
fn undecided_rules_are_computed_both_ways() {
    let program = ok(&format!(
        "{CUP}규칙 깨짐: 떨어짐 이면 80%\n\
         목표 둘다충족 = 깨짐 >= 50%\n\
         목표 한쪽만 = 깨짐 >= 60%\n\
         목표 둘다아님 = 깨짐 >= 75%"
    ));
    let broken = rule(&program, "깨짐");
    assert_eq!(broken.method, None);
    assert_eq!(broken.value, "56%~70%");
    let outcomes: Vec<_> = broken
        .outcomes
        .as_ref()
        .unwrap()
        .iter()
        .map(|o| (o.choices["깨짐"], o.value.as_str()))
        .collect();
    assert_eq!(outcomes, [("최소", "70%"), ("곱", "56%")]);

    // 두 방식 모두에서 충족하면 보장, 한쪽에서만 충족하면 가능, 어느 쪽에서도 아니면 불가능.
    assert_eq!(goal(&program, "둘다충족"), ("보장", vec![]));
    assert_eq!(goal(&program, "한쪽만"), ("가능", vec!["깨짐".to_string()]));
    assert_eq!(goal(&program, "둘다아님"), ("불가능", vec![]));

    let ir = serde_json::to_value(&program).unwrap();
    assert_eq!(ir["goals"][1]["undecidedRules"][0], "깨짐");
    assert!(ir["goals"][0].get("undecidedRules").is_none());
}

#[test]
fn rules_chain_and_collapse_when_methods_agree() {
    let program = ok(&format!(
        "{CUP}규칙 깨짐: 떨어짐 이면 80%\n\
         규칙 미끄러짐: 젖음 이면 90% 곱으로\n\
         규칙 다침: 깨짐 그리고 미끄러짐 이면 50%\n\
         규칙 확실: 참 이면 80%\n\
         규칙 없음: 0% 이면 80%\n\
         목표 다침큼 = 다침 >= 30%"
    ));
    // 깨짐(56%~70%)과 미끄러짐(36%)의 최솟값은 어느 쪽이든 36%이므로 깨짐의 방식은 다침에 영향을 주지 않습니다.
    let hurt = rule(&program, "다침");
    assert_eq!(hurt.value, "18%~36%");
    assert_eq!(hurt.outcomes.as_ref().unwrap().len(), 2);
    assert_eq!(goal(&program, "다침큼"), ("가능", vec!["다침".to_string()]));
    // 전제가 100%나 0%이면 두 방식의 결과가 같아 하나로 합쳐집니다.
    assert_eq!(rule(&program, "확실").value, "80%");
    assert!(rule(&program, "확실").outcomes.is_none());
    assert_eq!(rule(&program, "없음").value, "0%");
}

#[test]
fn chained_undecided_rules_multiply_worlds() {
    let program = ok(&format!(
        "{CUP}규칙 깨짐: 떨어짐 이면 80%\n\
         규칙 번짐: 깨짐 이면 90%\n\
         목표 높음 = 번짐 >= 55%"
    ));
    let spread = rule(&program, "번짐");
    let mut values: Vec<_> = spread
        .outcomes
        .as_ref()
        .unwrap()
        .iter()
        .map(|o| o.value.clone())
        .collect();
    values.sort();
    // 최소·최소 70%, 최소·곱 63%, 곱·최소 56%, 곱·곱 50.4%
    assert_eq!(values, ["50.4%", "56%", "63%", "70%"]);
    assert_eq!(
        goal(&program, "높음"),
        ("가능", vec!["깨짐".to_string(), "번짐".to_string()])
    );
}

#[test]
fn the_same_rule_keeps_one_method_within_an_expression() {
    // '깨짐 그리고 아님 깨짐'에서 두 '깨짐'은 같은 규칙이므로 같은 방식으로 계산합니다.
    let program = ok(&format!(
        "{CUP}규칙 깨짐: 떨어짐 이면 80%\n정의 공존: 정도 = 깨짐 그리고 아님 깨짐"
    ));
    let coexist = program
        .statements
        .iter()
        .find(|s| s.name == "공존")
        .unwrap();
    let outcomes = coexist.outcomes.as_ref().unwrap();
    assert_eq!(outcomes.len(), 2);
    assert_eq!(coexist.value, "30%~44%");
}

#[test]
fn checks_must_hold_under_both_methods() {
    ok(&format!(
        "{CUP}규칙 깨짐: 떨어짐 이면 80%\n검사 충분 = 깨짐 >= 50%"
    ));
    let diagnostic = compile(&format!(
        "{CUP}규칙 깨짐: 떨어짐 이면 80%\n검사 엄격 = 깨짐 >= 60%"
    ))
    .err()
    .expect("split");
    assert_eq!((diagnostic.code, diagnostic.line), ("K540", 8));
    assert!(
        diagnostic.message.contains("깨짐 최소"),
        "{}",
        diagnostic.message
    );
    assert!(
        diagnostic.message.contains("깨짐 곱"),
        "{}",
        diagnostic.message
    );
}

#[test]
fn rule_errors() {
    assert_eq!(fails(&format!("{CUP}규칙 깨짐: 떨어짐 80%")), ("K200", 7));
    assert_eq!(
        fails(&format!("{CUP}규칙 깨짐: 떨어짐 이면 80% 대충")),
        ("K200", 7)
    );
    assert_eq!(fails(&format!("{CUP}규칙 깨짐: 3 이면 80%")), ("K303", 7));
    assert_eq!(
        fails(&format!("{CUP}규칙 깨짐: 떨어짐 이면 참")),
        ("K303", 7)
    );
    assert_eq!(
        fails(&format!("{CUP}규칙 떨어짐: 떨어짐 이면 80%")),
        ("K301", 7)
    );
    // 미지값은 정도나 논리가 될 수 없으므로, 전제에 들어오려면 비교를 거쳐야 하고 그 비교에서 거절됩니다.
    assert_eq!(
        fails("기준 시계: 시간축\n미지 가: 시간 = ?\n규칙 깨짐: 가 > 1 초 이면 80%"),
        ("K405", 3)
    );
    // 조건에는 방식에 따라 갈라지는 값을 쓸 수 없습니다.
    assert_eq!(
        fails(&format!(
            "{CUP}규칙 깨짐: 떨어짐 이면 80%\n조건 가 = 깨짐 >= 60%"
        )),
        ("K541", 8)
    );
    assert_eq!(
        fails(&format!(
            "{CUP}규칙 깨짐: 떨어짐 이면 80%\n기준 시계: 시간축\n미지 가: 시각 = 시계 @ ?\n\
             목표 섞음 = 가 <= 시계 @ 1 초 그리고 깨짐 >= 60%"
        )),
        ("K541", 10)
    );
}

#[test]
fn too_many_undecided_rules() {
    let mut source = String::from("규칙 r0: 70% 이면 80%\n");
    for i in 1..13 {
        source.push_str(&format!("규칙 r{i}: r{} 이면 90%\n", i - 1));
    }
    assert_eq!(fails(&source), ("K542", 13));
}

#[test]
fn diagnostic_particles_follow_the_final_consonant() {
    let message = |source: &str| compile(source).err().unwrap().message;
    let frame = message("기준 실험시계: 시간축\n검사 가 = 실험시계 == 실험시계");
    assert!(frame.starts_with("'실험시계'는 기준이므로"), "{frame}");
    let distance = message("기준 실험시계: 시간축\n정의 가: 시각 = 실험시계 @ 1 미터");
    assert!(distance.contains("'거리'가 주어졌습니다"), "{distance}");
    let points = message("기준 시계: 시간축\n정의 가: 시각 = 시계 @ 1 초\n정의 나: 시각 = 가 + 가");
    assert!(points.contains("'시각(시계)'와 '시각(시계)'를"), "{points}");
    // 기호로 끝나는 인용은 그대로 둡니다.
    let form = message("미지 가: 시각 = ?");
    assert!(form.contains("'= 기준 @ ?'로 적으세요"), "{form}");
}

#[test]
fn propagation_example_ships() {
    let source = std::fs::read_to_string("examples/정도전파.kpl").unwrap();
    let program = ok(&source);
    assert_eq!(goal(&program, "깨짐확실").0, "보장");
    assert_eq!(goal(&program, "깨짐심함").0, "가능");
}
