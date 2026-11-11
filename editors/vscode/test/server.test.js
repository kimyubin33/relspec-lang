// VS Code가 쓰는 통신 라이브러리(vscode-jsonrpc)로 언어 서버와 대화해 봅니다. 실행: npm test
// 먼저 저장소에서 `cargo build --release`(또는 `cargo build`)로 aha을 빌드해 두어야 합니다.
const fs = require("fs");
const path = require("path");
const assert = require("assert");
const { spawn } = require("child_process");
const rpc = require("vscode-jsonrpc/node");

const executable = process.platform === "win32" ? "aha.exe" : "aha";
const root = path.join(__dirname, "..", "..", "..");
const binary =
  process.env.AHA_SERVER ??
  ["release", "debug"].map((p) => path.join(root, "target", p, executable)).find((p) => fs.existsSync(p));
if (!binary) {
  console.error("aha 실행 파일이 없습니다. 저장소에서 cargo build --release를 실행하세요.");
  process.exit(1);
}
// VS Code의 언어 클라이언트가 실행하는 것과 같은 인자입니다.
const server = spawn(binary, ["lsp", "--stdio"]);
const connection = rpc.createMessageConnection(
  new rpc.StreamMessageReader(server.stdout),
  new rpc.StreamMessageWriter(server.stdin)
);
const uri = "file:///예제.aha";
const text = `대상 컵
행위 떨어지다: 주체
행위 깨지다: 주체
컵이 70% 떨어졌다
깨짐: 컵이 떨어지면 컵이 80% 깨진다
목표 깨짐심함 = 깨짐 >= 60%
`;
let published = [];
connection.onNotification("textDocument/publishDiagnostics", (params) => published.push(params));
connection.listen();

const wait = (ms) => new Promise((r) => setTimeout(r, ms));
(async () => {
  const init = await connection.sendRequest("initialize", { processId: process.pid, rootUri: null, capabilities: {} });
  assert.strictEqual(init.serverInfo.name, "aha");
  for (const key of ["hoverProvider", "definitionProvider", "documentSymbolProvider", "completionProvider"]) {
    assert.ok(init.capabilities[key], key);
  }
  connection.sendNotification("initialized", {});

  connection.sendNotification("textDocument/didOpen", { textDocument: { uri, languageId: "aha", version: 1, text } });
  await wait(300);
  const judged = published.at(-1).diagnostics;
  assert.ok(judged.some((d) => d.severity === 3 && d.message.includes("조건문 '깨짐': 56%~70%")));
  assert.ok(judged.some((d) => d.message.includes("목표 '깨짐심함': 가능")));

  connection.sendNotification("textDocument/didChange", {
    textDocument: { uri, version: 2 },
    contentChanges: [{ text: text.replace("컵이 70% 떨어졌다", "컵가 70% 떨어졌다") }],
  });
  await wait(300);
  const [error] = published.at(-1).diagnostics;
  assert.strictEqual(error.code, "K502");
  assert.deepStrictEqual(error.range, { start: { line: 3, character: 0 }, end: { line: 3, character: 2 } });

  const hover = await connection.sendRequest("textDocument/hover", { textDocument: { uri }, position: { line: 4, character: 5 } });
  assert.ok(hover.contents.value.includes("역할: **주체**"));
  const definition = await connection.sendRequest("textDocument/definition", { textDocument: { uri }, position: { line: 4, character: 9 } });
  assert.strictEqual(definition.range.start.line, 1);
  const symbols = await connection.sendRequest("textDocument/documentSymbol", { textDocument: { uri } });
  assert.deepStrictEqual(symbols.map((s) => s.name), ["컵", "떨어지다", "깨지다", "깨짐", "깨짐심함"]);

  await connection.sendRequest("shutdown");
  connection.sendNotification("exit");
  server.on("exit", (code) => {
    assert.strictEqual(code, 0);
    console.log("언어 서버 대화 통과");
    process.exit(0);
  });
})().catch((error) => {
  console.error("실패:", error);
  server.kill();
  process.exit(1);
});
