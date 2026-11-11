# Chapter 6. Sentences

> In this chapter
> - Declaring actions and writing Korean sentences
> - How particles decide roles
> - References for "like this"
> - Degree: how much something is the case
> - Tense

*Corresponds to chapters 8, 9, 10 and 11 of the Korean edition.*

## 6.1 A sentence is an event

Chapter 4 wrote an event as a list of roles. AHAA also reads an ordinary Korean sentence and builds the same event from it. Any line that does not start with a keyword is a sentence.

```aha
대상 유빈
대상 공
행위 굴리다: 주체, 대상

유빈이 공을 굴렸다
```

| Korean | English |
| --- | --- |
| `행위 굴리다: 주체, 대상` | action `roll`: requires an agent and an object |
| `유빈이 공을 굴렸다` | Yubin rolled the ball |

`행위` (*action*) declares a verb in its dictionary form (ending in `-다`) and lists the roles a sentence with that verb must fill. The agent is always required.

Leave out a required role and the sentence is rejected:

```aha 오류=K504
대상 유빈
대상 공
행위 굴리다: 주체, 대상

유빈이 굴렸다
```

"The action `굴리다` needs the role `대상` (object)." The sentence "Yubin rolled" is fine between people. Here the missing object is caught.

## 6.2 Particles decide roles

Korean marks the function of a noun with a **particle** attached to its end, where English uses word order and prepositions. AHAA reads the particle together with the type of the noun.

| Particle | Type of the noun | Role |
| --- | --- | --- |
| `이`/`가`, `은`/`는` | object | agent (주체) |
| `을`/`를` | object | object (대상) |
| `에게`, `한테` | object | recipient (상대) |
| `으로`/`로` | object | instrument (도구) |
| `에` | time point | start (시작) |
| `에` | position | place (장소) |
| `부터` | time point / position | start / origin (출발) |
| `까지` | time point / position | end (종료) / destination (도착) |
| `에서` | position | place (장소) |

The same particle `에` means "at a time" or "at a place"; the type of the noun settles which. Because the particle carries the role, word order is free, as it is in Korean.

Each pair such as `이`/`가` is one particle with two forms: the first follows a syllable that ends in a consonant, the second a syllable that ends in a vowel. The compiler checks the form (`K502`), and it recognises the conjugated verb (`굴렸다`, `굴린다`, `굴리겠다`) as the declared `굴리다`.

A name that was never declared is an error (`K501`). This is the point of the sentence grammar: every noun in a sentence is something you declared, with a type.

## 6.3 References: what "like this" points to

Words such as `이렇게` (*like this*) point at something outside the sentence. AHAA requires the sentence to cite what it points to.

```aha
대상 유빈
대상 설계안
행위 설계하다: 주체, 대상

유빈이 설계안을 이렇게 설계했다 (참조 1)

참조 1: 모듈을 세 층으로 나누고, 각 층은 바로 아래 층만 참조한다
```

| Korean | English |
| --- | --- |
| `유빈이 설계안을 이렇게 설계했다 (참조 1)` | Yubin designed the plan like this (ref 1) |
| `참조 1: …` | ref 1: split the modules into three layers; each layer refers only to the one below |

Without `(참조 1)` the sentence fails with `K510`; citing a reference that does not exist fails with `K511`. The text of a reference is free prose. The compiler guarantees that the pointer leads somewhere, not that the prose is clear (chapter 8 adds a check on the prose itself).

## 6.4 Degree

Many statements are not simply true or false. "The cup fell" may hold to some extent. In AHAA you write the extent as a percentage inside the sentence.

```aha
대상 컵
행위 떨어지다: 주체

떨어짐: 컵이 70% 떨어졌다
정의 공존: 정도 = 떨어짐 그리고 아님 떨어짐
검사 떨어지되떨어지지않음 = 공존 == 30%
```

| Korean | English |
| --- | --- |
| `떨어짐: 컵이 70% 떨어졌다` | `fell`: the cup fell, to degree 70% (the sentence is named `fell`) |
| `정의 공존: 정도 = 떨어짐 그리고 아님 떨어짐` | define `both`: degree = fell and not fell |
| `검사 … = 공존 == 30%` | check: `both` equals 30% |

A name followed by a colon at the start of a sentence names the sentence; its value is its degree.

| Operation | Definition | With A = 70%, B = 40% |
| --- | --- | --- |
| `A 그리고 B` (and) | min(A, B) | 40% |
| `A 또는 B` (or) | max(A, B) | 70% |
| `아님 A` (not) | 100% − A | 30% |

This is **fuzzy logic** as defined by Zadeh in 1965. Notice the check above: "fell and not fell" is 30%, not zero. In classical logic a statement and its negation cannot both hold; with degrees they can, partly. The law of non-contradiction and the law of the excluded middle are the two classical laws that fuzzy logic gives up.

A degree is **not a probability**. 70% does not mean "it fell in 70% of cases". It means it fell to that extent, once. Degrees cannot be added or multiplied in expressions (`K304`), and a degree outside 0% to 100% is an error (`K520`).

## 6.5 Tense

If a file defines a time point named `지금` (*now*), the tense of the verb is checked against the time of the event.

| Tense | Marker | Meaning |
| --- | --- | --- |
| Past | `-았/었-` (눌렀다, *pressed*) | the event ended by now |
| Present | `-ㄴ다/-는다` (누른다, *presses*) | now lies inside the event |
| Future | `-겠-` (누르겠다, *will press*) | the event starts after now |

```aha
기준 실험시계: 시간축
대상 유빈
대상 버튼
대상 화면
행위 켜지다: 주체, 시작
행위 누르다: 주체, 대상, 시작

정의 지금: 시각 = 실험시계 @ 3 초
정의 점등: 시각 = 실험시계 @ 1000 밀리초
미지 반응: 시각 = 실험시계 @ ?

자극: 화면이 점등에 켜졌다
눌림: 유빈이 반응에 버튼을 눌렀다

조건 자극뒤 = 눌림.시작 >= 자극.시작
조건 지연상한 = 눌림.시작 - 자극.시작 <= 2500 밀리초

목표 여유 = 눌림.시작 <= 실험시계 @ 3 초
```

| Korean | English |
| --- | --- |
| `정의 지금: 시각 = 실험시계 @ 3 초` | define `now` = lab-clock at 3 s |
| `자극: 화면이 점등에 켜졌다` | `stimulus`: the screen turned on at `light-on` |
| `눌림: 유빈이 반응에 버튼을 눌렀다` | `press`: Yubin pressed the button at `response` |
| `목표 여유 = 눌림.시작 <= 실험시계 @ 3 초` | goal `in-time`: the press started by 3 s |

```text
미지 '반응': 1000 밀리초 ~ 3000 밀리초 (실험시계 기준)
목표 '여유': 보장
```

The delay limit alone would allow a response as late as 3500 ms. The verb is in the past tense, so the press has already happened: the range is cut at `now`, 3000 ms, and the goal is guaranteed. The tense is treated as information, not decoration.

A tense that contradicts fixed times is an error (`K530`). The analysis follows Reichenbach's 1947 description of tense as a relation between the time of speaking and the time of the event.

## Summary

- A sentence is an event; particles decide which noun fills which role.
- A required role that is missing is an error. A pointing word must cite a reference.
- A percentage in a sentence is a degree, combined with minimum, maximum and complement.
- Tense is a constraint between `지금` and the event.
