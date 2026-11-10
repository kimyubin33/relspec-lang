//! 0.5 문장과 제약의 연결: 문장·사건 역할의 미지값, 시간 순서 제약, 시제.

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

fn status<'a>(program: &'a CompiledProgram, name: &str) -> &'a str {
    program
        .goals
        .iter()
        .find(|goal| goal.name == name)
        .expect("goal")
        .status
}

fn range(program: &CompiledProgram, name: &str) -> (Option<String>, Option<String>) {
    let unknown = program
        .unknowns
        .iter()
        .find(|u| u.name == name)
        .expect("unknown");
    (unknown.min.clone(), unknown.max.clone())
}

fn some(a: &str, b: &str) -> (Option<String>, Option<String>) {
    (Some(a.into()), Some(b.into()))
}

const REACTION: &str = "기준 실험시계: 시간축\n\
    대상 유빈\n대상 버튼\n대상 화면\n\
    행위 켜지다: 주체, 시작\n행위 누르다: 주체, 대상, 시작\n\
    정의 지금: 시각 = 실험시계 @ 3 초\n\
    정의 점등: 시각 = 실험시계 @ 1000 밀리초\n\
    미지 반응: 시각 = 실험시계 @ ?\n\
    자극: 화면이 점등에 켜졌다\n";

const RULES: &str = "조건 자극뒤 = 눌림.시작 >= 자극.시작\n\
    조건 지연상한 = 눌림.시작 - 자극.시작 <= 2500 밀리초\n\
    목표 마감 = 눌림.시작 <= 실험시계 @ 2500 밀리초\n\
    목표 여유 = 눌림.시작 <= 실험시계 @ 3 초\n";

#[test]
fn tense_narrows_unknown_times() {
    // 과거 시제: 눌린 시각은 지금(3초) 이전이므로 지연 상한(3.5초)보다 범위가 좁아집니다.
    let past = ok(&format!(
        "{REACTION}눌림: 유빈이 반응에 버튼을 눌렀다\n{RULES}"
    ));
    assert_eq!(range(&past, "반응"), some("1000", "3000"));
    assert_eq!(status(&past, "마감"), "가능");
    assert_eq!(status(&past, "여유"), "보장");
    assert_eq!(past.sentences[1].tense, Some("과거"));

    // 미래 시제: 지금보다 뒤에 시작합니다.
    let future = ok(&format!(
        "{REACTION}눌림: 유빈이 반응에 버튼을 누르겠다\n{RULES}"
    ));
    assert_eq!(range(&future, "반응"), some("3001", "3500"));
    assert_eq!(status(&future, "여유"), "불가능");
    assert_eq!(future.sentences[1].tense, Some("미래"));

    // 기본형에는 시제가 없으므로 지연 상한만 적용됩니다.
    let plain = ok(&format!(
        "{REACTION}눌림: 유빈이 반응에 버튼을 누르다\n{RULES}"
    ));
    assert_eq!(range(&plain, "반응"), some("1000", "3500"));
    assert_eq!(plain.sentences[1].tense, None);
}

#[test]
fn tense_contradictions() {
    // 확정된 시각: 1초에 켜진 일을 미래 시제로 쓸 수 없습니다.
    let source = REACTION.replace("화면이 점등에 켜졌다", "화면이 점등에 켜지겠다");
    assert_eq!(fails(&source), ("K530", 10));

    // 미지값: 조건으로 반응이 4초 이후로 정해졌는데 과거 시제로 쓰면 모순입니다.
    let source =
        format!("{REACTION}조건 늦음 = 반응 >= 실험시계 @ 4 초\n유빈이 반응에 버튼을 눌렀다");
    assert_eq!(fails(&source), ("K530", 12));

    // 현재 시제: 지금이 사건 안에 있어야 합니다.
    let base = "기준 시계: 시간축\n대상 공\n행위 구르다: 주체\n정의 지금: 시각 = 시계 @ 5 초\n\
                정의 가: 시각 = 시계 @ 1 초\n정의 나: 시각 = 시계 @ 3 초\n";
    assert_eq!(
        fails(&format!("{base}공이 가부터 나까지 구른다")),
        ("K530", 7)
    );
    ok(&format!("{base}공이 가부터 나까지 굴렀다"));

    // 지금과 사건 시각의 기준이 다르면 시제를 확인할 수 없습니다.
    let base = "기준 시계: 시간축\n기준 영상: 시간축\n대상 공\n행위 구르다: 주체\n\
                정의 지금: 시각 = 영상 @ 5 초\n정의 가: 시각 = 시계 @ 1 초\n";
    assert_eq!(fails(&format!("{base}공이 가에 굴렀다")), ("K306", 7));
}

#[test]
fn tense_detection() {
    let program = ok(
        "대상 유빈\n행위 하다: 주체\n행위 먹다: 주체\n행위 있다: 주체\n\
         유빈이 했다\n유빈이 한다\n유빈이 하겠다\n유빈이 하다\n\
         유빈이 먹는다\n유빈이 먹었다\n유빈이 있다\n유빈이 있었다",
    );
    let tenses: Vec<_> = program.sentences.iter().map(|s| s.tense).collect();
    assert_eq!(
        tenses,
        [
            Some("과거"),
            Some("현재"),
            Some("미래"),
            None,
            Some("현재"),
            Some("과거"),
            None,
            Some("과거"),
        ]
    );
}

#[test]
fn sentences_and_events_give_the_same_judgement() {
    let header = "기준 시계: 시간축\n대상 공\n행위 구르다: 주체, 시작\n\
                  미지 출발: 시각 = 시계 @ ?\n미지 정지: 시각 = 시계 @ ?\n";
    let rules = "조건 시작범위 = 굴림.시작 >= 시계 @ 1 초 그리고 굴림.시작 <= 시계 @ 2 초\n\
                 조건 길이 = 굴림.소요 <= 3 초\n\
                 목표 늦게끝남 = 굴림.종료 >= 시계 @ 4 초\n\
                 목표 빨리끝남 = 굴림.종료 <= 시계 @ 5 초\n\
                 목표 시작전끝남 = 굴림.종료 < 굴림.시작\n";
    let event = ok(&format!(
        "{header}사건 굴림: 주체 = 공, 시작 = 출발, 종료 = 정지\n{rules}"
    ));
    let sentence = ok(&format!(
        "{header}굴림: 공이 출발부터 정지까지 굴렀다\n{rules}"
    ));
    for program in [&event, &sentence] {
        assert_eq!(status(program, "늦게끝남"), "가능");
        assert_eq!(status(program, "빨리끝남"), "보장");
        // 종료가 시작보다 앞설 수 없다는 관계가 두 표기 모두에서 제약으로 들어갑니다.
        assert_eq!(status(program, "시작전끝남"), "불가능");
    }
    assert_eq!(range(&event, "정지"), range(&sentence, "정지"));
    assert_eq!(range(&event, "정지"), some("1000", "5000"));
}

#[test]
fn ordering_with_unknowns_is_a_constraint() {
    let header = "기준 시계: 시간축\n미지 가: 시각 = 시계 @ ?\n미지 나: 시각 = 시계 @ ?\n";
    // 조건이 먼저 '가가 나보다 뒤'라고 정했으면, 가에 시작해 나에 끝나는 사건은 있을 수 없습니다.
    let source = format!("{header}조건 순서 = 가 - 나 >= 1 초\n사건 일: 시작 = 가, 종료 = 나");
    assert_eq!(fails(&source), ("K402", 5));
    // 사건이 먼저 나오면 뒤의 조건이 모순으로 보고됩니다.
    let source = format!("{header}사건 일: 시작 = 가, 종료 = 나\n조건 순서 = 가 - 나 >= 1 초");
    assert_eq!(fails(&source), ("K404", 5));

    let program = ok(&format!(
        "{header}사건 일: 시작 = 가, 종료 = 가 + 200 밀리초\n검사 길이 = 일.소요 == 200 밀리초"
    ));
    assert_eq!(program.events[0].roles[1].value, "가 + 200");
}

#[test]
fn labeled_sentences_are_degrees_and_events() {
    let program = ok("기준 시계: 시간축\n대상 공\n행위 구르다: 주체\n\
         정의 가: 시각 = 시계 @ 1 초\n정의 나: 시각 = 시계 @ 4 초\n\
         굴림: 공이 가부터 나까지 60% 굴렀다\n\
         검사 비율 = 굴림 == 60%\n검사 길이 = 굴림.소요 == 3 초 그리고 굴림.주체 == 공");
    assert_eq!(program.statements.len(), 4);

    let source = "기준 시계: 시간축\n대상 공\n행위 구르다: 주체\n정의 가: 시각 = 시계 @ 1 초\n\
                  굴림: 공이 가에 굴렀다\n검사 길이 = 굴림.소요 == 0 초";
    assert_eq!(fails(source), ("K314", 6));
}

#[test]
fn positions_may_move_backwards() {
    // 시간과 달리 위치는 뒤로도 움직일 수 있습니다(0.4에서 잘못 거절하던 경우).
    let program = ok("기준 복도: 위치축\n대상 공\n행위 구르다: 주체\n\
         정의 가: 위치 = 복도 @ 5 미터\n정의 나: 위치 = 복도 @ 1 미터\n\
         굴림: 공이 가부터 나까지 굴렀다\n검사 뒤로 = 굴림.이동 == -4 미터");
    assert_eq!(program.sentences[0].roles.len(), 3);
}

#[test]
fn tense_example_ships() {
    let source = std::fs::read_to_string("examples/반응시제.kpl").unwrap();
    let program = ok(&source);
    assert_eq!(range(&program, "반응"), some("1000", "3000"));
    assert_eq!(status(&program, "마감"), "가능");
    assert_eq!(status(&program, "여유"), "보장");
}
