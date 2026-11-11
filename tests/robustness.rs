//! 편집기는 글자를 칠 때마다 쓰다 만 문서를 컴파일합니다.
//! 어떤 입력에도 컴파일러가 멈추지(panic) 않고 결과나 진단을 돌려주는지 확인합니다.

use std::fs;
use std::panic;

fn sources() -> Vec<(String, String)> {
    let mut sources = Vec::new();
    for dir in ["examples", "examples/실패"] {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|ext| ext == "aha") {
                sources.push((
                    path.display().to_string(),
                    fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }
    // 교재의 예제 블록도 재료로 씁니다.
    for entry in fs::read_dir("book").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "md") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for (index, block) in text.split("```aha").skip(1).enumerate() {
            if let Some((_, rest)) = block.split_once('\n')
                && let Some((body, _)) = rest.split_once("```")
            {
                sources.push((format!("{}#{index}", path.display()), body.to_string()));
            }
        }
    }
    sources
}

fn survives(name: &str, text: &str) -> Result<(), String> {
    let owned = text.to_string();
    panic::catch_unwind(move || {
        let _ = aha::compile(&owned);
    })
    .map_err(|_| format!("{name}: 다음 입력에서 컴파일러가 멈췄습니다:\n{text}\n"))
}

#[test]
fn compiler_never_panics_on_partial_documents() {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let mut failures = Vec::new();
    let mut checked = 0usize;
    for (name, source) in sources() {
        // 1. 글자 하나씩 쳐 나가는 모든 중간 상태
        for (end, _) in source.char_indices().chain([(source.len(), ' ')]) {
            checked += 1;
            if let Err(message) = survives(&name, &source[..end]) {
                failures.push(message);
                break;
            }
        }
        // 2. 줄 하나를 지운 상태, 줄의 뒷부분을 지운 상태
        let lines: Vec<&str> = source.lines().collect();
        for skip in 0..lines.len() {
            let without: Vec<&str> = lines
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, l)| *l)
                .collect();
            checked += 1;
            if let Err(message) = survives(&name, &without.join("\n")) {
                failures.push(message);
                break;
            }
        }
        // 3. 토큰 하나를 지운 상태
        let words: Vec<&str> = source.split(' ').collect();
        for skip in 0..words.len().min(400) {
            let without: Vec<&str> = words
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, w)| *w)
                .collect();
            checked += 1;
            if let Err(message) = survives(&name, &without.join(" ")) {
                failures.push(message);
                break;
            }
        }
    }
    panic::set_hook(hook);
    failures.truncate(5);
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
    assert!(checked > 10_000, "{checked}");
}

#[test]
fn compiler_never_panics_on_odd_input() {
    let hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let odd = [
        "",
        " ",
        "\n\n",
        ":",
        "=",
        "(",
        ")",
        "()",
        "(참조",
        "(참조 1",
        "(참조 1,",
        "참조",
        "참조 1",
        "참조 1:",
        "참조 :",
        "정의",
        "정의 가",
        "정의 가:",
        "정의 가: 정수",
        "정의 가: 정수 =",
        "검사",
        "검사 가 =",
        "기준",
        "기준 가:",
        "대상",
        "사건",
        "사건 가:",
        "사건 가: 시작",
        "사건 가: 시작 =",
        "미지",
        "미지 가: 시각 =",
        "미지 가: 시각 = 나 @",
        "조건",
        "목표 가 =",
        "관측",
        "관측 가 =",
        "행위",
        "행위 다",
        "행위 가다 (",
        "행위 가다 (갔다",
        "행위 가다:",
        "규칙",
        "규칙 가:",
        "규칙 가: 참",
        "규칙 가: 참 이면",
        "가:",
        "가: ",
        "면",
        "하면",
        "가 면 다",
        "이렇게",
        "이것이",
        "70%",
        "70% 70%",
        "%",
        "?",
        "@",
        ".",
        "가.",
        "가.나",
        "가 @",
        "1 @ 2",
        "- - - 1",
        "아님",
        "((((",
        "1 + ",
        "1 초 초",
        "999999999999999999999999999999 %",
        "\u{feff}",
        "가\u{0301}",
        "😀",
        "😀이 😀다",
        "\t\t",
        "#",
        "# 주석만",
        "'",
        "\"",
        "\\",
        "대상 가\n가가",
        "대상 가\n행위 하다: 주체\n가가 하면",
        "대상 가\n행위 하다: 주체\n가가 하면 하면 한다",
    ];
    let mut failures = Vec::new();
    for text in odd {
        if let Err(message) = survives("odd", text) {
            failures.push(message);
        }
        // 다른 문맥 뒤에 붙였을 때도 확인합니다.
        let after =
            format!("기준 시계: 시간축\n대상 컵\n행위 떨어지다: 주체\n컵이 떨어졌다\n{text}");
        if let Err(message) = survives("odd-after", &after) {
            failures.push(message);
        }
    }
    panic::set_hook(hook);
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
