# Chapter 3. Values, units and checks

> In this chapter
> - Definitions and types
> - Built-in units and exact conversion
> - Expressions and their precedence
> - Checks, and the type errors that protect them

*Corresponds to chapters 3 and 4 of the Korean edition.*

## 3.1 Definitions

`정의` (*define*) gives a name to a fixed value. A name can be used on any later line.

```aha
정의 거리한계: 거리 = 2 미터
정의 이동거리: 거리 = 150 센티미터
정의 횟수: 정수 = 3
정의 준비됨: 논리 = 참
검사 한계안 = 이동거리 <= 거리한계
```

| Korean | English |
| --- | --- |
| `정의 거리한계: 거리 = 2 미터` | define `distance-limit`: distance = 2 metres |
| `정의 이동거리: 거리 = 150 센티미터` | define `moved`: distance = 150 centimetres |
| `정의 횟수: 정수 = 3` | define `count`: integer = 3 |
| `정의 준비됨: 논리 = 참` | define `ready`: boolean = true |
| `검사 한계안 = 이동거리 <= 거리한계` | check `within-limit` = moved <= distance-limit |

## 3.2 Types

| Type | English | Example value | Stored as |
| --- | --- | --- | --- |
| `정수` | integer | `3`, `120만` | exact integer |
| `논리` | boolean | `참` (true), `거짓` (false) | |
| `시간` | duration | `250 밀리초`, `2 시간` | milliseconds |
| `거리` | distance | `150 센티미터`, `3 킬로미터` | millimetres |
| `시각` | time point | `clock @ 1 초` (chapter 4) | milliseconds on a frame |
| `위치` | position | `corridor @ 3 미터` (chapter 4) | millimetres on a frame |
| `대상` | object | a declared thing (chapter 4) | |
| `정도` | degree | `80%` (chapter 6) | exact decimal |

## 3.3 Units

| Kind | Units |
| --- | --- |
| Duration | `밀리초` ms, `초` s, `분` min, `시간` h, `일` day |
| Distance | `밀리미터` mm, `센티미터` cm, `미터` m, `킬로미터` km |

All arithmetic is done on exact integers in the base unit, so `150 센티미터` and `1500 밀리미터` are the same value and there is no rounding. This is why the language has no floating-point numbers: a check such as "equal" must mean exactly equal.

Large numbers can be written with the Korean number words: `3만` is 30,000 and `1억 2000만` is 120,000,000. You can declare your own units (won, people, items); chapter 8 shows how.

## 3.4 Expressions

Operators, from lowest to highest precedence:

| Operators | Meaning |
| --- | --- |
| `또는` | or |
| `그리고` | and |
| `==`, `!=` | equal, not equal |
| `<`, `<=`, `>`, `>=` | comparison |
| `+`, `-` | addition, subtraction |
| `*` | multiplication (at least one side must be an integer) |
| `-`, `아님` | negation, not |

Parentheses group as usual. There is no division: a result that is not a whole number of base units could not be stored exactly.

## 3.5 Checks

`검사` (*check*) states something that must be true. If it is false, compilation stops with `K401`.

A check must be a boolean, and the two sides of an operation must fit together. Adding a duration to a distance is rejected before anything is computed:

```aha 오류=K304
정의 가: 시간 = 1 초
정의 나: 거리 = 2 미터
검사 이상함 = 가 + 나 > 0
```

```text
예제.aha(3,12): error K304: '+' 연산에 사용할 수 없는 조합입니다. 왼쪽은 시간, 오른쪽은 거리입니다.
```

"This combination cannot be used with `+`. The left side is a duration, the right side is a distance."

This is **dimensional analysis**, the same idea physicists use to catch mistakes: a quantity carries its dimension, and only quantities of the same dimension can be added or compared. Programming-language research calls it a type system with units of measure.

Using a name before it is defined is also an error:

```aha 오류=K302
검사 빠름 = 지연 <= 300 밀리초
```

"`지연` (delay) has not been declared."

## Summary

- `정의` names a value, `검사` asserts a condition.
- Values are stored exactly, in milliseconds and millimetres.
- Types and units are checked before any value is computed.
