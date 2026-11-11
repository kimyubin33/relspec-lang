//! 교재(`book/`)의 모든 예제를 실제 컴파일러로 검사합니다.
//!
//! 코드 블록의 정보 문자열(```뒤의 글)로 기대 결과를 적습니다.
//!
//! - ```` ```kpl ```` : 그 블록만으로 컴파일되어야 합니다. 이후 `이어서` 블록의 앞부분이 됩니다.
//! - ```` ```kpl 오류=K504 ```` : 그 블록만으로 컴파일하면 `K504`가 나와야 합니다.
//! - ```` ```kpl 이어서 ```` : 바로 앞까지 쌓인 블록 뒤에 붙여 컴파일되어야 하며, 이후에도 쌓입니다.
//! - ```` ```kpl 이어서 오류=K504 ```` : 앞까지 쌓인 블록 뒤에 붙이면 `K504`가 나와야 합니다(쌓이지 않음).
//!
//! - ```` ```kpl 경고=K601,K602 ```` : 컴파일되고, 본문 점검의 알림이 적힌 순서대로 정확히 이것들이어야 합니다.
//!   `경고=`가 없는 블록의 알림은 확인하지 않습니다.
//!
//! 그 밖의 블록(```text, ```powershell 등)은 검사하지 않습니다.

use std::fs;

struct Block {
    file: String,
    line: usize,
    info: String,
    body: String,
}

fn blocks() -> Vec<Block> {
    let mut paths: Vec<_> = fs::read_dir("book")
        .expect("book 폴더")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();
    let mut blocks = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path).unwrap();
        let mut current: Option<Block> = None;
        for (index, line) in text.lines().enumerate() {
            match current.as_mut() {
                None => {
                    if let Some(info) = line.strip_prefix("```kpl") {
                        current = Some(Block {
                            file: path.display().to_string(),
                            line: index + 1,
                            info: info.trim().to_string(),
                            body: String::new(),
                        });
                    }
                }
                Some(block) => {
                    if line.starts_with("```") {
                        blocks.push(current.take().unwrap());
                    } else {
                        block.body.push_str(line);
                        block.body.push('\n');
                    }
                }
            }
        }
        assert!(
            current.is_none(),
            "{}: 닫히지 않은 코드 블록",
            path.display()
        );
    }
    blocks
}

#[test]
fn every_book_example_behaves_as_written() {
    let mut context = String::new();
    let mut context_file = String::new();
    let mut checked = 0;
    let mut failures = Vec::new();

    for block in blocks() {
        let continues = block.info.split_whitespace().any(|w| w == "이어서");
        let expected = block
            .info
            .split_whitespace()
            .find_map(|w| w.strip_prefix("오류="))
            .map(str::to_string);
        if continues && context_file != block.file {
            failures.push(format!(
                "{}:{}: '이어서' 블록 앞에 같은 파일의 기본 블록이 없습니다",
                block.file, block.line
            ));
            continue;
        }
        let source = if continues {
            format!("{context}{}", block.body)
        } else {
            block.body.clone()
        };
        let warned = block
            .info
            .split_whitespace()
            .find_map(|w| w.strip_prefix("경고="));
        let result = kpl::compile(&source);
        checked += 1;
        match (&expected, result) {
            (None, Ok(program)) => {
                if let Some(wanted) = warned {
                    let codes: Vec<_> = program.warnings.iter().map(|w| w.code).collect();
                    if codes.join(",") != wanted {
                        failures.push(format!(
                            "{}:{}: 알림이 {wanted}이어야 하지만 [{}]입니다",
                            block.file,
                            block.line,
                            codes.join(",")
                        ));
                    }
                }
                context = source;
                context_file = block.file.clone();
            }
            (None, Err(diagnostic)) => failures.push(format!(
                "{}:{}: 컴파일되어야 하지만 오류가 났습니다: {diagnostic}",
                block.file, block.line
            )),
            (Some(code), Err(diagnostic)) if diagnostic.code == code => {}
            (Some(code), Err(diagnostic)) => failures.push(format!(
                "{}:{}: {code}가 나와야 하지만 다른 오류가 났습니다: {diagnostic}",
                block.file, block.line
            )),
            (Some(code), Ok(_)) => failures.push(format!(
                "{}:{}: {code}가 나와야 하지만 컴파일되었습니다",
                block.file, block.line
            )),
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
    assert!(checked >= 100, "교재 예제가 너무 적습니다: {checked}개");
}
