# The AHAA Language — English Edition

**AHAA — AI-bound Human language, Assessed by AI** (pronounced "aha", 아하)

**A language for writing down what you tell an AI without silently leaving things out.**

Ordinary speech omits most of what a word means and lets the listener fill the gaps from context. When the reader is an AI, the gaps are filled differently every time, and the writer never sees which gaps were left. AHAA lets you write a word as a **bundle of dimensions**, connect words with sentences, and have the text **checked**: how much of it is filled in, and whether its parts agree.

- **Version:** AHAA 0.14 (`languageVersion: "0.14"`).
- **Every example is checked.** Each `aha` code block in this book is compiled by `tests/book.rs`. If an example that should compile fails, or an example that should fail gives a different error, `cargo test` fails. The examples therefore always match the current compiler.
- **This is a condensed edition.** The [Korean textbook](../book/README.md) has 20 chapters and four appendices with full theory, proofs and exercises. This edition covers the same language in 11 chapters. Each chapter says which Korean chapters it corresponds to. Where the two differ, the Korean edition and the compiler are authoritative.
- **The keywords are Korean.** AHAA is written with Korean keywords and Korean sentences. Every example here comes with an English gloss, and [Appendix A](appendix-a-keywords.md) lists all keywords. You do not need to read Korean to follow the book, but you do need to type it to write AHAA.

## What AHAA does: write, connect, get checked

| Activity | Meaning | Chapters |
| --- | --- | --- |
| **Write** | Say what there is, with words and dimensions, values and units | 3, 8, **10** |
| **Connect** | Say who does what, when and where, how much, and what causes what | 4, 6, 7 |
| **Get checked** | Have the text verified: is it well-formed, how filled is it, is it connected | 5, 8, 9, 10, 11 |
| Tools | Installing, the editor, the output for other programs | 2 |

There are two kinds of check. The **computed check** works today and is what most of this book describes. The **AI check** is not built; chapter 11 describes it as a plan.

## Contents

| Ch. | Title | Covers | Korean ch. |
| --- | --- | --- | --- |
| 1 | [What AHAA is](01-what-is-ahaa.md) | Words omit most things, blanks, three numbers, what exists now | 1 |
| 2 | [Getting started](02-getting-started.md) | Install, first file, `check` and `compile`, reading diagnostics, the editor | 2, 12, 15 |
| 3 | [Values, units and checks](03-values-units-checks.md) | Definitions, types, units, expressions, checks | 3, 4 |
| 4 | [Time, position and events](04-time-position-events.md) | Reference frames, points and quantities, objects, events and roles | 5, 6 |
| 5 | [Unknowns and goals](05-unknowns-and-goals.md) | Unknown values, constraints, goals, the three verdicts | 7 |
| 6 | [Sentences](06-sentences.md) | Actions, particles and roles, references, degree, tense | 8, 9, 10, 11 |
| 7 | [Rules and conditionals](07-rules-and-conditionals.md) | Propagating degree, minimum and product, conditional sentences | 13, 14 |
| 8 | [Writing short, and the prose check](08-writing-short-and-prose-check.md) | List declarations, custom units, notes, attached lines, warnings | 16, 17 |
| 9 | [The graph](09-graph.md) | Nodes, edges, components, the viewer | 18 |
| 10 | [Words and scenes](10-words-and-scenes.md) | Word definitions, scenes, blanks, weights, the fill score | 19 |
| 11 | [What is measured, and what comes next](11-measurement-and-next.md) | Operational definitions, the two claims, the six steps, limits | 20 |
| A | [Keywords and diagnostics](appendix-a-keywords.md) | Every keyword with a gloss, error code ranges | Appendices A, B |

**Short path.** If you only want the central idea, read chapters 1, 2, 10 and 11.

## How to read the examples

A block marked `aha` is AHAA source. It is followed by a gloss table, and sometimes by the compiler's output in a `text` block. The compiler's messages are in Korean; the book translates them.

```aha
정의 한계: 시간 = 300 밀리초
검사 빠름 = 250 밀리초 <= 한계
```

| Korean | English |
| --- | --- |
| `정의 한계: 시간 = 300 밀리초` | define `limit`: duration = 300 milliseconds |
| `검사 빠름 = 250 밀리초 <= 한계` | check `fast` = 250 milliseconds <= limit |

Examples that are expected to fail name the error code in the block's first line, as in `aha 오류=K401` (`오류` means *error*). The test suite verifies that exactly that error occurs.
