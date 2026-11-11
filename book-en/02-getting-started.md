# Chapter 2. Getting started

> In this chapter
> - Building the compiler and running the three commands
> - Reading a diagnostic
> - What the compiled output is for
> - The VS Code extension

*Corresponds to chapters 2, 12 and 15 of the Korean edition.*

## 2.1 Install

You need [Rust](https://www.rust-lang.org/tools/install).

```powershell
git clone https://github.com/kimyubin33/relspec-lang.git
cd relspec-lang
cargo build --release
```

The executable is `target\release\aha.exe` on Windows and `target/release/aha` elsewhere. Source files end in `.aha` and are UTF-8 text.

## 2.2 Three commands

| Command | English alias | What it does |
| --- | --- | --- |
| `aha 검사 file.aha` | `aha check` | Checks the file and prints a summary |
| `aha 컴파일 file.aha` | `aha compile` | Checks the file and prints the result as JSON |
| `aha 그래프 file.aha` | `aha graph` | Writes the graph of the file as an HTML page (chapter 9) |

Exit code 0 means the file passed, 1 means a language error, 2 means a problem with the file or the command line.

## 2.3 A first file

```aha
# convert units and check a duration
정의 시작: 시간 = 1 초
정의 종료: 시간 = 시작 + 250 밀리초
검사 시간순서 = 종료 >= 시작
```

| Korean | English |
| --- | --- |
| `정의 시작: 시간 = 1 초` | define `start`: duration = 1 second |
| `정의 종료: 시간 = 시작 + 250 밀리초` | define `end`: duration = start + 250 milliseconds |
| `검사 시간순서 = 종료 >= 시작` | check `order` = end >= start |

```text
검사 통과 | 정의 2개, 검사 1개
```

"Check passed | 2 definitions, 1 check." One statement per line. A line starting with `#` is a comment.

## 2.4 Reading a diagnostic

Make the check false:

```aha 오류=K401
정의 시작: 시간 = 1 초
정의 종료: 시간 = 시작 + 250 밀리초
검사 너무짧음 = 종료 - 시작 > 300 밀리초
```

```text
예제.aha(3,4): error K401: 검사 '너무짧음'의 결과가 거짓입니다.
```

A diagnostic has the form `file(line,column): error CODE: message`. Here: the result of check `too-short` is false, at line 3, column 4. The format is the one Visual Studio and VS Code recognise, so the location is clickable in those tools.

- The compiler stops at the **first error**. Fix the top one and run again.
- The code tells you the stage: `K1xx` characters, `K2xx` grammar, `K3xx` names and types, `K4xx` verdicts, `K5xx` sentences, `K6xx` prose warnings, `K7xx` words and scenes. Appendix A lists the ranges.
- Warnings (`K6xx`) use the word `warning` instead of `error` and do not stop compilation.

## 2.5 The compiled output

`aha compile` prints a JSON document, the **intermediate representation**: every value converted to a base unit (milliseconds, millimetres), every name resolved, every check with its result. It is what another program, or later an AI model, reads instead of the source text. It begins like this:

```text
{
  "languageVersion": "0.14",
  ...
```

Converting once to a fixed form means that whoever reads the result does not have to interpret Korean grammar or units again.

## 2.6 The editor

The repository contains a VS Code extension in `editors/vscode`. It gives `.aha` files:

- colouring of keywords, names, numbers and units,
- errors and warnings underlined while you type,
- the verdicts and ranges shown next to the lines they belong to,
- hover, go-to-definition, an outline, completion,
- a command that opens the graph of the current file.

It works through a **language server**: the editor starts `aha lsp` and talks to it over the Language Server Protocol, so the editor and the command line always use the same compiler. Installation steps are in `editors/vscode/README.md` (Korean).
