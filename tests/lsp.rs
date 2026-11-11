//! 언어 서버(`aha lsp`)를 실제로 실행해 LSP 메시지를 주고받습니다.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

struct Client {
    child: Child,
    stdin: Option<ChildStdin>,
    incoming: Receiver<Value>,
    next_id: u64,
}

const URI: &str = "file:///예제.aha";

impl Client {
    fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_aha"))
            // VS Code의 언어 클라이언트는 표준 입출력으로 연결할 때 `--stdio`를 덧붙입니다.
            .args(["lsp", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("aha lsp 실행");
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (sender, incoming) = mpsc::channel();
        // 읽기는 따로 돌려서, 서버가 답하지 않아도 시험이 멈추지 않고 실패하게 합니다.
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut length = None;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let line = line.trim_end();
                    if line.is_empty() {
                        break;
                    }
                    if let Some(value) = line.strip_prefix("Content-Length:") {
                        length = value.trim().parse::<usize>().ok();
                    }
                }
                let Some(length) = length else { return };
                let mut body = vec![0; length];
                if reader.read_exact(&mut body).is_err() {
                    return;
                }
                let Ok(message) = serde_json::from_slice(&body) else {
                    return;
                };
                if sender.send(message).is_err() {
                    return;
                }
            }
        });
        let mut client = Self {
            child,
            stdin,
            incoming,
            next_id: 1,
        };
        let reply = client.request("initialize", json!({ "capabilities": {} }));
        assert_eq!(reply["result"]["serverInfo"]["name"], "aha");
        client.notify("initialized", json!({}));
        client
    }

    fn send(&mut self, message: Value) {
        let body = message.to_string();
        let stdin = self.stdin.as_mut().unwrap();
        write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        stdin.flush().unwrap();
    }

    fn receive(&mut self) -> Value {
        self.incoming
            .recv_timeout(Duration::from_secs(20))
            .expect("서버가 답하지 않았습니다")
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let reply = self.receive();
        assert_eq!(reply["id"], id, "{reply}");
        reply
    }

    /// 문서를 열거나 고치고, 서버가 보낸 진단 목록을 돌려줍니다.
    fn open(&mut self, text: &str) -> Vec<Value> {
        self.notify(
            "textDocument/didOpen",
            json!({ "textDocument": { "uri": URI, "languageId": "aha", "version": 1, "text": text } }),
        );
        self.diagnostics()
    }

    fn change(&mut self, text: &str) -> Vec<Value> {
        self.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": URI, "version": 2 }, "contentChanges": [{ "text": text }] }),
        );
        self.diagnostics()
    }

    fn diagnostics(&mut self) -> Vec<Value> {
        let message = self.receive();
        assert_eq!(
            message["method"], "textDocument/publishDiagnostics",
            "{message}"
        );
        assert_eq!(message["params"]["uri"], URI);
        message["params"]["diagnostics"].as_array().unwrap().clone()
    }

    fn at(&mut self, method: &str, line: u64, character: u64) -> Value {
        self.request(
            method,
            json!({ "textDocument": { "uri": URI }, "position": { "line": line, "character": character } }),
        )["result"]
            .clone()
    }

    /// 종료 절차를 밟고 종료 코드를 돌려줍니다.
    fn stop(mut self) -> i32 {
        let reply = self.request("shutdown", Value::Null);
        assert!(reply["result"].is_null());
        self.notify("exit", Value::Null);
        self.stdin.take();
        self.child.wait().unwrap().code().unwrap()
    }
}

fn range(value: &Value) -> (u64, u64, u64, u64) {
    let n = |v: &Value| v.as_u64().unwrap();
    (
        n(&value["start"]["line"]),
        n(&value["start"]["character"]),
        n(&value["end"]["line"]),
        n(&value["end"]["character"]),
    )
}

const DOCUMENT: &str = "기준 실험시계: 시간축\n\
대상 유빈\n\
대상 버튼\n\
대상 컵\n\
행위 누르다: 주체, 대상, 시작\n\
행위 떨어지다: 주체\n\
행위 깨지다: 주체\n\
정의 지금: 시각 = 실험시계 @ 3 초\n\
미지 반응: 시각 = 실험시계 @ ?\n\
눌림: 유빈이 반응에 버튼을 이렇게 눌렀다 (참조 1)\n\
조건 지연상한 = 눌림.시작 <= 실험시계 @ 3500 밀리초\n\
목표 마감 = 눌림.시작 <= 실험시계 @ 3 초\n\
컵이 70% 떨어졌다\n\
깨짐: 컵이 떨어지면 컵이 80% 깨진다\n\
목표 깨짐심함 = 깨짐 >= 60%\n\
목표 안됨 = 깨짐 >= 90%\n\
참조 1: 검지로 한 번 짧게\n";

#[test]
fn announces_capabilities_and_shuts_down_cleanly() {
    let mut client = Client::start();
    let reply = client.request("initialize", json!({}));
    let capabilities = &reply["result"]["capabilities"];
    assert_eq!(capabilities["textDocumentSync"], 1);
    for provider in [
        "hoverProvider",
        "definitionProvider",
        "documentSymbolProvider",
    ] {
        assert_eq!(capabilities[provider], true, "{provider}");
    }
    assert!(capabilities["completionProvider"].is_object());
    assert_eq!(
        reply["result"]["serverInfo"]["version"],
        aha::LANGUAGE_VERSION
    );
    assert_eq!(client.stop(), 0);
}

#[test]
fn exit_without_shutdown_is_an_error() {
    let mut client = Client::start();
    client.notify("exit", Value::Null);
    client.stdin.take();
    assert_eq!(client.child.wait().unwrap().code(), Some(1));
}

#[test]
fn reports_errors_as_you_type() {
    let mut client = Client::start();

    // 주어가 없는 문장: 행위 위에 K504.
    let diagnostics = client.open("대상 유빈\n행위 설계하다: 주체\n설계했다\n");
    assert_eq!(diagnostics.len(), 1);
    let error = &diagnostics[0];
    assert_eq!(error["severity"], 1);
    assert_eq!(error["code"], "K504");
    assert_eq!(error["source"], "aha");
    assert_eq!(range(&error["range"]), (2, 0, 2, 4));
    assert!(error["message"].as_str().unwrap().contains("주체"));

    // 고치면 오류가 사라집니다.
    let diagnostics = client.change("대상 유빈\n행위 설계하다: 주체\n유빈이 설계했다\n");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    // 쓰는 도중의 문서도 진단을 받습니다.
    for partial in ["정의", "정의 가", "정의 가: 정수 =", "가: ", "(참조", ""] {
        let diagnostics = client.change(partial);
        assert_eq!(diagnostics.len(), 1, "{partial}");
        assert_eq!(diagnostics[0]["severity"], 1);
    }

    // UTF-16에서 두 단위인 글자 뒤의 위치도 맞습니다.
    let diagnostics = client.change("# 😀\n검사 틀림 = 1 > 2\n");
    assert_eq!(diagnostics[0]["code"], "K401");
    assert_eq!(range(&diagnostics[0]["range"]), (1, 3, 1, 5));

    // 닫으면 진단을 지웁니다.
    client.notify(
        "textDocument/didClose",
        json!({ "textDocument": { "uri": URI } }),
    );
    assert!(client.diagnostics().is_empty());
    assert_eq!(client.stop(), 0);
}

#[test]
fn shows_judgements_on_their_lines() {
    let mut client = Client::start();
    let diagnostics = client.open(DOCUMENT);
    let find = |needle: &str| {
        diagnostics
            .iter()
            .find(|d| d["message"].as_str().unwrap().contains(needle))
            .unwrap_or_else(|| panic!("'{needle}' 없음: {diagnostics:?}"))
    };

    let unknown = find("미지 '반응'");
    assert_eq!(unknown["severity"], 3);
    assert_eq!(range(&unknown["range"]).0, 8);
    assert!(unknown["message"].as_str().unwrap().contains("3000 밀리초"));

    let guaranteed = find("목표 '마감': 보장");
    assert_eq!(
        (
            guaranteed["severity"].as_u64(),
            range(&guaranteed["range"]).0
        ),
        (Some(3), 11)
    );

    let rule = find("조건문 '깨짐': 56%~70%");
    assert_eq!(range(&rule["range"]), (13, 0, 13, 22));

    let possible = find("목표 '깨짐심함': 가능");
    assert!(possible["message"].as_str().unwrap().contains("깨짐"));

    // 불가능한 목표는 경고로 표시합니다.
    let impossible = find("목표 '안됨': 불가능");
    assert_eq!(impossible["severity"], 2);
    assert_eq!(range(&impossible["range"]).0, 15);
    assert_eq!(client.stop(), 0);
}

#[test]
fn hover_explains_names_roles_and_verbs() {
    let mut client = Client::start();
    client.open(DOCUMENT);
    let text = |hover: &Value| {
        hover["contents"]["value"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };

    // 정의한 이름
    let hover = client.at("textDocument/hover", 7, 4);
    assert!(
        text(&hover).contains("**정의** `지금`: 시각 = 3000 밀리초 (실험시계 기준)"),
        "{hover}"
    );
    assert_eq!(range(&hover["range"]), (7, 3, 7, 5));

    // 문장 속의 '유빈이': 역할과 대상
    let hover = client.at("textDocument/hover", 9, 5);
    let shown = text(&hover);
    assert!(
        shown.contains("역할: **주체**") && shown.contains("**대상** `유빈`"),
        "{shown}"
    );

    // 활용형 '눌렀다' → 행위 '누르다'
    let hover = client.at("textDocument/hover", 9, 21);
    let shown = text(&hover);
    assert!(
        shown.contains("`눌렀다` → 행위 `누르다`") && shown.contains("필수 역할 주체, 대상, 시작"),
        "{shown}"
    );

    // 미지값, 목표, 조건문은 판정 결과를 보여 줍니다.
    assert!(text(&client.at("textDocument/hover", 8, 4)).contains("미지 '반응'"));
    assert!(text(&client.at("textDocument/hover", 14, 4)).contains("목표 '깨짐심함': 가능"));
    assert!(text(&client.at("textDocument/hover", 14, 12)).contains("조건문 '깨짐': 56%~70%"));
    // 조건문의 전제 '떨어지면'
    assert!(text(&client.at("textDocument/hover", 13, 9)).contains("행위 `떨어지다`"));

    // 예약어는 형식을 보여 주고, 빈 곳에서는 아무것도 보여 주지 않습니다.
    assert!(text(&client.at("textDocument/hover", 7, 0)).contains("정의 이름: 타입 = 식"));
    assert!(
        client.at("textDocument/hover", 0, 100).is_null()
            || !text(&client.at("textDocument/hover", 0, 100)).is_empty()
    );
    assert!(client.at("textDocument/hover", 99, 0).is_null());

    // 오류가 생겨도 마지막으로 성공한 설명을 계속 보여 줍니다.
    let broken = DOCUMENT.replace("컵이 70% 떨어졌다", "컵 70% 떨어졌다");
    assert_eq!(client.change(&broken)[0]["code"], "K501");
    assert!(text(&client.at("textDocument/hover", 7, 4)).contains("**정의** `지금`"));
    assert_eq!(client.stop(), 0);
}

#[test]
fn goes_to_definitions() {
    let mut client = Client::start();
    client.open(DOCUMENT);

    // '유빈이' → 대상 유빈
    let target = client.at("textDocument/definition", 9, 5);
    assert_eq!(target["uri"], URI);
    assert_eq!(range(&target["range"]), (1, 3, 1, 5));

    // '눌림.시작'의 눌림 → 이름 붙은 문장
    let target = client.at("textDocument/definition", 10, 10);
    assert_eq!(range(&target["range"]), (9, 0, 9, 2));

    // '(참조 1)' → '참조 1:' 줄. '참조'에서도, 번호에서도 됩니다.
    for character in [26, 28] {
        let target = client.at("textDocument/definition", 9, character);
        assert_eq!(range(&target["range"]), (16, 0, 16, 4), "{character}");
    }

    // 활용형 '눌렀다' → 행위 누르다
    let target = client.at("textDocument/definition", 9, 21);
    assert_eq!(range(&target["range"]), (4, 3, 4, 6));

    // '실험시계' → 기준
    let target = client.at("textDocument/definition", 7, 13);
    assert_eq!(range(&target["range"]), (0, 3, 0, 7));

    assert!(client.at("textDocument/definition", 7, 0).is_null());
    assert_eq!(client.stop(), 0);
}

#[test]
fn lists_symbols_and_completions_even_with_errors() {
    let mut client = Client::start();
    // 마지막 줄에 오류가 있어도 개요와 자동 완성은 지금 문서에서 읽습니다.
    let diagnostics = client.open("기준 시계: 시간축\n대상 컵\n정의 가: 정수 = 1\n깨짐: 컵이\n참조 3: 메모\n정의 나: 정수 =\n");
    assert_eq!(diagnostics[0]["severity"], 1);

    let symbols = client.request(
        "textDocument/documentSymbol",
        json!({ "textDocument": { "uri": URI } }),
    );
    let listed: Vec<(String, String)> = symbols["result"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s["detail"].as_str().unwrap().to_string(),
                s["name"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let expected = [
        ("기준", "시계"),
        ("대상", "컵"),
        ("정의", "가"),
        ("문장", "깨짐"),
        ("참조", "참조 3"),
        ("정의", "나"),
    ];
    assert_eq!(listed.len(), expected.len(), "{listed:?}");
    for (got, want) in listed.iter().zip(expected) {
        assert_eq!((got.0.as_str(), got.1.as_str()), want);
    }
    assert_eq!(range(&symbols["result"][0]["selectionRange"]), (0, 3, 0, 5));

    let completion = client.at("textDocument/completion", 5, 0);
    let labels: Vec<&str> = completion
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect();
    for label in [
        "정의",
        "목표",
        "규칙",
        "시각",
        "정도",
        "밀리초",
        "그리고",
        "최소로",
        "시계",
        "컵",
        "가",
        "나",
        "깨짐",
    ] {
        assert!(labels.contains(&label), "{label} 없음");
    }
    assert!(!labels.contains(&"참조 3"));
    assert_eq!(client.stop(), 0);
}

#[test]
fn unknown_requests_get_errors_and_unknown_notifications_are_ignored() {
    let mut client = Client::start();
    client.notify("$/setTrace", json!({ "value": "off" }));
    client.notify("workspace/didChangeConfiguration", json!({}));
    let reply = client.request("textDocument/formatting", json!({}));
    assert_eq!(reply["error"]["code"], -32601);
    // 열지 않은 문서에 대한 요청에도 멈추지 않습니다.
    assert!(client.at("textDocument/hover", 0, 0).is_null());
    assert!(client.at("textDocument/definition", 0, 0).is_null());
    assert_eq!(client.stop(), 0);
}

#[test]
fn understands_the_shorter_forms() {
    let mut client = Client::start();
    let diagnostics = client.open(
        "대상 신청자, 강의실\n\
         행위 예약하다, 승인하다: 주체, 대상\n\
         단위 원, 명\n\
         신청자가 강의실을 예약한다: 이틀 전까지 신청한다\n\
         정의 최대요금 = 12만 원\n\
         미지 요금: 원\n\
         조건 요금 >= 0 원\n\
         목표 요금 <= 최대요금\n",
    );
    // 이름 없는 목표의 판정이 그 줄에 표시됩니다.
    let goal = diagnostics
        .iter()
        .find(|d| d["range"]["start"]["line"] == 7)
        .expect("goal finding");
    assert!(
        goal["message"]
            .as_str()
            .unwrap()
            .contains("목표 '요금 <= 최대요금': 가능"),
        "{goal}"
    );

    // 쉼표 뒤의 이름으로도 갑니다: '강의실을' → 대상 줄의 두 번째 이름
    let target = client.at("textDocument/definition", 3, 6);
    assert_eq!(range(&target["range"]), (0, 8, 0, 11));
    // '예약한다' → 행위 줄의 첫 번째, 조건의 '요금' → 미지 줄 (조건 줄 자신이 아님)
    let target = client.at("textDocument/definition", 3, 11);
    assert_eq!(range(&target["range"]), (1, 3, 1, 7));
    let target = client.at("textDocument/definition", 6, 4);
    assert_eq!(range(&target["range"]), (5, 3, 5, 5));

    // 단위에 마우스를 올리면 설명이 나옵니다.
    let hover = client.at("textDocument/hover", 2, 3);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap_or_default()
            .contains("**단위** `원`"),
        "{hover}"
    );
    assert_eq!(client.stop(), 0);
}

#[test]
fn colours_names_in_bodies_and_reports_body_warnings() {
    let mut client = Client::start();
    let diagnostics = client.open(
        "대상 신청자, 강의실\n\
         행위 예약하다: 주체, 대상\n\
         단위 원\n\
         정의 기본요금 = 3만 원\n\
         신청자가 강의실을 예약한다: 기본요금을 낸다\n\
         본문: 신청자는 3만 원을 적절히 낸다\n",
    );
    // 본문 점검의 알림은 그 낱말의 범위에 경고로 표시됩니다.
    let found: Vec<_> = diagnostics
        .iter()
        .filter(|d| d["severity"] == 2)
        .map(|d| (d["code"].as_str().unwrap().to_string(), range(&d["range"])))
        .collect();
    assert_eq!(
        found,
        [
            ("K601".to_string(), (5, 9, 5, 13)),
            ("K602".to_string(), (5, 15, 5, 18)),
        ]
    );

    // 본문 속의 선언한 이름: (줄 차이, 열 차이, 길이, 종류, 수식)
    let tokens = client.request(
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": URI } }),
    );
    assert_eq!(
        tokens["result"]["data"],
        json!([4, 16, 4, 0, 0, 1, 4, 3, 0, 0])
    );

    // 본문 속의 이름에서도 설명과 정의로 이동이 됩니다.
    let hover = client.at("textDocument/hover", 4, 17);
    assert!(
        hover["contents"]["value"]
            .as_str()
            .unwrap_or_default()
            .contains("**정의** `기본요금`: 원 = 30000 원"),
        "{hover}"
    );
    let target = client.at("textDocument/definition", 5, 5);
    assert_eq!(range(&target["range"]), (0, 3, 0, 6));
    assert_eq!(client.stop(), 0);
}
