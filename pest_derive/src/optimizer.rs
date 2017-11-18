// pest. The Elegant Parser
// Copyright (C) 2017  Dragoș Tiselice
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use ast::*;

pub fn optimize(rules: Vec<Rule>) -> Vec<Rule> {
    rules.into_iter().map(|rule| {
        let rotate_right = |expr| {
            // TODO: Use box syntax when it gets stabilized.
            match expr {
                Expr::Seq(lhs, rhs) => {
                    let lhs = *lhs;
                    match lhs {
                        Expr::Seq(ll, lr) => {
                            Expr::Seq(
                                ll,
                                Box::new(Expr::Seq(lr, rhs))
                            )
                        }
                        lhs => Expr::Seq(Box::new(lhs), rhs)
                    }
                }
                Expr::Choice(lhs, rhs) => {
                    let lhs = *lhs;
                    match lhs {
                        Expr::Choice(ll, lr) => {
                            Expr::Choice(
                                ll,
                                Box::new(Expr::Choice(lr, rhs))
                            )
                        }
                        lhs => Expr::Choice(Box::new(lhs), rhs)
                    }
                }
                expr => expr
            }
        };

        let unpack_repetitions = |expr| {
            match expr {
                Expr::RepOnce(expr) => {
                    Expr::Seq(
                        expr.clone(),
                        Box::new(Expr::Rep(expr))
                    )
                }
                Expr::RepExact(expr, num) => {
                    (1..num + 1).map(|_| {
                                    *expr.clone()
                                })
                                .rev()
                                .fold(None, |rep, expr| {
                                    match rep {
                                        None => Some(expr),
                                        Some(rep) => {
                                            Some(
                                                Expr::Seq(
                                                    Box::new(expr),
                                                    Box::new(rep)
                                                )
                                            )
                                        }
                                    }
                                })
                                .unwrap()
                }
                Expr::RepMin(expr, min) => {
                    (1..min + 2).map(|i| {
                                    if i <= min {
                                        *expr.clone()
                                    } else {
                                        Expr::Rep(expr.clone())
                                    }
                                })
                                .rev()
                                .fold(None, |rep, expr| {
                                    match rep {
                                        None => Some(expr),
                                        Some(rep) => {
                                            Some(
                                                Expr::Seq(
                                                    Box::new(expr),
                                                    Box::new(rep)
                                                )
                                            )
                                        }
                                    }
                                })
                                .unwrap()
                }
                Expr::RepMax(expr, max) => {
                    (1..max + 1).map(|_| {
                                    Expr::Opt(expr.clone())
                                })
                                .rev()
                                .fold(None, |rep, expr| {
                                    match rep {
                                        None => Some(expr),
                                        Some(rep) => {
                                            Some(
                                                Expr::Seq(
                                                    Box::new(expr),
                                                    Box::new(rep)
                                                )
                                            )
                                        }
                                    }
                                })
                                .unwrap()
                }
                Expr::RepMinMax(expr, min, max) => {
                    (1..max + 1).map(|i| {
                                    if i <= min {
                                        *expr.clone()
                                    } else {
                                        Expr::Opt(expr.clone())
                                    }
                                })
                                .rev()
                                .fold(None, |rep, expr| {
                                    match rep {
                                        None => Some(expr),
                                        Some(rep) => {
                                            Some(
                                                Expr::Seq(
                                                    Box::new(expr),
                                                    Box::new(rep)
                                                )
                                            )
                                        }
                                    }
                                })
                                .unwrap()
                }
                expr => expr
            }
        };

        match rule {
            Rule { name, ty, expr } => Rule {
                name,
                ty,
                expr: expr.map_bottom_up(rotate_right)
                          .map_bottom_up(unpack_repetitions)
                          .map_bottom_up(|expr| {
                              if ty == RuleType::Atomic {
                                  // TODO: Use box syntax when it gets stabilized.
                                  match expr {
                                      Expr::Seq(lhs, rhs) => {
                                          match (*lhs, *rhs) {
                                              (Expr::Str(lhs), Expr::Str(rhs)) => {
                                                  Expr::Str(lhs + &rhs)
                                              }
                                              (Expr::Insens(lhs), Expr::Insens(rhs)) => {
                                                  Expr::Insens(lhs + &rhs)
                                              }
                                              (lhs, rhs) => Expr::Seq(Box::new(lhs), Box::new(rhs))
                                          }
                                      }
                                      expr => expr
                                  }
                              } else {
                                  expr
                              }
                          })
                          .map_top_down(|expr| {
                              // TODO: Use box syntax when it gets stabilized.
                              match expr {
                                  Expr::Choice(lhs, rhs) => {
                                      match (*lhs, *rhs) {
                                          (Expr::Seq(l1, r1), Expr::Seq(l2, r2)) => {
                                              if l1 == l2 {
                                                  Expr::Seq(
                                                      l1,
                                                      Box::new(Expr::Choice(
                                                          r1,
                                                          r2
                                                      ))
                                                  )
                                              } else {
                                                  Expr::Choice(
                                                      Box::new(Expr::Seq(l1, r1)),
                                                      Box::new(Expr::Seq(l2, r2))
                                                  )
                                              }
                                          }
                                          (lhs, rhs) => Expr::Choice(Box::new(lhs), Box::new(rhs))
                                      }
                                  }
                                  expr => expr
                              }
                          })
            }
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use quote::Ident;

    #[test]
    fn concat_strings() {
        let rules = vec![
            Rule {
                name: Ident::new("rule"),
                ty:   RuleType::Atomic,
                expr: Expr::Seq(
                    Box::new(Expr::Seq(
                        Box::new(Expr::Str("a".to_owned())),
                        Box::new(Expr::Str("b".to_owned()))
                    )),
                    Box::new(Expr::Seq(
                        Box::new(Expr::Str("c".to_owned())),
                        Box::new(Expr::Str("d".to_owned()))
                    ))
                )
            }
        ];
        let concatenated = vec![
            Rule {
                name: Ident::new("rule"),
                ty:   RuleType::Atomic,
                expr: Expr::Str("abcd".to_owned())
            }
        ];

        assert_eq!(optimize(rules), concatenated);
    }

    #[test]
    fn unroll_loop_exact() {
        let rules = vec![
            Rule {
                name: Ident::new("rule"),
                ty: RuleType::Atomic,
                expr: Expr::RepExact(
                    Box::new(Expr::Ident(Ident::new("a"))),
                    3
                )
            }
        ];
        let unrolled = vec![
            Rule {
                name: Ident::new("rule"),
                ty: RuleType::Atomic,
                expr: Expr::Seq(
                    Box::new(Expr::Ident(Ident::new("a"))),
                    Box::new(Expr::Seq(
                        Box::new(Expr::Ident(Ident::new("a"))),
                        Box::new(Expr::Ident(Ident::new("a")))
                    ))
                )
            }
        ];

        assert_eq!(optimize(rules), unrolled);
    }


    #[test]
    fn unroll_loop_max() {
        let rules = vec![
            Rule {
                name: Ident::new("rule"),
                ty: RuleType::Atomic,
                expr: Expr::RepMax(
                    Box::new(Expr::Str("a".to_owned())),
                    3
                )
            }
        ];
        let unrolled = vec![
            Rule {
                name: Ident::new("rule"),
                ty: RuleType::Atomic,
                expr: Expr::Seq(
                    Box::new(Expr::Opt(
                        Box::new(Expr::Str("a".to_owned()))
                    )),
                    Box::new(Expr::Seq(
                        Box::new(Expr::Opt(
                            Box::new(Expr::Str("a".to_owned()))
                        )),
                        Box::new(Expr::Opt(
                            Box::new(Expr::Str("a".to_owned()))
                        ))
                    ))
                )
            }
        ];

        assert_eq!(optimize(rules), unrolled);
    }

    #[test]
    fn unroll_loop_min() {
        let rules = vec![
            Rule {
                name: Ident::new("rule"),
                ty: RuleType::Atomic,
                expr: Expr::RepMin(
                    Box::new(Expr::Str("a".to_owned())),
                    2
                )
            }
        ];
        let unrolled = vec![
            Rule {
                name: Ident::new("rule"),
                ty: RuleType::Atomic,
                expr: Expr::Seq(
                    Box::new(Expr::Str("a".to_owned())),
                    Box::new(Expr::Seq(
                        Box::new(Expr::Str("a".to_owned())),
                        Box::new(Expr::Rep(
                            Box::new(Expr::Str("a".to_owned()))
                        ))
                    ))
                )
            }
        ];

        assert_eq!(optimize(rules), unrolled);
    }

    #[test]
    fn unroll_loop_min_max() {
        let rules = vec![
            Rule {
                name: Ident::new("rule"),
                ty: RuleType::Atomic,
                expr: Expr::RepMinMax(
                    Box::new(Expr::Str("a".to_owned())),
                    2,
                    3
                )
            }
        ];
        let unrolled = vec![
            Rule {
                name: Ident::new("rule"),
                ty: RuleType::Atomic,
                expr: Expr::Seq(
                    /* TODO possible room for improvement here:
                     * not sure what rationale behind reversing
                     * the unroll on min/max was, but it seems
                     * to eliminate the possiblity of concatenating
                     * repeated strings at the beginning of
                     * repetitions
                    Box::new(Expr::Str("aa".to_owned())),
                    Box::new(Expr::Opt(
                        Box::new(Expr::Str("a".to_owned()))
                    ))
                    */
                    Box::new(Expr::Str("a".to_owned())),
                    Box::new(Expr::Seq(
                        Box::new(Expr::Str("a".to_owned())),
                        Box::new(Expr::Opt(
                            Box::new(Expr::Str("a".to_owned())),
                        ))
                    ))
                )
            }
        ];

        assert_eq!(optimize(rules), unrolled);
    }


    #[test]
    fn concat_insensitive_strings() {
        let rules = vec![
            Rule {
                name: Ident::new("rule"),
                ty:   RuleType::Atomic,
                expr: Expr::Seq(
                    Box::new(Expr::Seq(
                        Box::new(Expr::Insens("a".to_owned())),
                        Box::new(Expr::Insens("b".to_owned()))
                    )),
                    Box::new(Expr::Seq(
                        Box::new(Expr::Insens("c".to_owned())),
                        Box::new(Expr::Insens("d".to_owned()))
                    ))
                )
            }
        ];
        let concatenated = vec![
            Rule {
                name: Ident::new("rule"),
                ty:   RuleType::Atomic,
                expr: Expr::Insens("abcd".to_owned())
            }
        ];

        assert_eq!(optimize(rules), concatenated);
    }

    #[test]
    fn long_common_sequence() {
        let rules = vec![
            Rule {
                name: Ident::new("rule"),
                ty:   RuleType::Silent,
                expr: Expr::Choice(
                    Box::new(Expr::Seq(
                        Box::new(Expr::Ident(Ident::new("a"))),
                        Box::new(Expr::Seq(
                            Box::new(Expr::Ident(Ident::new("b"))),
                            Box::new(Expr::Ident(Ident::new("c")))
                        ))
                    )),
                    Box::new(Expr::Seq(
                        Box::new(Expr::Seq(
                            Box::new(Expr::Ident(Ident::new("a"))),
                            Box::new(Expr::Ident(Ident::new("b")))
                        )),
                        Box::new(Expr::Ident(Ident::new("d")))
                    ))
                )
            }
        ];
        let optimized = vec![
            Rule {
                name: Ident::new("rule"),
                ty:   RuleType::Silent,
                expr: Expr::Seq(
                    Box::new(Expr::Ident(Ident::new("a"))),
                    Box::new(Expr::Seq(
                        Box::new(Expr::Ident(Ident::new("b"))),
                        Box::new(Expr::Choice(
                            Box::new(Expr::Ident(Ident::new("c"))),
                            Box::new(Expr::Ident(Ident::new("d")))
                        ))
                    ))
                )
            }
        ];

        assert_eq!(optimize(rules), optimized);
    }
}
