//! 0.10 본문 점검: 본문 속의 선언한 이름, 정의와 같은 값, 기준 없는 낱말, 지시어, 띄어쓰기.

use aha::{CompiledProgram, compile};

fn ok(source: &str) -> CompiledProgram {
    compile(source).unwrap_or_else(|error| panic!("{error}\n{source}"))
}

/// (코드, 줄, 열, 길이)
fn warnings(program: &CompiledProgram) -> Vec<(&'static str, usize, usize, usize)> {
    program
        .warnings
        .iter()
        .map(|w| (w.code, w.line, w.column, w.length))
        .collect()
}

/// (줄, 열, 이름)
fn mentions(program: &CompiledProgram) -> Vec<(usize, usize, &str)> {
    program
        .mentions
        .iter()
        .map(|m| (m.line, m.column, m.name.as_str()))
        .collect()
}

const BASE: &str = "대상 신청자, 강의실\n행위 예약하다: 주체, 대상\n단위 원, 명\n\
    정의 정원 = 40 명\n정의 기본요금 = 3만 원\n정의 예약마감 = 2 일\n";

#[test]
fn declared_names_are_found_in_every_kind_of_body() {
    let program = ok(&format!(
        "{BASE}신청자가 강의실을 이렇게 예약한다 (참조 1): 정원을 넘지 않는다\n\
         참조 1: 신청자는 기본요금부터 낸다\n\
         본문: 강의실의 정원이 바뀌면 다시 알린다\n\
         본문: 요금표는 뒤에 선언한다\n\
         대상 요금표\n"
    ));
    assert_eq!(
        mentions(&program),
        [
            (7, 28, "정원"),
            (8, 7, "신청자"),
            (8, 12, "기본요금"),
            (9, 5, "강의실"),
            (9, 10, "정원"),
            // 본문보다 뒤에 선언한 이름도 알아봅니다.
            (10, 5, "요금표"),
        ]
    );
    assert_eq!(program.mentions[2].length, 4);
    assert!(program.warnings.is_empty(), "{:?}", warnings(&program));
    assert_eq!(program.texts.len(), 2);
    assert_eq!(program.texts[0].line, 9);

    // 이름으로 시작하지만 조사가 아닌 글자가 이어지면 다른 낱말입니다.
    let program = ok(&format!("{BASE}본문: 정원사와 신청자격은 이름이 아니다\n"));
    assert!(program.mentions.is_empty(), "{:?}", mentions(&program));
}

#[test]
fn an_amount_equal_to_a_definition_is_reported() {
    let program = ok(&format!(
        "{BASE}본문: 3만 원을 내고 30,000원 영수증을 받는다. 3만원도 같다\n\
         본문: 40명까지 들어가고 2일 전에 마감한다. 48 시간과 같다\n\
         본문: 5만 원이나 41 명, 3만 명은 정의와 다르다. 연도 2026과 3만도 아니다\n"
    ));
    assert_eq!(
        warnings(&program),
        [
            ("K601", 7, 5, 4),
            ("K601", 7, 14, 7),
            ("K601", 7, 32, 3),
            ("K601", 8, 5, 3),
            ("K601", 8, 16, 2),
            ("K601", 8, 28, 5),
        ]
    );
    let first = &program.warnings[0].message;
    assert!(
        first.contains("'3만 원'") && first.contains("'기본요금'을"),
        "{first}"
    );
    assert!(program.warnings[4].message.contains("'예약마감'"));
}

#[test]
fn vague_words_pointers_and_spacing_are_reported() {
    let program = ok(&format!(
        "{BASE}신청자가 강의실을 예약한다: 적절히 확인하고 가급적 빨리 낸다\n\
         본문: 해당 신청과 그것을 확인한다. 해당하는 경우는 아니다\n\
         본문: 예약 마감이 지나면 기본 요금을 돌려주지 않는다\n"
    ));
    assert_eq!(
        warnings(&program),
        [
            ("K602", 7, 17, 3),
            ("K602", 7, 26, 3),
            ("K602", 7, 30, 2),
            ("K603", 8, 5, 2),
            ("K603", 8, 12, 2),
            ("K604", 9, 5, 5),
            ("K604", 9, 16, 5),
        ]
    );
    assert!(program.warnings[5].message.contains("'예약 마감'"));
    assert!(program.warnings[5].message.contains("'예약마감'으로"));
}

#[test]
fn warnings_do_not_stop_compilation_and_stay_out_of_clean_output() {
    // 경고가 있어도 판정은 그대로 나옵니다.
    let program = ok(&format!(
        "{BASE}미지 참석자: 명\n조건 참석자 >= 1 명\n목표 참석자 <= 정원\n본문: 적절히 한다\n"
    ));
    assert_eq!(program.goals[0].status, "가능");
    assert_eq!(program.warnings.len(), 1);

    // 본문이 없는 명세의 중간 표현에는 새 항목이 없습니다.
    let json = serde_json::to_string(&ok("정의 가 = 1\n")).unwrap();
    for key in ["texts", "mentions", "warnings"] {
        assert!(!json.contains(key), "{key}");
    }
    // 글 안의 기호와 '#'은 그대로 글입니다.
    let program = ok("본문: 가격(VAT 포함) > 100% # 주석 아님 = ?\n");
    assert_eq!(
        program.texts[0].text,
        "가격(VAT 포함) > 100% # 주석 아님 = ?"
    );
    assert_eq!(compile("본문:\n").err().unwrap().code, "K200");
    assert_eq!(compile("본문 글\n").err().unwrap().code, "K200");
}

#[test]
fn shipped_example_reports_the_documented_warnings() {
    let source = std::fs::read_to_string("examples/본문점검.aha").unwrap();
    let program = ok(&source);
    let codes: Vec<_> = program.warnings.iter().map(|w| (w.code, w.line)).collect();
    assert_eq!(
        codes,
        [("K601", 17), ("K602", 17), ("K604", 21), ("K603", 21)]
    );
    assert_eq!(program.mentions.len(), 5);
}

#[test]
fn lines_starting_with_a_dash_belong_to_the_line_above() {
    let program = ok(&format!(
        "{BASE}신청자가 강의실을 예약한다\n\
         - 정원을 넘지 않는지 확인한다\n\
         - 기본요금을 낸다 # 주석 아님\n\
         \n\
         # 주석과 빈 줄을 사이에 두어도 이어집니다\n\
         - 3만 원 영수증을 받는다\n\
         참조 1: 예약 규정\n\
         - 신청자만 예약한다\n\
         본문: 조사 원칙\n\
         -적절히 하지 않는다\n\
         행위 떨어지다, 깨지다: 주체\n\
         강의실이 떨어졌다\n\
         강의실이 떨어지면 강의실이 깨진다: 설명\n\
         - 조건문에도 딸린다\n"
    ));
    let texts = |items: &[aha::IrItem]| -> Vec<String> {
        items.iter().map(|item| item.text.clone()).collect()
    };
    assert_eq!(
        texts(&program.sentences[0].items),
        [
            "정원을 넘지 않는지 확인한다",
            "기본요금을 낸다 # 주석 아님",
            "3만 원 영수증을 받는다"
        ]
    );
    assert_eq!(program.sentences[0].items[2].line, 12);
    assert_eq!(texts(&program.references[0].items), ["신청자만 예약한다"]);
    assert_eq!(program.references[0].line, 13);
    assert_eq!(texts(&program.texts[0].items), ["적절히 하지 않는다"]);
    assert_eq!(texts(&program.conditionals[0].items), ["조건문에도 딸린다"]);
    assert_eq!(program.conditionals[0].note.as_deref(), Some("설명"));
    // 딸린 줄도 본문이므로 이름을 찾고 점검합니다.
    assert_eq!(
        mentions(&program)[..3],
        [(8, 3, "정원"), (9, 3, "기본요금"), (14, 3, "신청자")]
    );
    assert_eq!(warnings(&program), [("K601", 12, 3, 4), ("K602", 16, 2, 3)]);

    // 딸릴 곳이 없으면 오류입니다.
    let code = |source: &str| compile(source).err().unwrap().code;
    assert_eq!(code("- 처음부터 딸린 줄\n"), "K200");
    assert_eq!(code("정의 가 = 1\n- 정의에는 딸리지 않는다\n"), "K200");
    assert_eq!(
        code(&format!("{BASE}신청자가 강의실을 예약한다\n-\n")),
        "K200"
    );
    // 식의 음수는 그대로입니다.
    ok("정의 가 = -3\n검사 가 == 0 - 3\n");
}

#[test]
fn a_long_line_of_text_is_reported() {
    let long = "가나다라 ".repeat(17);
    let program = ok(&format!("본문: {long}\n본문: {}\n", "가".repeat(80)));
    assert_eq!(warnings(&program), [("K605", 1, 5, 4)]);
    assert!(program.warnings[0].message.contains("84자"));
}
