# Appendix A. Keywords and diagnostics

*Corresponds to appendices A and B of the Korean edition, which contain the complete list of error codes and the full grammar in EBNF.*

## A.1 Statement keywords

| Keyword | Romanisation | English | Form | Chapter |
| --- | --- | --- | --- | --- |
| `정의` | jeong-ui | define | `정의 name [: type] = expression` | 3 |
| `검사` | geomsa | check | `검사 [name =] expression` | 3 |
| `기준` | gijun | frame | `기준 name: 시간축` or `위치축` | 4 |
| `대상` | daesang | object | `대상 a, b, c` | 4 |
| `사건` | sageon | event | `사건 name: role = value, …` | 4 |
| `미지` | miji | unknown | `미지 name: type` or `미지 name: 시각 = frame @ ?` | 5 |
| `조건` | jogeon | constraint | `조건 [name =] expression` | 5 |
| `목표` | mokpyo | goal | `목표 [name =] expression` | 5 |
| `관측` | gwancheuk | observe | `관측 unknown = value` | 5 |
| `행위` | haeng-wi | action | `행위 verb, verb: role, role` | 6 |
| `참조` | chamjo | reference | `참조 1: text`, cited as `(참조 1)` | 6 |
| `규칙` | gyuchik | rule | `규칙 name: premise 이면 strength [최소로 \| 곱으로]` | 7 |
| `단위` | danwi | unit | `단위 a, b` | 8 |
| `본문` | bonmun | prose | `본문: text` | 8 |
| `-` | | attached line | `- text` under a sentence, reference or prose line | 8 |
| `주제` | juje | topic | `주제 <text>` | 10 |
| `낱말` | natmal | word | `낱말 name: dimension[*][(type)][weight], …` | 10 |
| `장면` | jangmyeon | scene | `장면 name`, then indented lines | 10 |

A line that starts with none of these is a Korean sentence (chapter 6).

## A.2 Types, values and operators

| Korean | English | | Korean | English |
| --- | --- | --- | --- | --- |
| `정수` | integer | | `참` / `거짓` | true / false |
| `논리` | boolean | | `그리고` | and |
| `시간` | duration | | `또는` | or |
| `거리` | distance | | `아님` | not |
| `시각` | time point | | `이면` | if … then (in `규칙`) |
| `위치` | position | | `최소로` | by minimum |
| `대상` | object | | `곱으로` | by product |
| `정도` | degree | | `시간축` / `위치축` | time axis / position axis |

## A.3 Units and number words

| Korean | English | | Korean | English |
| --- | --- | --- | --- | --- |
| `밀리초` | millisecond | | `밀리미터` | millimetre |
| `초` | second | | `센티미터` | centimetre |
| `분` | minute | | `미터` | metre |
| `시간` | hour | | `킬로미터` | kilometre |
| `일` | day | | | |

Number words: `십` 10, `백` 100, `천` 1,000, `만` 10,000, `억` 100,000,000, `조` 1,000,000,000,000, and their combinations (`십만`, `백만`, `천만`, …).

## A.4 Roles

| Korean | English | Used in |
| --- | --- | --- |
| `주체` | agent | events, sentences |
| `대상` | object (acted on) | events, sentences |
| `관측자` | observer | events |
| `상대` | recipient | sentences |
| `도구` | instrument | sentences |
| `시작` / `종료` | start / end | events, sentences |
| `장소` | place | sentences |
| `출발` / `도착` | origin / destination | events, sentences |
| `방식` | manner | sentences (`이렇게` with a reference) |
| `소요` / `이동` | duration / displacement | read-only, derived |

## A.5 Built-in words for scenes

| Word | English | Dimensions |
| --- | --- | --- |
| `보는자리` | viewpoint | `거리` distance, `높이` height, `방향` direction, `화각` angle of view |
| `빛` | light | `방향` direction, `세기` strength, `색` colour, `때` time |
| `배경` | background | `내용` content |
| `날씨` | weather | `상태` state, `세기` strength |

## A.6 Words in compiler output

| Korean | English |
| --- | --- |
| `검사 통과` | check passed |
| `N개` | N items |
| `채움` | fill (score) |
| `빈칸` | blank |
| `보장` / `가능` / `불가능` | guaranteed / possible / impossible |
| `최소` / `곱` | minimum / product |
| `조건문 N행` | conditional on line N |
| `연결망` · `점` · `선` · `덩어리` | graph · node · edge · component |

## A.7 Diagnostic code ranges

The format is `file(line,column): error CODE: message`. The compiler stops at the first error. Warnings use `warning` and do not stop it.

| Range | Stage | Examples |
| --- | --- | --- |
| `K1xx` | Characters and tokens | `K100` unknown symbol |
| `K2xx` | Grammar | `K200` syntax error, `K201` empty file |
| `K3xx` | Names and types | `K301` duplicate name, `K302` undeclared name, `K303` wrong type, `K304` operands do not fit, `K305`–`K308` points and frames, `K315` undeclared unit |
| `K4xx` | Verdicts | `K401` a check is false, `K402` end before start, `K403` a check is not guaranteed, `K404` contradiction, `K405` unsupported constraint |
| `K5xx` | Sentences | `K501` unknown word, `K502` wrong particle form, `K503` undeclared action, `K504` missing role, `K510`–`K512` references, `K520` degree out of range, `K530` tense, `K540`–`K542` rule methods, `K550`–`K552` conditionals |
| `K6xx` | Prose warnings | `K601` duplicated value, `K602` vague word, `K603` unresolved pointer, `K604` name written with a space, `K605` long line |
| `K7xx` | Words and scenes | `K701` duplicate dimension, `K702` undefined word, `K703` unknown dimension, `K704` required dimension empty |

The complete table with causes is in [Appendix A of the Korean edition](../book/부록A-진단-코드.md).
