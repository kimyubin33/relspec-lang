//! 0.4 문장 문법, 참조와 지시어, 정도(퍼지 논리).

use aha::{CompiledProgram, compile};

fn ok(source: &str) -> CompiledProgram {
    compile(source).unwrap_or_else(|error| panic!("{error}\n{source}"))
}

fn fails(source: &str) -> (&'static str, usize) {
    let diagnostic = compile(source)
        .err()
        .unwrap_or_else(|| panic!("expected a diagnostic for:\n{source}"));
    (diagnostic.code, diagnostic.line)
}

const DESIGN: &str = "대상 유빈\n대상 설계안\n행위 설계하다: 주체, 대상\n";

#[test]
fn sentences_without_a_subject_do_not_compile() {
    // 사용자가 처음 제시한 예: 누가 설계했는지가 없습니다.
    let diagnostic = compile(&format!("{DESIGN}이렇게 설계를 하다"))
        .err()
        .expect("missing subject");
    assert_eq!((diagnostic.code, diagnostic.line), ("K504", 4));
    assert!(
        diagnostic.message.contains("주체"),
        "{}",
        diagnostic.message
    );

    assert_eq!(fails(&format!("{DESIGN}유빈이 설계했다")), ("K504", 4));
}

#[test]
fn demonstratives_need_references() {
    assert_eq!(
        fails(&format!("{DESIGN}유빈이 설계안을 이렇게 설계했다")),
        ("K510", 4)
    );

    let program = ok(&format!(
        "{DESIGN}유빈이 설계안을 이렇게 설계했다 (참조 1)\n\
         참조 1: 모듈을 세 층으로 나누고, 각 층은 아래 층만 참조한다 # 본문 속 기호도 그대로"
    ));
    let sentence = &program.sentences[0];
    assert_eq!(
        (sentence.verb.as_str(), sentence.form.as_str()),
        ("설계하다", "설계했다")
    );
    let roles: Vec<_> = sentence
        .roles
        .iter()
        .map(|r| (r.role.as_str(), r.particle.as_str(), r.value.as_str()))
        .collect();
    assert_eq!(
        roles,
        [
            ("주체", "이", "유빈"),
            ("대상", "을", "설계안"),
            ("방식", "", "참조 1")
        ]
    );
    assert_eq!(sentence.roles[2].reference, Some(1));
    assert_eq!(
        program.references[0].text,
        "모듈을 세 층으로 나누고, 각 층은 아래 층만 참조한다 # 본문 속 기호도 그대로"
    );

    // 지시 대명사는 조사로 역할을 채웁니다.
    let pronoun = ok(&format!(
        "{DESIGN}유빈이 이것을 설계했다 (참조 7)\n참조 7: 설계 초안"
    ));
    assert_eq!(pronoun.sentences[0].roles[1].reference, Some(7));
}

#[test]
fn reference_rules() {
    assert_eq!(
        fails(&format!(
            "{DESIGN}유빈이 설계안을 설계했다 (참조 2)\n참조 1: 다른 내용"
        )),
        ("K511", 4)
    );
    assert_eq!(fails("참조 1: 가\n참조 1: 나"), ("K512", 2));
    assert_eq!(fails("참조 1:"), ("K200", 1));
    // 지시어 두 개에는 참조 두 개가 필요합니다.
    assert_eq!(
        fails(&format!(
            "{DESIGN}그것이 이것을 설계했다 (참조 1)\n참조 1: 가"
        )),
        ("K510", 4)
    );
    ok(&format!(
        "{DESIGN}그것이 이것을 설계했다 (참조 1, 참조 2)\n참조 1: 가\n참조 2: 나"
    ));
}

#[test]
fn particle_forms_follow_the_final_consonant() {
    let base = "대상 유빈\n대상 나무\n대상 연필\n대상 칼\n행위 쓰다: 주체\n";
    assert_eq!(fails(&format!("{base}유빈가 썼다")), ("K502", 6));
    assert_eq!(fails(&format!("{base}나무이 썼다")), ("K502", 6));
    assert_eq!(fails(&format!("{base}유빈이 나무을 썼다")), ("K502", 6));
    // ㄹ 받침 뒤에는 '로'를 씁니다.
    assert_eq!(fails(&format!("{base}유빈이 연필으로 썼다")), ("K502", 6));
    ok(&format!(
        "{base}유빈이 연필로 썼다\n유빈은 칼로 썼다\n나무가 썼다"
    ));
}

#[test]
fn verbs_match_conjugated_forms() {
    let program = ok("대상 컵\n대상 유빈\n대상 물\n\
         행위 떨어지다: 주체\n행위 오다: 주체\n행위 먹다: 주체, 대상\n\
         행위 듣다 (들었다): 주체\n행위 부르다: 주체\n\
         컵이 떨어졌다\n컵이 떨어진다\n유빈이 왔다\n유빈이 물을 먹었다\n유빈이 들었다\n유빈이 불렀다");
    let forms: Vec<_> = program.sentences.iter().map(|s| s.verb.as_str()).collect();
    assert_eq!(
        forms,
        ["떨어지다", "떨어지다", "오다", "먹다", "듣다", "부르다"]
    );

    // '마셨다'는 '먹다'의 활용형이 아닙니다.
    assert_eq!(
        fails("대상 유빈\n대상 물\n행위 먹다: 주체, 대상\n유빈이 물을 마셨다"),
        ("K503", 4)
    );
    assert_eq!(
        fails("대상 유빈\n행위 뛰다: 주체\n행위 달리다 (뛰었다): 주체\n유빈이 뛰었다"),
        ("K507", 4)
    );
    assert_eq!(fails("행위 달리기: 주체"), ("K200", 1));
    assert_eq!(fails("행위 달리다: 주체, 속도"), ("K311", 1));
}

#[test]
fn particles_and_types_choose_roles_together() {
    let program = ok("기준 시계: 시간축\n기준 복도: 위치축\n대상 공\n\
         정의 출발시각: 시각 = 시계 @ 1 초\n\
         정의 도착시각: 시각 = 시계 @ 3 초\n\
         정의 입구: 위치 = 복도 @ 0 미터\n\
         정의 끝: 위치 = 복도 @ 5 미터\n\
         행위 구르다: 주체, 시작\n\
         공이 출발시각부터 도착시각까지 입구부터 끝까지 굴렀다\n\
         공이 출발시각에 입구에서 굴렀다");
    let roles: Vec<_> = program.sentences[0]
        .roles
        .iter()
        .map(|r| r.role.as_str())
        .collect();
    assert_eq!(roles, ["주체", "시작", "종료", "출발", "도착"]);
    let roles: Vec<_> = program.sentences[1]
        .roles
        .iter()
        .map(|r| r.role.as_str())
        .collect();
    assert_eq!(roles, ["주체", "시작", "장소"]);

    let base = "기준 시계: 시간축\n대상 공\n정의 가: 시각 = 시계 @ 1 초\n\
                정의 나: 시각 = 시계 @ 3 초\n행위 구르다: 주체\n";
    assert_eq!(
        fails(&format!("{base}공이 나부터 가까지 굴렀다")),
        ("K402", 6)
    );
    assert_eq!(fails(&format!("{base}공이 공에 굴렀다")), ("K506", 6));
    assert_eq!(fails(&format!("{base}가가 굴렀다")), ("K506", 6));
    assert_eq!(fails(&format!("{base}공이 공이 굴렀다")), ("K505", 6));
    assert_eq!(fails(&format!("{base}철수가 굴렀다")), ("K501", 6));
    assert_eq!(fails(&format!("{base}공 굴렀다")), ("K501", 6));
    assert_eq!(fails(&format!("{base}공이 굴렀다 정말")), ("K503", 6));
    assert_eq!(fails(&format!("{base}공이 = 굴렀다")), ("K500", 6));
    assert_eq!(fails("시작 = 1"), ("K500", 1));
}

#[test]
fn degrees_follow_fuzzy_logic() {
    let program = ok("대상 컵\n행위 떨어지다: 주체\n\
         떨어짐: 컵이 70% 떨어졌다\n\
         정의 공존: 정도 = 떨어짐 그리고 아님 떨어짐\n\
         정의 배중: 정도 = 떨어짐 또는 아님 떨어짐\n\
         정의 섞임: 정도 = 참 그리고 떨어짐\n\
         검사 떨어지되떨어지지않음 = 공존 == 30% 그리고 배중 == 70% 그리고 섞임 == 70%\n\
         검사 크기비교 = 떨어짐 > 50% 그리고 아님 떨어짐 <= 30%");
    assert_eq!(program.sentences[0].degree, 70);
    let value = |name: &str| {
        program
            .statements
            .iter()
            .find(|s| s.name == name)
            .unwrap()
            .value
            .clone()
    };
    // 퍼지 논리에서는 모순율(A 그리고 아님 A = 0)과 배중률(A 또는 아님 A = 100)이 성립하지 않습니다.
    assert_eq!(value("공존"), "30%");
    assert_eq!(value("배중"), "70%");

    // 정도를 적지 않은 문장은 100%입니다.
    let plain = ok("대상 컵\n행위 떨어지다: 주체\n가: 컵이 떨어졌다\n검사 완전 = 가 == 100%");
    assert_eq!(plain.sentences[0].degree, 100);

    let base = "대상 컵\n행위 떨어지다: 주체\n";
    assert_eq!(fails(&format!("{base}컵이 101% 떨어졌다")), ("K520", 3));
    assert_eq!(fails("정의 가: 정도 = 150%"), ("K520", 1));
    assert_eq!(fails(&format!("{base}컵이 10% 20% 떨어졌다")), ("K505", 3));
    assert_eq!(
        fails(&format!("{base}가: 컵이 떨어졌다\n검사 나 = 가")),
        ("K303", 4)
    );
    assert_eq!(
        fails(&format!("{base}가: 컵이 떨어졌다\n정의 나: 정도 = 가 + 가")),
        ("K304", 4)
    );
    assert_eq!(
        fails(&format!("{base}가: 컵이 떨어졌다\n가: 컵이 떨어졌다")),
        ("K301", 4)
    );
}

#[test]
fn sentence_example_ships() {
    let source = std::fs::read_to_string("examples/문장과정도.aha").unwrap();
    let program = ok(&source);
    assert!(!program.sentences.is_empty());
}
