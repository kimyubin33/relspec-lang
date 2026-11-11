//! 0.2 관계 모델: 기준 위의 시각·위치, 대상, 사건 역할.

use aha::{CompiledProgram, compile};

fn ok(source: &str) -> CompiledProgram {
    compile(source).unwrap_or_else(|error| panic!("{error}"))
}

fn fails(source: &str) -> (&'static str, usize, usize) {
    let diagnostic = compile(source)
        .err()
        .unwrap_or_else(|| panic!("expected a diagnostic for:\n{source}"));
    (diagnostic.code, diagnostic.line, diagnostic.column)
}

const CLOCK: &str = "기준 실험시계: 시간축\n";

#[test]
fn instants_and_durations_follow_point_vector_rules() {
    let program = ok(&format!(
        "{CLOCK}정의 시작: 시각 = 실험시계 @ 1 초\n\
         정의 종료: 시각 = 시작 + 250 밀리초\n\
         정의 앞당김: 시각 = 250 밀리초 + 시작\n\
         정의 소요: 시간 = 종료 - 시작\n\
         검사 순서 = 종료 > 시작 그리고 소요 == 250 밀리초 그리고 앞당김 == 종료"
    ));
    let ir = serde_json::to_value(&program).unwrap();
    assert_eq!(ir["languageVersion"], aha::LANGUAGE_VERSION);
    assert_eq!(ir["frames"][0]["axis"], "시간축");
    assert_eq!(ir["statements"][1]["value"], "1250");
    assert_eq!(ir["statements"][1]["frame"], "실험시계");
    assert_eq!(ir["statements"][3]["type"], "시간");
    assert!(ir["statements"][3].get("frame").is_none());
}

#[test]
fn rejects_adding_two_instants() {
    let source = format!("{CLOCK}정의 가: 시각 = 실험시계 @ 1 초\n정의 나: 시각 = 가 + 가");
    assert_eq!(fails(&source), ("K305", 3, 14));
}

#[test]
fn rejects_mixing_two_frames() {
    let source = format!(
        "{CLOCK}기준 영상시계: 시간축\n\
         정의 가: 시각 = 실험시계 @ 0 초\n\
         정의 나: 시각 = 영상시계 @ 0 초\n\
         검사 같음 = 가 == 나"
    );
    assert_eq!(fails(&source), ("K306", 5, 11));
}

#[test]
fn rejects_durations_used_as_instants() {
    // 선언 타입, 비교, 사건 역할 세 자리 모두 같은 진단 코드로 보고합니다.
    let declared = format!("{CLOCK}정의 마감: 시각 = 300 밀리초");
    assert_eq!(fails(&declared), ("K307", 2, 4));

    let compared =
        format!("{CLOCK}정의 종료: 시각 = 실험시계 @ 1 초\n검사 제한 = 종료 <= 300 밀리초");
    assert_eq!(fails(&compared), ("K307", 3, 12));

    let role = format!("{CLOCK}사건 측정: 시작 = 1 초");
    assert_eq!(fails(&role), ("K307", 2, 8));
}

#[test]
fn anchors_require_a_frame_of_the_matching_axis() {
    assert_eq!(fails("정의 가: 시각 = 없는기준 @ 1 초"), ("K308", 1, 12));
    let wrong_axis = format!("{CLOCK}정의 가: 시각 = 실험시계 @ 1 미터");
    assert_eq!(fails(&wrong_axis), ("K308", 2, 17));
}

#[test]
fn frames_and_events_are_not_values() {
    let frame = format!("{CLOCK}검사 가 = 실험시계 == 실험시계");
    assert_eq!(fails(&frame), ("K309", 2, 8));

    let event = format!("{CLOCK}사건 측정: 시작 = 실험시계 @ 0 초\n검사 가 = 측정 == 측정");
    assert_eq!(fails(&event), ("K309", 3, 8));
}

#[test]
fn events_expose_roles_and_derived_measures() {
    let program = ok(&format!(
        "{CLOCK}기준 복도: 위치축\n\
         대상 공\n\
         대상 유빈\n\
         사건 굴림: 주체 = 유빈, 대상 = 공, 시작 = 실험시계 @ 1 초, 종료 = 실험시계 @ 3 초, 출발 = 복도 @ 0 미터, 도착 = 복도 @ 150 센티미터\n\
         사건 잡기: 주체 = 유빈, 시작 = 굴림.종료 + 200 밀리초\n\
         검사 순서 = 잡기.시작 - 굴림.종료 == 200 밀리초\n\
         검사 측정값 = 굴림.소요 == 2 초 그리고 굴림.이동 == 1500 밀리미터\n\
         검사 주체 = 굴림.주체 == 잡기.주체 그리고 굴림.대상 != 유빈\n\
         검사 순간 = 잡기.종료 == 잡기.시작"
    ));
    let ir = serde_json::to_value(&program).unwrap();
    assert_eq!(ir["entities"][1]["name"], "유빈");
    let caught = &ir["events"][1]["roles"];
    // 생략한 종료는 시작과 같은 값으로 채우고, 식이 없음을 표시합니다.
    assert_eq!(caught[2]["role"], "종료");
    assert_eq!(caught[2]["value"], "3200");
    assert!(caught[2].get("expression").is_none());
}

#[test]
fn event_role_rules() {
    let missing_start = "대상 공\n사건 굴림: 대상 = 공";
    assert_eq!(fails(missing_start), ("K312", 2, 4));

    let half_route = format!(
        "{CLOCK}기준 복도: 위치축\n사건 굴림: 시작 = 실험시계 @ 0 초, 출발 = 복도 @ 0 미터"
    );
    assert_eq!(fails(&half_route), ("K312", 3, 4));

    let duplicate = format!("{CLOCK}사건 굴림: 시작 = 실험시계 @ 0 초, 시작 = 실험시계 @ 1 초");
    assert_eq!(fails(&duplicate), ("K310", 2, 25));

    let unknown = format!("{CLOCK}사건 굴림: 시작 = 실험시계 @ 0 초, 속도 = 3");
    assert_eq!(fails(&unknown), ("K311", 2, 25));

    let not_entity = format!("{CLOCK}사건 굴림: 주체 = 3, 시작 = 실험시계 @ 0 초");
    assert_eq!(fails(&not_entity), ("K313", 2, 8));

    let no_route =
        format!("{CLOCK}사건 굴림: 시작 = 실험시계 @ 0 초\n검사 가 = 굴림.이동 == 0 미터");
    assert_eq!(fails(&no_route), ("K314", 3, 11));

    let backwards = format!("{CLOCK}사건 굴림: 시작 = 실험시계 @ 2 초, 종료 = 실험시계 @ 1 초");
    assert_eq!(fails(&backwards), ("K402", 2, 4));
}

#[test]
fn events_must_stay_on_one_frame() {
    let source = format!(
        "{CLOCK}기준 영상시계: 시간축\n사건 굴림: 시작 = 실험시계 @ 0 초, 종료 = 영상시계 @ 1 초"
    );
    assert_eq!(fails(&source), ("K306", 3, 25));
}

#[test]
fn entities_are_only_compared_for_identity() {
    let source = "대상 공\n대상 상자\n검사 가 = 공 < 상자";
    assert_eq!(fails(source), ("K304", 3, 10));
}

#[test]
fn shipped_examples_compile() {
    for path in ["examples/첫파일.aha", "examples/사건관계.aha"] {
        let source = std::fs::read_to_string(path).unwrap();
        compile(&source).unwrap_or_else(|error| panic!("{path}{error}"));
    }
    for (path, code) in [
        ("examples/실패/기준혼용.aha", "K306"),
        ("examples/실패/지속시간을시각으로.aha", "K307"),
        ("examples/실패/종료가시작보다앞섬.aha", "K402"),
        ("examples/실패/규칙과관측충돌.aha", "K404"),
    ] {
        let source = std::fs::read_to_string(path).unwrap();
        let diagnostic = compile(&source).err().expect(path);
        assert_eq!(diagnostic.code, code, "{path}");
    }
}
