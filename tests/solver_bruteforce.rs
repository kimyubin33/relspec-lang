//! 제약 판정을 작은 정수 범위의 전수 조사와 비교합니다.
//! 미지값 세 개를 -3..=3으로 묶고 임의의 차이 제약과 목표를 만들어,
//! 컴파일러의 `보장`·`가능`·`불가능`과 범위가 전수 조사 결과와 같은지 확인합니다.

use aha::compile;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
}

const NAMES: [&str; 3] = ["가", "나", "다"];
const OPS: [&str; 5] = ["<=", "<", ">=", ">", "=="];

/// (왼쪽 미지값, 오른쪽 미지값 또는 None, 연산자, 상수)
type Atom = (usize, Option<usize>, &'static str, i64);

fn random_atom(rng: &mut Lcg) -> Atom {
    let left = rng.next(3) as usize;
    let right = if rng.next(2) == 0 {
        None
    } else {
        Some((left + 1 + rng.next(2) as usize) % 3)
    };
    (
        left,
        right,
        OPS[rng.next(5) as usize],
        rng.next(9) as i64 - 4,
    )
}

fn source(atom: &Atom) -> String {
    let (l, r, op, c) = *atom;
    let lhs = match r {
        Some(r) => format!("{} - {}", NAMES[l], NAMES[r]),
        None => NAMES[l].to_string(),
    };
    let rhs = if c < 0 {
        format!("-{}", -c)
    } else {
        c.to_string()
    };
    format!("{lhs} {op} {rhs}")
}

fn holds(atom: &Atom, v: &[i64; 3]) -> bool {
    let (l, r, op, c) = *atom;
    let x = v[l] - r.map_or(0, |r| v[r]);
    match op {
        "<=" => x <= c,
        "<" => x < c,
        ">=" => x >= c,
        ">" => x > c,
        _ => x == c,
    }
}

#[test]
fn judgements_match_exhaustive_search() {
    let mut rng = Lcg(20261110);
    let mut checked = [0usize; 4];
    for _ in 0..600 {
        let conditions: Vec<Atom> = (0..rng.next(4)).map(|_| random_atom(&mut rng)).collect();
        let goals: Vec<Vec<Atom>> = (0..3)
            .map(|_| {
                (0..1 + rng.next(2))
                    .map(|_| random_atom(&mut rng))
                    .collect()
            })
            .collect();

        let mut text = String::new();
        for name in NAMES {
            text += &format!(
                "미지 {name}: 정수 = ?\n조건 {name}범위 = {name} >= -3 그리고 {name} <= 3\n"
            );
        }
        for (i, atom) in conditions.iter().enumerate() {
            text += &format!("조건 조건{i} = {}\n", source(atom));
        }
        for (i, atoms) in goals.iter().enumerate() {
            let body: Vec<_> = atoms.iter().map(source).collect();
            text += &format!("목표 목표{i} = {}\n", body.join(" 그리고 "));
        }

        let mut solutions = Vec::new();
        for a in -3..=3 {
            for b in -3..=3 {
                for c in -3..=3 {
                    let v = [a, b, c];
                    if conditions.iter().all(|atom| holds(atom, &v)) {
                        solutions.push(v);
                    }
                }
            }
        }

        let result = compile(&text);
        if solutions.is_empty() {
            let diagnostic = result
                .err()
                .unwrap_or_else(|| panic!("모순을 놓침:\n{text}"));
            assert_eq!(diagnostic.code, "K404", "{text}");
            checked[3] += 1;
            continue;
        }
        let program = result.unwrap_or_else(|error| panic!("{error}\n{text}"));

        for (i, atoms) in goals.iter().enumerate() {
            let count = solutions
                .iter()
                .filter(|v| atoms.iter().all(|atom| holds(atom, v)))
                .count();
            let expected = match count {
                0 => "불가능",
                n if n == solutions.len() => "보장",
                _ => "가능",
            };
            assert_eq!(program.goals[i].status, expected, "목표{i}\n{text}");
            checked[["보장", "가능", "불가능"]
                .iter()
                .position(|s| *s == expected)
                .unwrap()] += 1;
        }
        for (k, unknown) in program.unknowns.iter().enumerate() {
            let min = solutions.iter().map(|v| v[k]).min().unwrap();
            let max = solutions.iter().map(|v| v[k]).max().unwrap();
            assert_eq!(
                unknown.min.as_deref(),
                Some(min.to_string().as_str()),
                "{text}"
            );
            assert_eq!(
                unknown.max.as_deref(),
                Some(max.to_string().as_str()),
                "{text}"
            );
        }
    }
    // 세 판정과 모순이 모두 충분히 나왔는지 확인해 시험이 한쪽으로 치우치지 않게 합니다.
    assert!(checked.iter().all(|&n| n >= 20), "{checked:?}");
}
