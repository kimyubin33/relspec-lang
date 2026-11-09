use kpl::{CompiledProgram, compile};

fn value(program: &CompiledProgram, name: &str) -> String {
    program
        .statements
        .iter()
        .find(|statement| statement.name == name)
        .expect("expected a compiled declaration")
        .value
        .clone()
}

#[test]
fn exact_integer_arithmetic_carries_and_borrows_across_limbs() {
    let source = "정의 올림: 정수 = 999999999999999999 + 1\n\
                  정의 빌림: 정수 = 1000000000000000000 - 1\n\
                  정의 곱: 정수 = -999999999999999999 * 999999999999999999\n\
                  정의 영: 정수 = -0 - 0\n\
                  검사 부호 = 곱 < -1 그리고 영 == 0";
    let program = compile(source).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(value(&program, "올림"), "1000000000000000000");
    assert_eq!(value(&program, "빌림"), "999999999999999999");
    assert_eq!(
        value(&program, "곱"),
        "-999999999999999998000000000000000001"
    );
    assert_eq!(value(&program, "영"), "0");
}

#[test]
fn normalizes_hangul_names_and_units() {
    let source = "\u{feff}정의 시작: 시간 = 1 초\r\n정의 종료: 시간 = 시작 + 250 밀리초\r\n검사 순서 = 종료 >= 시작";
    let program = compile(source).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(value(&program, "종료"), "1250");
    let ir = serde_json::to_value(&program).unwrap();
    assert_eq!(ir["statements"][1]["unit"], "밀리초");
    assert_eq!(
        ir["statements"][1]["expression"]["operands"][0]["name"],
        "시작"
    );
}

#[test]
fn rejects_incompatible_units_at_the_operator() {
    let diagnostic = compile("정의 잘못: 시간 = 1 초 + 2 미터")
        .err()
        .expect("type mismatch should be rejected");
    assert_eq!(
        (diagnostic.code, diagnostic.line, diagnostic.column),
        ("K304", 1, 17)
    );
}
