//! 0.12 연결망: 명세를 점과 선으로 옮긴 것.

use aha::graph::{Graph, build, html};

fn graph(source: &str) -> Graph {
    let program = aha::compile(source).unwrap_or_else(|error| panic!("{error}\n{source}"));
    build(&program)
}

fn has(graph: &Graph, from: &str, to: &str, kind: &str, label: &str) -> bool {
    graph
        .edges
        .iter()
        .any(|e| e.from == from && e.to == to && e.kind == kind && e.label == label)
}

const BASE: &str = "기준 시계: 시간축\n대상 신청자, 강의실\n행위 예약하다: 주체, 대상\n단위 명\n\
    정의 정원 = 40 명\n정의 마감: 시각 = 시계 @ 3 초\n";

#[test]
fn a_sentence_becomes_a_point_tied_to_its_roles() {
    let g = graph(&format!("{BASE}신청자가 마감에 강의실을 80% 예약했다\n"));
    let sentence = g.nodes.iter().find(|n| n.kind == "문장").unwrap();
    assert_eq!(sentence.id, "문장:7");
    assert_eq!(sentence.label, "신청자가 강의실을 예약했다");
    assert_eq!(sentence.degree.as_deref(), Some("80%"));
    assert_eq!(sentence.line, Some(7));
    assert!(has(&g, "문장:7", "이름:신청자", "역할", "주체"));
    assert!(has(&g, "문장:7", "이름:강의실", "역할", "대상"));
    assert!(has(&g, "문장:7", "행위:예약하다", "행위", ""));
    // 시각은 그 기준의 점으로 이어집니다.
    assert!(has(&g, "문장:7", "이름:시계", "역할", "시작 @ 3000 밀리초"));
    assert!(has(&g, "이름:마감", "이름:시계", "기준", ""));
    assert!(has(&g, "이름:정원", "단위:명", "단위", ""));
    assert!(g.edges.iter().all(|e| e.checked));
    // 선의 양 끝은 언제나 있는 점입니다.
    for edge in &g.edges {
        assert!(g.nodes.iter().any(|n| n.id == edge.from), "{edge:?}");
        assert!(g.nodes.iter().any(|n| n.id == edge.to), "{edge:?}");
    }
}

#[test]
fn values_goals_and_rules_are_tied_by_use() {
    let g = graph(
        "단위 원\n정의 기본 = 3만 원\n정의 두배 = 기본 * 2\n미지 요금: 원\n\
         조건 요금 >= 기본\n목표 요금 <= 두배\n목표 싸다 = 요금 < 0 원\n\
         대상 컵\n행위 떨어지다, 깨지다: 주체\n\
         떨어짐: 컵이 70% 떨어졌다\n컵이 떨어지면 컵이 80% 깨진다 곱으로\n\
         규칙 위험: 떨어짐 이면 90% 최소로\n",
    );
    assert!(has(&g, "이름:두배", "이름:기본", "쓰임", ""));
    assert!(has(&g, "조건:0", "이름:요금", "쓰임", ""));
    assert!(has(&g, "목표:0", "이름:두배", "쓰임", ""));
    let goal = |label: &str| g.nodes.iter().find(|n| n.label == label).unwrap().status;
    assert_eq!(goal("요금 <= 두배"), Some("가능"));
    assert_eq!(goal("싸다"), Some("불가능"));
    let unknown = g.nodes.iter().find(|n| n.id == "이름:요금").unwrap();
    assert_eq!(unknown.detail, "30000 원 ~ 상한 없음");
    // 조건문의 전제는 앞의 사실에서 오고, 선에 강도가 붙습니다.
    assert!(has(&g, "이름:떨어짐", "문장:11", "전제", "80%"));
    let conditional = g.nodes.iter().find(|n| n.id == "문장:11").unwrap();
    assert_eq!(conditional.kind, "조건문");
    assert_eq!(conditional.degree.as_deref(), Some("56%"));
    assert!(has(&g, "이름:위험", "이름:떨어짐", "쓰임", ""));
}

#[test]
fn text_is_attached_and_its_mentions_are_unchecked_lines() {
    let g = graph(&format!(
        "{BASE}신청자가 강의실을 이렇게 예약한다 (참조 1): 정원을 확인한다\n\
         - 40 명을 적절히 넘지 않는다\n\
         참조 1: 신청자만 한다\n\
         본문: 강의실은 깨끗이 쓴다\n"
    ));
    assert!(has(&g, "글:7", "문장:7", "딸림", "설명"));
    assert!(has(&g, "글:8", "문장:7", "딸림", "딸린 줄"));
    assert!(has(&g, "문장:7", "참조:1", "참조", "방식"));
    // 글 속의 이름은 '언급'이며 확인된 선이 아닙니다.
    let mention = |from: &str, to: &str| {
        g.edges
            .iter()
            .find(|e| e.from == from && e.to == to && e.kind == "언급")
            .map(|e| e.checked)
    };
    assert_eq!(mention("글:7", "이름:정원"), Some(false));
    assert_eq!(mention("참조:1", "이름:신청자"), Some(false));
    assert_eq!(mention("글:10", "이름:강의실"), Some(false));
    // 알림은 그 글의 점에 셉니다(K601, K602).
    assert_eq!(g.nodes.iter().find(|n| n.id == "글:8").unwrap().warnings, 2);
}

#[test]
fn separate_parts_of_a_specification_are_separate_groups() {
    let g = graph(
        "대상 신청자, 강의실, 창고\n행위 예약하다: 주체, 대상\n신청자가 강의실을 예약한다\n\
         단위 원\n미지 요금: 원\n조건 요금 >= 0 원\n",
    );
    let groups = g.components();
    let labels = |group: &Vec<usize>| -> Vec<&str> {
        group.iter().map(|&i| g.nodes[i].label.as_str()).collect()
    };
    assert_eq!(groups.len(), 3);
    assert_eq!(
        labels(&groups[0]),
        ["신청자", "강의실", "예약하다", "신청자가 강의실을 예약한다"]
    );
    assert_eq!(labels(&groups[1]), ["원", "요금", "요금 >= 0 원"]);
    // 어디에도 쓰이지 않은 대상은 혼자 남습니다.
    assert_eq!(labels(&groups[2]), ["창고"]);

    // 글에서 이름을 부르면 두 부분이 이어집니다.
    let g = graph(
        "대상 신청자, 강의실\n행위 예약하다: 주체, 대상\n신청자가 강의실을 예약한다: 요금을 낸다\n\
         단위 원\n미지 요금: 원\n조건 요금 >= 0 원\n",
    );
    assert_eq!(g.components().len(), 1);
}

#[test]
fn the_page_is_self_contained_and_safe() {
    let g = graph("대상 컵\n행위 떨어지다: 주체\n컵이 떨어졌다: </script><b>굵게</b>\n");
    let page = html(&g, "예제<1>.aha");
    assert!(page.starts_with("<!doctype html>"));
    assert!(page.contains("\"id\":\"이름:컵\""));
    // 글에 든 태그가 문서를 깨뜨리지 않습니다.
    assert_eq!(page.matches("</script>").count(), 1);
    assert!(page.contains("예제＜1＞.aha"));
    // 다른 파일이나 주소를 부르지 않습니다.
    assert!(!page.contains("http://") && !page.contains("https://"));
    assert!(!page.contains("/*GRAPH*/") && !page.contains("/*TITLE*/"));

    // 같은 명세는 언제나 같은 연결망이 됩니다.
    let again = graph("대상 컵\n행위 떨어지다: 주체\n컵이 떨어졌다: </script><b>굵게</b>\n");
    assert_eq!(g.nodes, again.nodes);
    assert_eq!(g.edges, again.edges);
}
