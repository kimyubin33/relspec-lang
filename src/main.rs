use std::{env, fs, io, path::Path, process::ExitCode};

const USAGE: &str = "KPL 0.2\n사용법: kpl 검사 <파일.kpl> | kpl 컴파일 <파일.kpl>";

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
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
        let checks = program
            .statements
            .iter()
            .filter(|statement| statement.kind == "검사")
            .count();
        let mut summary = format!(
            "검사 통과 | 정의 {}개, 검사 {checks}개",
            program.statements.len() - checks
        );
        for (label, count) in [
            ("기준", program.frames.len()),
            ("대상", program.entities.len()),
            ("사건", program.events.len()),
        ] {
            if count > 0 {
                summary.push_str(&format!(", {label} {count}개"));
            }
        }
        println!("{summary}");
    }
    ExitCode::SUCCESS
}
