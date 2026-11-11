# Chapter 5. Unknowns and goals

> In this chapter
> - Declaring a value you do not know
> - Constraints that narrow it, and the range the compiler computes
> - Goals and the three verdicts: guaranteed, possible, impossible
> - Observations

*Corresponds to chapter 7 of the Korean edition.*

## 5.1 Leaving a value open

Often you do not know a value when you write: when a person will react, how long a step will take. AHAA lets you say so, and still computes what can be known.

| Statement | English | Meaning |
| --- | --- | --- |
| `미지` | unknown | declares a value that is not known |
| `조건` | constraint | an assumption every possible value must satisfy |
| `목표` | goal | a claim to be judged; it does not stop compilation |
| `관측` | observation | records the actual value of an unknown |

```aha
기준 실험시계: 시간축
사건 자극: 시작 = 실험시계 @ 1000 밀리초

미지 반응: 시각 = 실험시계 @ ?
조건 반응은자극뒤 = 반응 >= 자극.시작
조건 지연상한 = 반응 - 자극.시작 <= 200 밀리초

목표 마감준수 = 반응 <= 실험시계 @ 1100 밀리초
목표 넉넉한마감 = 반응 <= 실험시계 @ 1500 밀리초
목표 너무이른마감 = 반응 < 자극.시작
```

| Korean | English |
| --- | --- |
| `사건 자극: 시작 = 실험시계 @ 1000 밀리초` | event `stimulus`: start = lab-clock at 1000 ms |
| `미지 반응: 시각 = 실험시계 @ ?` | unknown `response`: a time point on lab-clock |
| `조건 반응은자극뒤 = 반응 >= 자극.시작` | constraint: response is not before the stimulus |
| `조건 지연상한 = 반응 - 자극.시작 <= 200 밀리초` | constraint: the delay is at most 200 ms |
| `목표 마감준수 = 반응 <= 실험시계 @ 1100 밀리초` | goal `meets-deadline`: response by 1100 ms |
| `목표 넉넉한마감 = …1500 밀리초` | goal `generous-deadline`: response by 1500 ms |
| `목표 너무이른마감 = 반응 < 자극.시작` | goal `too-early`: response before the stimulus |

```text
미지 '반응': 1000 밀리초 ~ 1200 밀리초 (실험시계 기준)
목표 '마감준수': 가능 (보장 여부를 확정하려면 관측 필요: 반응)
목표 '넉넉한마감': 보장
목표 '너무이른마감': 불가능
```

The compiler reports that `response` lies between 1000 and 1200 ms, and gives each goal a verdict.

## 5.2 The three verdicts

Every assignment of values that satisfies all constraints is a **solution**. Here there are 201 of them (1000, 1001, … 1200).

| Verdict | Korean | Meaning | In the example |
| --- | --- | --- | --- |
| **Guaranteed** | 보장 | true in **every** solution | response ≤ 1500 |
| **Possible** | 가능 | true in some solutions, false in others | response ≤ 1100 |
| **Impossible** | 불가능 | true in **no** solution | response < 1000 |

If the constraints have no solution at all, they contradict each other and compilation stops with `K404`.

For a "possible" goal the compiler says which unknown would have to be observed to settle it. A `검사` (check) that involves an unknown must be *guaranteed*, otherwise it fails with `K403`. Use a check for what must hold, and a goal for what you want to know.

In the vocabulary of logic, each solution is a **possible world**; "guaranteed" is necessity and "possible" is possibility, as in modal logic.

## 5.3 How it is computed

Trying every solution works for 201 of them and fails quickly: ten unknowns with a thousand values each give 10³⁰ candidates. The compiler does not enumerate.

Every constraint AHAA accepts has the form *x − y ≤ c*: a bound on the **difference** of two unknowns (a bound on a single unknown is a difference from a fixed origin). A set of such constraints is a **simple temporal network**, introduced by Dechter, Meiri and Pearl in 1991. Draw each constraint as an edge of weight *c*; then

- the constraints are consistent exactly when the graph has no cycle of negative total weight, and
- the tightest bound on any difference is the shortest path between the two nodes.

Both are computed with a standard shortest-path algorithm in time polynomial in the number of unknowns, and the result is exact, not an approximation. The repository tests this against brute-force enumeration on small cases.

The price is a restricted form. Constraints with `!=`, with `또는` (or), or with sums and products of unknowns are outside it and are rejected with `K405`, so that the compiler never gives a verdict it cannot back.

## 5.4 Observations

`관측` (*observe*) records what actually happened. The unknown becomes a fixed value and the verdicts are settled.

```aha
기준 실험시계: 시간축
사건 자극: 시작 = 실험시계 @ 1000 밀리초
미지 반응: 시각 = 실험시계 @ ?
조건 지연상한 = 반응 - 자극.시작 <= 200 밀리초
관측 반응 = 실험시계 @ 1080 밀리초
목표 마감준수 = 반응 <= 실험시계 @ 1100 밀리초
```

```text
목표 '마감준수': 보장
```

An observation that contradicts the constraints is an error (`K404`): the assumptions were wrong, and the file says so.

## Summary

- `미지` leaves a value open; `조건` narrows it; the compiler computes the range.
- A `목표` is judged guaranteed, possible or impossible.
- The computation is exact because the constraints are limited to differences.
