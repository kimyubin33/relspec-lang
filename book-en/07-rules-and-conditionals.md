# Chapter 7. Rules and conditionals

> In this chapter
> - Carrying a degree from one statement to another
> - Two ways of combining: minimum and product
> - Why the compiler reports both
> - Writing a rule as a conditional sentence

*Corresponds to chapters 13 and 14 of the Korean edition.*

## 7.1 Rules

A rule says: to the extent that the premise holds, the conclusion holds, with a certain **strength**.

```aha
대상 컵
대상 바닥
행위 떨어지다: 주체
행위 젖다: 주체

떨어짐: 컵이 70% 떨어졌다
젖음: 바닥이 40% 젖었다

규칙 깨짐: 떨어짐 이면 80%
규칙 미끄러짐: 젖음 이면 90% 곱으로
규칙 다침: 깨짐 그리고 미끄러짐 이면 50%

목표 깨짐확실 = 깨짐 >= 50%
목표 깨짐심함 = 깨짐 >= 60%
```

| Korean | English |
| --- | --- |
| `떨어짐: 컵이 70% 떨어졌다` | `fell`: the cup fell, 70% |
| `젖음: 바닥이 40% 젖었다` | `wet`: the floor got wet, 40% |
| `규칙 깨짐: 떨어짐 이면 80%` | rule `broken`: if `fell`, then with strength 80% |
| `규칙 미끄러짐: 젖음 이면 90% 곱으로` | rule `slippery`: if `wet`, then 90%, by product |
| `규칙 다침: 깨짐 그리고 미끄러짐 이면 50%` | rule `hurt`: if `broken` and `slippery`, then 50% |
| `목표 깨짐확실 = 깨짐 >= 50%` | goal: `broken` is at least 50% |
| `목표 깨짐심함 = 깨짐 >= 60%` | goal: `broken` is at least 60% |

```text
규칙 '깨짐': 56%~70% (최소 70%, 곱 56%)
규칙 '미끄러짐': 36% (곱으로 계산)
규칙 '다침': 18%~36% (최소 36%, 곱 18%)
목표 '깨짐확실': 보장
목표 '깨짐심함': 가능 (규칙의 계산 방식에 따라 다름: 깨짐)
```

## 7.2 Minimum and product

There are two standard ways to combine the degree of the premise with the strength of the rule.

| Method | Keyword | Formula | Premise 70%, strength 80% |
| --- | --- | --- | --- |
| Minimum (Mamdani) | `최소로` | min(premise, strength) | 70% |
| Product (Larsen) | `곱으로` | premise × strength | 56% |

Both are in common use in fuzzy control, and neither is "the correct one". They answer slightly different questions: minimum treats the strength as a ceiling, product treats it as a discount.

If you write the method, the compiler uses it. If you do not, it **computes both and reports the range**. That is what `56%~70%` means for the rule `broken`.

The verdicts of chapter 5 then apply to the choice of method:

- `broken >= 50%` holds under both methods: **guaranteed**.
- `broken >= 60%` holds under minimum (70%) and fails under product (56%): **possible**, and the message says it depends on the method of the rule `broken`.

The design choice here is to not pick a method silently. A text whose conclusion flips with the method is under-specified, and the compiler says where. A `검사` whose truth depends on the method fails with `K540`.

## 7.3 Conditional sentences

The same thing can be written as a sentence. The verb ending `-면` (or `-으면` after a consonant) means *if*.

```aha
대상 컵
대상 바닥
행위 떨어지다: 주체
행위 깨지다: 주체
행위 젖다: 주체

컵이 70% 떨어졌다

깨짐: 컵이 떨어지면 컵이 80% 깨진다
컵이 깨지면 바닥이 90% 젖는다 곱으로

목표 깨짐심함 = 깨짐 >= 60%
```

| Korean | English |
| --- | --- |
| `컵이 70% 떨어졌다` | the cup fell, 70% (a fact) |
| `깨짐: 컵이 떨어지면 컵이 80% 깨진다` | `broken`: if the cup falls, the cup breaks, strength 80% |
| `컵이 깨지면 바닥이 90% 젖는다 곱으로` | if the cup breaks, the floor gets wet, strength 90%, by product |

How a conditional is read:

- The part before `-면` is the **premise**, the part after is the **conclusion**.
- The percentage in the conclusion is the strength of the rule.
- The premise is matched against a **fact stated earlier** with the same action and the same roles; its degree is used.
- The conclusion becomes a new fact, so the next conditional can use it as its premise.
- If two conditionals reach the same conclusion, a later premise uses the larger of the two degrees.

A conditional whose premise matches no earlier fact is an error (`K551`): the chain has a hole in it. So is a fact that appears only after the conditional that needs it (`K552`). Reading top to bottom, every step must rest on something already said.

This is forward chaining, the oldest inference method of rule-based systems, applied to degrees.

## Summary

- A rule carries the degree of its premise to its conclusion, limited by its strength.
- Minimum and product are both computed unless you choose; a result that depends on the choice is reported as "possible".
- A conditional sentence is a rule whose premise must match an earlier fact.
