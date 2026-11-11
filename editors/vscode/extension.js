// RelSpec (KPL) VS Code 확장.
// 문법 색칠은 syntaxes/kpl.tmLanguage.json이 맡고, 이 파일은 언어 서버(`kpl lsp`)를 실행해 연결합니다.

const fs = require("fs");
const path = require("path");
const { execFile } = require("child_process");
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

let graphPanel;
let graphFile;

/** `kpl 그래프 파일 -`을 실행해 연결망 HTML을 받습니다. */
function renderGraph(file) {
  return new Promise((resolve, reject) => {
    execFile(findServer(), ["그래프", file, "-"], { maxBuffer: 64 * 1024 * 1024 }, (error, stdout, stderr) => {
      if (error) {
        reject(new Error(stderr.trim() || error.message));
      } else {
        resolve(stdout);
      }
    });
  });
}

async function refreshGraph() {
  if (!graphPanel || !graphFile) {
    return;
  }
  try {
    graphPanel.webview.html = await renderGraph(graphFile);
  } catch (error) {
    // 오류가 있는 동안에는 마지막으로 성공한 그림을 그대로 둡니다.
    vscode.window.setStatusBarMessage(`RelSpec 연결망: 오류를 고치면 다시 그립니다 (${error.message})`, 6000);
  }
}

async function showGraph() {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== "kpl") {
    vscode.window.showInformationMessage("연결망은 .kpl 파일을 연 상태에서 볼 수 있습니다.");
    return;
  }
  if (editor.document.isUntitled) {
    vscode.window.showInformationMessage("파일을 먼저 저장하세요.");
    return;
  }
  if (editor.document.isDirty) {
    await editor.document.save();
  }
  graphFile = editor.document.uri.fsPath;
  if (!graphPanel) {
    graphPanel = vscode.window.createWebviewPanel("kplGraph", "RelSpec 연결망", vscode.ViewColumn.Beside, {
      enableScripts: true,
      retainContextWhenHidden: true,
    });
    graphPanel.onDidDispose(() => {
      graphPanel = undefined;
    });
    // 연결망에서 줄 번호를 누르면 그 줄로 갑니다.
    graphPanel.webview.onDidReceiveMessage(async (message) => {
      if (message.type === "reveal" && graphFile) {
        const document = await vscode.workspace.openTextDocument(graphFile);
        const position = new vscode.Position(Math.max(0, message.line - 1), 0);
        await vscode.window.showTextDocument(document, {
          viewColumn: vscode.ViewColumn.One,
          selection: new vscode.Range(position, position),
        });
      }
    });
  }
  graphPanel.title = `연결망: ${path.basename(graphFile)}`;
  try {
    graphPanel.webview.html = await renderGraph(graphFile);
    graphPanel.reveal(vscode.ViewColumn.Beside, true);
  } catch (error) {
    vscode.window.showErrorMessage(`연결망을 만들지 못했습니다. 오류를 먼저 고치세요. (${error.message})`);
  }
}

async function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("kpl.showGraph", showGraph),
    vscode.workspace.onDidSaveTextDocument((document) => {
      if (graphPanel && document.uri.fsPath === graphFile) {
        refreshGraph();
      }
    }),
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
