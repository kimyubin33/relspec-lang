# Chapter 4. Time, position and events

> In this chapter
> - The difference between a quantity and a point
> - Reference frames and the `@` operator
> - Objects, events and their roles

*Corresponds to chapters 5 and 6 of the Korean edition.*

## 4.1 Quantities and points

"Two seconds" and "at two seconds" are different kinds of thing. The first is a **quantity**: a length of time. The second is a **point**: a place on a time line. AHAA gives them different types.

| | Quantity | Point |
| --- | --- | --- |
| Time | `시간` duration | `시각` time point |
| Space | `거리` distance | `위치` position |

A point only has meaning relative to a **reference frame** (`기준`): a clock or an axis with an origin. You make a point with `frame @ quantity`.

```aha
기준 실험시계: 시간축
기준 복도: 위치축

정의 던짐: 시각 = 실험시계 @ 1 초
정의 끝: 시각 = 던짐 + 1500 밀리초
정의 문: 위치 = 복도 @ 3 미터

검사 걸린시간 = 끝 - 던짐 == 1500 밀리초
```

| Korean | English |
| --- | --- |
| `기준 실험시계: 시간축` | frame `lab-clock`: a time axis |
| `기준 복도: 위치축` | frame `corridor`: a position axis |
| `정의 던짐: 시각 = 실험시계 @ 1 초` | define `throw`: time point = lab-clock at 1 second |
| `정의 끝: 시각 = 던짐 + 1500 밀리초` | define `end`: time point = throw + 1500 ms |
| `정의 문: 위치 = 복도 @ 3 미터` | define `door`: position = corridor at 3 metres |
| `검사 걸린시간 = 끝 - 던짐 == 1500 밀리초` | check `elapsed` = end − throw == 1500 ms |

## 4.2 What you can compute with points

| Operation | Result | |
| --- | --- | --- |
| point + quantity | point | one second after the throw |
| point − point (same frame) | quantity | the time between two moments |
| point + point | error `K305` | "3 o'clock plus 4 o'clock" means nothing |
| points on different frames | error `K306` | two clocks that were never synchronised |

```aha 오류=K305
기준 실험시계: 시간축
정의 가: 시각 = 실험시계 @ 1 초
정의 나: 시각 = 실험시계 @ 2 초
정의 합: 시각 = 가 + 나
```

Mathematics calls this structure an **affine space**: points can be subtracted to give a vector, and a vector can be added to a point, but points cannot be added to each other. Dates and durations in calendar libraries, and temperatures in Celsius versus temperature differences, follow the same rule.

## 4.3 Objects and events

`대상` (*object*) declares a thing. `사건` (*event*) declares that something happened, and fills **roles**.

```aha
기준 실험시계: 시간축
기준 복도: 위치축

대상 공
대상 유빈

정의 던짐: 시각 = 실험시계 @ 1 초
정의 반응지연: 시간 = 200 밀리초

사건 굴림: 주체 = 유빈, 대상 = 공, 시작 = 던짐, 종료 = 던짐 + 1500 밀리초, 출발 = 복도 @ 0 미터, 도착 = 복도 @ 3 미터
사건 잡기: 주체 = 유빈, 대상 = 공, 시작 = 굴림.종료 + 반응지연

검사 순서 = 잡기.시작 > 굴림.종료
검사 소요제한 = 굴림.소요 <= 2 초
검사 이동거리 = 굴림.이동 == 300 센티미터
검사 같은주체 = 굴림.주체 == 잡기.주체
```

| Korean | English |
| --- | --- |
| `대상 공` / `대상 유빈` | object `ball` / object `Yubin` |
| `사건 굴림: 주체 = 유빈, 대상 = 공, 시작 = …, 종료 = …, 출발 = …, 도착 = …` | event `roll`: agent = Yubin, object = ball, start, end, origin, destination |
| `사건 잡기: … 시작 = 굴림.종료 + 반응지연` | event `catch`: … start = roll.end + reaction-delay |
| `검사 순서 = 잡기.시작 > 굴림.종료` | check `order` = catch.start > roll.end |
| `검사 소요제한 = 굴림.소요 <= 2 초` | check `duration-limit` = roll.duration <= 2 s |
| `검사 이동거리 = 굴림.이동 == 300 센티미터` | check `distance` = roll.displacement == 300 cm |
| `검사 같은주체 = 굴림.주체 == 잡기.주체` | check `same-agent` = roll.agent == catch.agent |

```text
검사 통과 | 정의 2개, 검사 4개, 기준 2개, 대상 2개, 사건 2개
```

### Roles

| Role | English | Type |
| --- | --- | --- |
| `주체` | agent | object |
| `대상` | object (what is acted on) | object |
| `관측자` | observer | object |
| `시작` | start (required) | time point |
| `종료` | end | time point |
| `출발`, `도착` | origin, destination (both or neither) | position |

Two more roles are **derived** and can only be read: `소요` (duration = end − start) and `이동` (displacement = destination − origin). An event without an end is an instant, so its duration is zero.

An event whose end comes before its start is rejected (`K402`).

The idea of describing an event by a fixed set of roles comes from linguistics, where they are called **thematic roles** (agent, patient, source, goal), and from Davidson's proposal in philosophy to treat an event as a thing that such statements are about. Chapter 6 uses the same roles for sentences.

## Summary

- A quantity is a length; a point is a place on a frame. They have different types.
- `frame @ quantity` makes a point. Points on different frames cannot be mixed.
- An event fills roles, and you can read them back with `event.role`.
