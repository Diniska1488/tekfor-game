use mlua::Result as LuaResult;

use macroquad::rand::gen_range;

pub struct Basic {
  pub variants: Vec<String>,
}

impl Basic {
  pub fn new(n_pairs: usize) -> Self {
    // TODO: Кешировать все правильные варианты, а не генерировать их заново.
    let mut variants = generate_valid_paren_sequence(n_pairs);

    // Случайным образом отфильтровываем данные 50/50, чтоб не сломать вообще все варианты.
    for variant in variants.iter_mut().filter(|_| gen_range::<f32>(0.0, 1.0) < 0.5) {
      break_valid_paren_sequence(variant);
    }
    Self { variants }
  }

  pub fn is_valid_with(&self, cb: impl Fn(&str) -> LuaResult<bool>) -> bool {
    self.variants.iter().all(|seq| cb(seq).is_ok_and(|b| b == is_valid(seq)))
  }
}

impl Default for Basic {
  fn default() -> Self {
    const N_PAIRS: usize = 10;

    Self::new(N_PAIRS)
  }
}

fn is_valid(seq: &str) -> bool {
  if seq.is_empty() || !seq.len().is_multiple_of(2) {
    return false;
  }

  let mut depth = 0isize;

  for byte in seq.as_bytes() {
    match byte {
      b'(' => depth += 1,
      b')' => depth -= 1,
      _ => unreachable!(),
    }

    if depth < 0 {
      return false;
    }
  }
  depth == 0
}

// Самая наивная реализация - быстрее, чем если бы тут были:
// * побитовые операции;
// * эмуляция стека внутри цикла.
fn generate_valid_paren_sequence(n: usize) -> Vec<String> {
  fn backtrack(
    n: usize,
    left_count: usize,
    right_count: usize,
    string: &mut String,
    result: &mut Vec<String>,
  ) {
    if left_count == n && right_count == n {
      result.push(string.clone());
      return;
    }

    if left_count < n {
      string.push('(');
      backtrack(n, left_count + 1, right_count, string, result);
      string.pop();
    }

    if left_count > right_count {
      string.push(')');
      backtrack(n, left_count, right_count + 1, string, result);
      string.pop();
    }
  }

  let mut result = Vec::with_capacity(nth_catalan_number(n));
  {
    backtrack(n, 0, 0, &mut String::with_capacity(n * 2), &mut result);
  }
  result
}

/// "Ломает" входную последовательность посредством перестановки двух случайных символов.
fn break_valid_paren_sequence(seq: &mut String) {
  let bytes = unsafe { seq.as_mut_vec() };

  let from_idx = gen_range(0, bytes.len());
  let to_idx = loop {
    let n = gen_range(0, bytes.len());
    if bytes[n] != bytes[from_idx] {
      break n;
    }
  };

  bytes.swap(from_idx, to_idx);
}

// https://en.wikipedia.org/wiki/Catalan_number
fn nth_catalan_number(n: usize) -> usize {
  let mut catalan = Vec::with_capacity(n + 1);
  catalan.push(1);

  for i in 1..=n {
    catalan.push(catalan[i - 1] * 2 * (2 * i - 1) / (i + 1));
  }
  catalan[n]
}

#[test]
fn nth_catalan_number_test() {
  assert_eq!(nth_catalan_number(4), 14);
}

#[test]
fn is_valid_test() {
  assert!(is_valid("(())"));
  assert!(!is_valid("(("));
  assert!(is_valid("(())()()"));
  assert!(!is_valid("()))()()((()()()(())"));
}
