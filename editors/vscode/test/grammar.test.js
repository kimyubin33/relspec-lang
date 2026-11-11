// 문법 색칠 규칙을 VS Code의 색칠 엔진(vscode-textmate)으로 확인합니다. 실행: npm test
const fs = require("fs");
const path = require("path");
const vsctm = require("vscode-textmate");
const oniguruma = require("vscode-oniguruma");

const grammarPath = path.join(__dirname, "..", "syntaxes", "kpl.tmLanguage.json");
const wasm = fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm")).buffer;
const lib = oniguruma.loadWASM(wasm).then(() => ({
  createOnigScanner: (p) => new oniguruma.OnigScanner(p),
  createOnigString: (s) => new oniguruma.OnigString(s),
}));
const registry = new vsctm.Registry({
  onigLib: lib,
  loadGrammar: async () => vsctm.parseRawGrammar(fs.readFileSync(grammarPath, "utf8"), grammarPath),
});

const source = `# 주석입니다
기준 실험시계: 시간축
정의 지금: 시각 = 실험시계 @ 3 초
미지 반응: 시각 = 실험시계 @ ?
눌림: 유빈이 반응에 버튼을 이렇게 눌렀다 (참조 1, 참조 2)
검사 값 = 깨짐 >= 60% 그리고 아님 거짓
규칙 위험: 깨짐 이면 90% 최소로
컵이 70% 떨어졌다   # 줄 끝 주석
그것이 250 밀리초 == 시작
참조 1: 본문 # 이것은 주석이 아님
시간순서 정의초 참조표
대상 조사자, 조사범위
행위 확인하다 (확인했다), 조사하다: 주체, 대상
미지 가, 나: 시간
조건 요금 >= 0 초
목표 넘침 = 요금 == 0 초
단위 원
조사자가 조사범위를 80% 확인한다 (참조 1): 몇 곳인가? 120만 원 # 주석 아님
첫째: 컵이 떨어졌다: 바닥 기준 == 0
둘째: 컵이 떨어졌다 # 주석: 설명 아님
- 추정치를 만들지 말라 (근거 == 없음)`;

// [줄 번호(0부터), 낱말, 들어 있어야 하는 범위 이름] 또는 [줄, 낱말, null] = 특별한 범위가 없어야 함
const expectations = [
  [0, "# 주석입니다", "comment.line"],
  [1, "기준", "keyword.control.declaration"],
  [1, "실험시계", "entity.name.function"],
  [1, "시간축", "support.type"],
  [2, "정의", "keyword.control.declaration"],
  [2, "지금", "entity.name.function"],
  [2, "시각", "support.type"],
  [2, "=", "keyword.operator.assignment"],
  [2, "@", "keyword.operator.anchor"],
  [2, "3", "constant.numeric"],
  [2, "초", "keyword.other.unit"],
  [3, "?", "constant.language.unknown"],
  [4, "눌림", "entity.name.function"],
  [4, "이렇게", "variable.language.demonstrative"],
  [4, "참조", "keyword.other.reference"],
  [4, "1", "constant.numeric.reference"],
  [4, "유빈이 반응에 버튼을", null],
  [5, "검사", "keyword.control.declaration"],
  [5, ">=", "keyword.operator.comparison"],
  [5, "60", "constant.numeric"],
  [5, "%", "keyword.other.unit"],
  [5, "그리고", "keyword.operator.logical"],
  [5, "아님", "keyword.operator.logical"],
  [5, "거짓", "constant.language"],
  [6, "규칙", "keyword.control.declaration"],
  [6, "이면", "keyword.control.rule"],
  [6, "최소로", "keyword.control.rule"],
  [7, "70", "constant.numeric"],
  [7, "# 줄 끝 주석", "comment.line"],
  [8, "그것이", "variable.language.demonstrative"],
  [8, "250", "constant.numeric"],
  [8, "밀리초", "keyword.other.unit"],
  [8, "==", "keyword.operator.comparison"],
  [9, "참조", "keyword.control.declaration"],
  [9, " 본문 # 이것은 주석이 아님", "string.unquoted.reference"],
  // 예약어를 포함한 긴 이름은 색칠하지 않습니다.
  [10, "시간순서 정의초 참조표", null],
  // 쉼표로 나눈 이름은 모두 선언한 이름입니다.
  [11, "조사자", "entity.name.function"],
  [11, "조사범위", "entity.name.function"],
  [12, "확인하다", "entity.name.function"],
  [12, "조사하다", "entity.name.function"],
  [12, "확인했다", "meta.forms"],
  [13, "가", "entity.name.function"],
  [13, "나", "entity.name.function"],
  [13, "시간", "support.type"],
  // 이름 없는 조건의 첫 낱말은 선언이 아닙니다.
  [14, "조건", "keyword.control.declaration"],
  [14, "요금", null],
  [15, "넘침", "entity.name.function"],
  [16, "원", "entity.name.function"],
  // 문장 뒤의 설명은 글입니다. 안의 '?', 수, '#'을 따로 색칠하지 않습니다.
  [17, "80", "constant.numeric"],
  [17, "참조", "keyword.other.reference"],
  [17, " 몇 곳인가? 120만 원 # 주석 아님", "string.unquoted.note"],
  [18, "첫째", "entity.name.function"],
  [18, " 바닥 기준 == 0", "string.unquoted.note"],
  [19, "둘째", "entity.name.function"],
  [19, "# 주석: 설명 아님", "comment.line"],
  [20, "-", "punctuation.definition.list"],
  [20, " 추정치를 만들지 말라 (근거 == 없음)", "string.unquoted.item"],
];

registry.loadGrammar("source.kpl").then((grammar) => {
  const lines = source.split("\n");
  const tokens = [];
  let stack = vsctm.INITIAL;
  for (const line of lines) {
    const result = grammar.tokenizeLine(line, stack);
    tokens.push(result.tokens.map((t) => ({ text: line.slice(t.startIndex, t.endIndex), scopes: t.scopes })));
    stack = result.ruleStack;
  }
  let failed = 0;
  for (const [row, text, scope] of expectations) {
    const token = tokens[row].find((t) => t.text === text || t.text.trim() === text.trim());
    const scopes = token ? token.scopes.slice(1) : null;
    const ok = token && (scope === null ? scopes.length === 0 : scopes.some((s) => s.startsWith(scope)));
    if (!ok) {
      failed++;
      console.log(`FAIL 줄 ${row} '${text}' 기대 ${scope} 실제 ${JSON.stringify(scopes)}`);
      console.log("   줄의 토큰:", JSON.stringify(tokens[row].map((t) => [t.text, t.scopes.slice(1).join(" ")])));
    }
  }
  console.log(failed === 0 ? `문법 색칠 ${expectations.length}건 통과` : `${failed}건 실패`);
  process.exit(failed === 0 ? 0 : 1);
});
