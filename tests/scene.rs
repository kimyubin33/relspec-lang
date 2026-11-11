//! 0.13 낱말과 장면: 차원의 묶음인 낱말, 들여쓰기로 적는 장면, 빈칸과 채움 점수.

use kpl::{CompiledProgram, compile};

fn ok(source: &str) -> CompiledProgram {
    compile(source).unwrap_or_else(|error| panic!("{error}\n{source}"))
}

fn fails(source: &str) -> (&'static str, usize, usize) {
    let diagnostic = compile(source)
        .err()
        .unwrap_or_else(|| panic!("expected a diagnostic for:\n{source}"));
    (diagnostic.code, diagnostic.line, diagnostic.column)
}

const WORDS: &str =
    "낱말 책상: 재질, 가로(거리)\n낱말 컵: 색* 3, 재질, 채움(정도) 2\n낱말 책: 색\n";

#[test]
fn a_word_is_a_bundle_of_dimensions() {
    let program = ok(&format!(
        "주제 <창가의 책상>\n{WORDS}장면 방\n  책: 색 파랑\n"
    ));
    assert_eq!(program.topic.as_deref(), Some("창가의 책상"));
    let cup = &program.words[1];
    assert_eq!(cup.name, "컵");
    let dims: Vec<_> = cup
        .dimensions
        .iter()
        .map(|d| (d.name.as_str(), d.required, d.weight, d.kind))
        .collect();
    assert_eq!(
        dims,
        [
            ("색", true, 3, None),
            ("재질", false, 1, None),
            ("채움", false, 2, Some("정도"))
        ]
    );
    // 꺾쇠 없이 적어도 됩니다.
    assert_eq!(
        ok("주제 바람이 분다\n정의 가 = 1\n").topic.as_deref(),
        Some("바람이 분다")
    );

    assert_eq!(fails("낱말 컵: 색, 색\n").0, "K701");
    assert_eq!(fails("낱말 컵: 색\n낱말 컵: 재질\n").0, "K301");
    assert_eq!(fails("낱말 컵: 값(원)\n").0, "K315");
    assert_eq!(fails("낱말 컵: 색 0\n").0, "K200");
    assert_eq!(fails("낱말 컵\n").0, "K200");
    assert_eq!(fails("주제 <가>\n주제 <나>\n정의 가 = 1\n").0, "K200");
}

#[test]
fn indentation_puts_words_inside_words() {
    let program = ok(&format!(
        "{WORDS}장면 책상위\n  책상: 재질 나무, 가로 120 센티미터\n    컵: 색 흰색, 채움 80%\n    책: 색 파랑\n      컵 작은컵: 색 검정\n  빛: 방향 왼쪽 창문, 세기 80%\n\n  # 빈 줄과 주석이 사이에 있어도 됩니다\n  보는자리: 거리 1 미터\n정의 끝 = 1\n"
    ));
    let scene = &program.scenes[0];
    let shape: Vec<_> = scene
        .items
        .iter()
        .map(|i| {
            (
                i.name.as_str(),
                i.word.as_str(),
                i.depth,
                i.parent.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        shape,
        [
            ("책상", "책상", 0, None),
            ("컵", "컵", 1, Some("책상")),
            ("책", "책", 1, Some("책상")),
            ("작은컵", "컵", 2, Some("책")),
            ("빛", "빛", 0, None),
            ("보는자리", "보는자리", 0, None),
        ]
    );
    // 타입이 있는 차원은 값으로 읽고, 없는 차원은 글 그대로입니다.
    let desk = &scene.items[0];
    assert_eq!(desk.values[1].value, "1200 밀리미터");
    assert_eq!(scene.items[4].values[0].value, "왼쪽 창문");
    assert_eq!(scene.items[4].values[1].value, "80%");
    assert_eq!(scene.items[1].blanks, ["재질"]);
    // 장면이 끝난 뒤의 줄은 보통의 줄입니다.
    assert!(program.statements.iter().any(|s| s.name == "끝"));
}

#[test]
fn fill_is_the_weighted_share_of_dimensions_written() {
    // 컵: 색 3 + 재질 1 + 채움 2 = 6 가운데 색만 적으면 3.
    let program = ok(&format!("{WORDS}장면 방\n  컵: 색 흰색\n"));
    assert_eq!(program.scenes[0].fill, 50);
    let program = ok(&format!(
        "{WORDS}장면 방\n  컵: 색 흰색, 채움 80%, 재질 유리\n"
    ));
    assert_eq!(program.scenes[0].fill, 100);
    assert!(program.scenes[0].items[0].blanks.is_empty());
    // 책상 2 + 컵 6 = 8 가운데 책상.재질 1 + 컵.색 3 + 컵.채움 2 = 6.
    let program = ok(&format!(
        "{WORDS}장면 방\n  책상: 재질 나무\n    컵: 색 흰색, 채움 10%\n"
    ));
    assert_eq!(program.scenes[0].fill, 75);
}

#[test]
fn scene_mistakes_are_reported_where_they_are() {
    // 정의하지 않은 낱말
    assert_eq!(
        fails(&format!("{WORDS}장면 방\n  의자: 색 검정\n")),
        ("K702", 5, 3)
    );
    // 없는 차원, 두 번 적은 차원
    assert_eq!(
        fails(&format!("{WORDS}장면 방\n  책: 무게 3\n")),
        ("K703", 5, 6)
    );
    assert_eq!(
        fails(&format!("{WORDS}장면 방\n  책: 색 파랑, 색 빨강\n")),
        ("K703", 5, 13)
    );
    // 꼭 적어야 하는 차원
    assert_eq!(
        fails(&format!("{WORDS}장면 방\n  컵: 재질 유리\n")),
        ("K704", 5, 3)
    );
    // 타입이 있는 차원의 값
    assert_eq!(
        fails(&format!("{WORDS}장면 방\n  컵: 색 흰색, 채움 3 미터\n")).0,
        "K303"
    );
    assert_eq!(
        fails(&format!("{WORDS}장면 방\n  컵: 색 흰색, 채움 150%\n")).0,
        "K520"
    );
    // 같은 이름은 한 번만. 또 쓰려면 이름을 붙입니다.
    assert_eq!(
        fails(&format!("{WORDS}장면 방\n  책: 색 파랑\n  책: 색 빨강\n")),
        ("K301", 6, 3)
    );
    ok(&format!(
        "{WORDS}장면 방\n  책: 색 파랑\n  책 둘째책: 색 빨강\n"
    ));
    // 값이 없는 차원, 빈 장면
    assert_eq!(fails(&format!("{WORDS}장면 방\n  책: 색\n")).0, "K200");
    assert_eq!(fails(&format!("{WORDS}장면 방\n정의 가 = 1\n")).0, "K200");
    assert_eq!(fails(&format!("{WORDS}장면 방\n장면 방\n")).0, "K200");
}

#[test]
fn words_in_a_scene_can_be_used_in_sentences() {
    let program = ok(&format!(
        "{WORDS}장면 방\n  책상: 재질 나무\n    컵: 색 흰색\n행위 떨어지다, 깨지다: 주체\n컵이 70% 떨어졌다\n컵이 떨어지면 컵이 80% 깨진다 곱으로\n"
    ));
    assert_eq!(program.sentences[0].roles[0].value, "컵");
    assert_eq!(program.conditionals[0].value, "56%");
    // 장면 밖의 이름과도 겹칠 수 없습니다.
    assert_eq!(
        fails(&format!("{WORDS}대상 컵\n장면 방\n  컵: 색 흰색\n")).0,
        "K301"
    );
}

#[test]
fn the_graph_shows_containment_and_blanks() {
    let program = ok(&format!(
        "{WORDS}장면 방\n  책상: 재질 나무\n    컵: 색 흰색\n행위 떨어지다: 주체\n컵이 떨어졌다\n"
    ));
    let graph = kpl::graph::build(&program);
    let edge = |from: &str, to: &str| {
        graph
            .edges
            .iter()
            .any(|e| e.from == from && e.to == to && e.kind == "포함")
    };
    assert!(edge("장면:방", "이름:책상"));
    assert!(edge("이름:책상", "이름:컵"));
    let cup = graph.nodes.iter().find(|n| n.id == "이름:컵").unwrap();
    assert_eq!(cup.detail, "색: 흰색\n빈칸: 재질, 채움");
    assert_eq!(graph.components().len(), 1);
}

#[test]
fn shipped_example_reports_its_blanks() {
    let source = std::fs::read_to_string("examples/장면.kpl").unwrap();
    let program = ok(&source);
    let scene = &program.scenes[0];
    assert_eq!((scene.items.len(), scene.fill), (6, 59));
    let blanks: usize = scene.items.iter().map(|i| i.blanks.len()).sum();
    assert_eq!(blanks, 9);
}
