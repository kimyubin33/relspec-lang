# Chapter 8. Writing short, and the prose check

> In this chapter
> - Shorter forms: lists, omitted types, omitted names
> - Your own units, and large numbers
> - Notes and attached lines of free prose
> - The five warnings the compiler gives about prose

*Corresponds to chapters 16 and 17 of the Korean edition.*

## 8.1 The same meaning in fewer lines

The forms in chapters 3 to 7 are explicit, and writing every one of them in full is tiring. Since 0.9 the common cases have short forms.

```aha
대상 신청자, 강의실, 담당자
행위 예약하다, 승인하다: 주체, 대상
단위 원, 명

정의 정원 = 40 명
정의 기본요금 = 3만 원
정의 최대요금 = 12만원

검사 기본요금 < 최대요금

미지 참석자: 명
조건 참석자 >= 1 명
조건 참석자 <= 정원
목표 참석자 <= 정원
```

| Korean | English |
| --- | --- |
| `대상 신청자, 강의실, 담당자` | objects `applicant`, `classroom`, `manager` |
| `행위 예약하다, 승인하다: 주체, 대상` | actions `reserve`, `approve`: agent, object |
| `단위 원, 명` | units `won`, `people` |
| `정의 정원 = 40 명` | define `capacity` = 40 people |
| `정의 기본요금 = 3만 원` | define `base-fee` = 30,000 won |
| `검사 기본요금 < 최대요금` | check: base-fee < max-fee |
| `미지 참석자: 명` | unknown `attendees`, in people |

| Short form | What was shortened |
| --- | --- |
| `대상 a, b, c` | several declarations on one line |
| `정의 name = value` | the type, when the value shows it |
| `검사 expression` | the name of a check, constraint or goal |
| `단위 원, 명` | custom units: values in different units cannot be mixed |
| `3만`, `12만원` | Korean number words; the unit may be attached |
| `미지 x: unit` | the `= ?` |

A custom unit is a type of its own. Comparing won with people is a type error just as comparing seconds with metres is.

## 8.2 Notes and attached lines

Not everything can be formalised. A sentence can carry free prose in two ways.

```aha
대상 신청자, 강의실, 담당자
행위 예약하다, 승인하다: 주체, 대상
단위 명
정의 정원 = 40 명

신청자가 강의실을 예약한다: 수업 시작 이틀 전까지 신청서를 낸다

담당자가 강의실을 승인한다
- 같은 시간에 다른 예약이 없는지 확인한다
- 승인 결과를 신청자에게 알린다

본문: 예약 원칙
- 강의실은 수업 목적으로만 쓴다
```

| Korean | English |
| --- | --- |
| `신청자가 강의실을 예약한다: …` | the applicant reserves the classroom: *submits the form two days before class* |
| `담당자가 강의실을 승인한다` | the manager approves the classroom |
| `- 같은 시간에 다른 예약이 없는지 확인한다` | – checks that there is no other reservation at that time |
| `- 승인 결과를 신청자에게 알린다` | – tells the applicant the result |
| `본문: 예약 원칙` | prose: reservation principles |

- Text after a second `:` on a sentence line is a **note** on that sentence.
- A line starting with `-` is an **attached line**. It belongs to the nearest sentence, reference or `본문` line above it. One instruction per line keeps lines short and readable.
- `본문:` (*prose*) is a line of free text that belongs to no sentence.

The sentence itself is checked by the grammar. The prose is not parsed as a sentence, but it is not ignored either.

## 8.3 The prose check

The compiler looks for declared names inside prose (the editor colours them), and reports five patterns that can be detected without understanding the text. They are **warnings**: the file still compiles.

```aha 경고=K601,K602,K604,K603
대상 신청자, 강의실, 담당자
행위 예약하다, 승인하다: 주체, 대상
단위 원
정의 기본요금 = 3만 원
정의 예약마감 = 2 일

담당자가 강의실을 승인한다: 3만 원이 입금되었는지 적절히 확인한다
본문: 예약 마감이 지나면 해당 신청은 받지 않는다
```

| Korean | English |
| --- | --- |
| `정의 기본요금 = 3만 원` | define `base-fee` = 30,000 won |
| `정의 예약마감 = 2 일` | define `reservation-deadline` = 2 days |
| `…: 3만 원이 입금되었는지 적절히 확인한다` | …: *appropriately* checks whether *30,000 won* was paid |
| `본문: 예약 마감이 지나면 해당 신청은 받지 않는다` | prose: after the *reservation deadline* (written with a space), *the said* application is not accepted |

| Code | What it finds | In the example | Why it matters |
| --- | --- | --- | --- |
| `K601` | A quantity in prose equal to a defined value | `3만 원` equals `기본요금` | Change the definition and the prose becomes wrong without anyone noticing |
| `K602` | A word with no stated criterion | `적절히` (appropriately) | The reader has to guess the threshold |
| `K603` | A pointing word that cannot be resolved | `해당` (the said) | It is not clear what is pointed at |
| `K604` | A declared name written with a space | `예약 마감` for `예약마감` | The mention is not linked to the name |
| `K605` | A prose line longer than 80 characters | (none here) | Several instructions run together |

A warning has the form `file(line,column): warning CODE: message` and the exit code stays 0.

**What the absence of warnings means.** Only that none of these five patterns occurred. It does not mean the prose can be read in only one way. Deciding that requires understanding the text, which is the job planned for the AI check (chapter 11). The computed check deliberately reports only what it can be sure of; a checker that guesses produces warnings people learn to ignore.

Requirements engineering has long catalogued such patterns under the name *requirements smells*: vague adverbs, unresolved pronouns, duplicated constants.

## Summary

- Short forms cut the ceremony without changing the meaning.
- Notes, attached lines and `본문` hold free prose.
- The prose check links prose to declared names and reports five reliable patterns as warnings.
