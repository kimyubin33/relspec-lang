// RelSpec (KPL) VS Code 확장.
// 문법 색칠은 syntaxes/kpl.tmLanguage.json이 맡고, 이 파일은 언어 서버(`kpl lsp`)를 실행해 연결합니다.

const fs = require("fs");
const path = require("path");
const vscode = require("vscode");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

let client;

/** 설정, 작업 폴더의 빌드 결과, PATH 순서로 kpl 실행 파일을 찾습니다. */
function findServer() {
  const configured = vscode.workspace.getConfiguration("kpl").get("server.path");
  if (configured) {
    return configured;
  }
  const executable = process.platform === "win32" ? "kpl.exe" : "kpl";
  for (const folder of vscode.workspace.workspaceFolders ?? []) {
    for (const profile of ["release", "debug"]) {
      const candidate = path.join(folder.uri.fsPath, "target", profile, executable);
      if (fs.existsSync(candidate)) {
        return candidate;
      }
    }
  }
  return executable;
}

async function start() {
  const command = findServer();
  const run = { command, args: ["lsp"], transport: TransportKind.stdio };
  client = new LanguageClient(
    "kpl",
    "RelSpec (KPL)",
    { run, debug: run },
    { documentSelector: [{ language: "kpl" }] }
  );
  try {
    await client.start();
  } catch (error) {
    client = undefined;
    vscode.window.showErrorMessage(
      `RelSpec 언어 서버를 실행하지 못했습니다 (${command}). ` +
        "저장소에서 'cargo build --release'로 빌드하거나, 설정 'kpl.server.path'에 kpl 실행 파일의 경로를 적으세요. " +
        `(${error.message ?? error})`
    );
  }
}

async function stop() {
  if (client) {
    const stopping = client;
    client = undefined;
    await stopping.stop();
  }
}

async function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("kpl.restartServer", async () => {
      await stop();
      await start();
    }),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration("kpl.server.path")) {
        await stop();
        await start();
      }
    })
  );
  await start();
}

function deactivate() {
  return stop();
}

module.exports = { activate, deactivate };
