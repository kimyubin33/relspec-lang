use std::{env, fs, io, path::Path, process::ExitCode};

mod lsp;
mod report;

const USAGE: &str = "KPL 0.10\n사용법: kpl 검사 <파일.kpl> | kpl 컴파일 <파일.kpl> | kpl lsp";

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    // 언어 서버: 편집기가 실행합니다. 표준 입출력으로 LSP 메시지를 주고받습니다.
    // VS Code는 `kpl lsp --stdio`처럼 통신 방식을 알리는 인자를 덧붙이므로, 첫 인자만 봅니다.
    if matches!(
        args.first().and_then(|arg| arg.to_str()),
        Some("lsp" | "서버")
    ) {
        return lsp::run();
    }
    let [command, filename] = args.as_slice() else {
        if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };

    let json = match command.to_str() {
        Some("검사" | "check") => false,
        Some("컴파일" | "compile") => true,
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let path = Path::new(filename);
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            if error.kind() == io::ErrorKind::InvalidData {
                eprintln!("{}: UTF-8로 저장된 .kpl 파일을 입력하세요.", path.display());
            } else {
                eprintln!("{}: {error}", path.display());
            }
            return ExitCode::from(2);
        }
    };
    let program = match kpl::compile(&source) {
        Ok(program) => program,
        Err(diagnostic) => {
            eprintln!("{}{diagnostic}", path.display());
            return ExitCode::from(1);
        }
    };

    if json {
        match serde_json::to_string_pretty(&program) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("JSON 출력 오류: {error}");
                return ExitCode::from(2);
            }
        }
    } else {
        println!("{}", report::summary(&program));
        for finding in report::findings(&program) {
            println!("{}", finding.text);
        }
        // 본문 점검의 알림. 컴파일은 통과한 것이므로 종료 코드는 0입니다.
        for warning in &program.warnings {
            println!(
                "{}({},{}): warning {}: {}",
                path.display(),
                warning.line,
                warning.column,
                warning.code,
                warning.message
            );
        }
    }

    ExitCode::SUCCESS
}
