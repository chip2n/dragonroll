pub mod ui;

use crate::utils::*;
use rand::Rng;
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
enum Token {
    Num(u32),
    Sym(char),
    OpenParen,
    CloseParen,
    Roll { number: u32, sides: u32 },
}

#[derive(Debug)]
struct EvalError {
    msg: String,
}

impl EvalError {
    fn new(msg: impl Into<String>) -> Self {
        EvalError { msg: msg.into() }
    }
}

pub fn eval(s: &str) -> Option<u32> {
    let mut rng = rand::thread_rng();
    eval_with_roller(s, &mut rng)
}

fn eval_with_roller(s: &str, roller: &mut impl DiceRoller) -> Option<u32> {
    let mut tokens = tokenize(s)?.into_iter();
    let tokens = shunting_yard(&mut tokens);
    eval_tokens(tokens, roller).ok()
}

trait DiceRoller {
    fn roll(&mut self, sides: u32) -> u32;
}

impl<R: Rng> DiceRoller for R {
    fn roll(&mut self, sides: u32) -> u32 {
        self.gen_range(0, sides) + 1
    }
}

fn roll_dice(number: u32, sides: u32, roller: &mut impl DiceRoller) -> Vec<u32> {
    (0..number).map(|_| roller.roll(sides)).collect()
}

fn consume_num(iter: &mut Peekable<impl Iterator<Item = char>>) -> Option<u32> {
    peek_while(iter, |c| c.is_digit(10))
        .collect::<String>()
        .parse::<u32>()
        .ok()
}

fn consume_num_or_roll(iter: &mut Peekable<impl Iterator<Item = char>>) -> Option<Token> {
    let n1 = consume_num(iter)?;

    if let Some('d') = iter.peek() {
        iter.next();
        let n2 = consume_num(iter)?;
        return Some(Token::Roll {
            number: n1,
            sides: n2,
        });
    }

    return Some(Token::Num(n1));
}

fn tokenize(s: &str) -> Option<Vec<Token>> {
    let mut iterator = s.chars().peekable();
    let mut result = vec![];

    loop {
        let c = iterator.peek().cloned();
        match c {
            Some(' ') => {
                iterator.next();
                continue;
            }
            Some('(') => {
                result.push(Token::OpenParen);
                iterator.next();
            }
            Some(')') => {
                result.push(Token::CloseParen);
                iterator.next();
            }
            Some('+') => {
                result.push(Token::Sym('+'));
                iterator.next();
            }
            Some('-') => {
                result.push(Token::Sym('-'));
                iterator.next();
            }
            Some('*') => {
                result.push(Token::Sym('*'));
                iterator.next();
            }
            Some('/') => {
                result.push(Token::Sym('/'));
                iterator.next();
            }
            Some(_) => {
                if let Some(token) = consume_num_or_roll(&mut iterator) {
                    result.push(token);
                } else {
                    return None;
                }
            }
            None => {
                break;
            }
        }
    }

    Some(result)
}

fn op_precedence(op: &Token) -> u8 {
    match op {
        Token::Sym('/') => 3,
        Token::Sym('*') => 3,
        Token::Sym('+') => 2,
        Token::Sym('-') => 2,
        _ => panic!("unknown op {:?}", op),
    }
}

fn shunting_yard(tokens: &mut impl Iterator<Item = Token>) -> Vec<Token> {
    let mut output_queue: Vec<Token> = vec![];
    let mut op_stack: Vec<Token> = vec![];

    while let Some(token) = tokens.next() {
        match token {
            Token::Num(..) => output_queue.push(token),
            Token::Roll { .. } => output_queue.push(token),
            Token::Sym(..) => {
                while op_stack.len() > 0 {
                    let top_op = &op_stack[0];
                    if *top_op == Token::OpenParen {
                        break;
                    }

                    let top_precedence = op_precedence(top_op);
                    let current_precedence = op_precedence(&token);

                    // all current operators are left associative
                    if top_precedence < current_precedence {
                        break;
                    }

                    // pop operators from the operator stack onto the output queue.
                    let top_op = op_stack.remove(0);
                    output_queue.push(top_op);
                }

                op_stack.insert(0, token);
            }
            Token::OpenParen => op_stack.insert(0, Token::OpenParen),
            Token::CloseParen => {
                while op_stack[0] != Token::OpenParen {
                    let top_op = op_stack.remove(0);
                    output_queue.push(top_op);
                }

                // if the stack runs out without finding a left parenthesis, then there are mismatched parentheses
                if op_stack.len() == 0 {
                    // TODO
                    panic!("mismatched parentheses");
                }

                // remove the leftover paren
                let top_op = op_stack.remove(0);
                assert!(top_op == Token::OpenParen);
            }
        }
    }

    // if op stack not empty, pop everything to output queue
    while op_stack.len() > 0 {
        let op = op_stack.remove(0);
        output_queue.push(op);
    }

    output_queue
}

// TODO use something more concrete than tokens directly?
fn eval_tokens(tokens: Vec<Token>, roller: &mut impl DiceRoller) -> Result<u32, EvalError> {
    fn apply(stack: &mut Vec<u32>, f: impl Fn(u32, u32) -> u32) {
        let b = stack.remove(stack.len() - 1);
        let a = stack.remove(stack.len() - 1);
        let result = f(a, b);
        stack.push(result);
    }

    let mut stack: Vec<u32> = vec![];

    for token in tokens.into_iter() {
        match token {
            Token::Num(n) => stack.push(n),
            Token::Roll { number, sides } => {
                let result = roll_dice(number, sides, roller).iter().sum();
                stack.push(result);
            }
            Token::Sym('+') => apply(&mut stack, |a, b| a + b),
            Token::Sym('-') => apply(&mut stack, |a, b| a - b),
            Token::Sym('*') => apply(&mut stack, |a, b| a * b),
            Token::Sym('/') => apply(&mut stack, |a, b| a / b),
            _ => return Err(EvalError::new(format!("Unknown token {:?}", token))),
        }
    }

    if stack.len() != 1 {
        Err(EvalError::new("Stack not empty after evaluation finished."))
    } else {
        Ok(stack[0])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod consume_num_or_roll {
        use super::*;

        #[test]
        fn single_num() {
            let mut iter = "123".chars().peekable();
            let result = consume_num_or_roll(&mut iter);
            assert_eq!(result, Some(Token::Num(123)));
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn single_roll() {
            let mut iter = "2d6".chars().peekable();
            let result = consume_num_or_roll(&mut iter);
            assert_eq!(
                result,
                Some(Token::Roll {
                    number: 2,
                    sides: 6
                })
            );
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn single_roll_with_junk() {
            let mut iter = "2d6abc".chars().peekable();
            let result = consume_num_or_roll(&mut iter);
            assert_eq!(
                result,
                Some(Token::Roll {
                    number: 2,
                    sides: 6
                })
            );
            assert_eq!(iter.collect::<String>(), "abc");
        }

        #[test]
        fn junk() {
            let mut iter = "x22".chars().peekable();
            let result = consume_num_or_roll(&mut iter);
            assert_eq!(result, None);
            assert_eq!(iter.collect::<String>(), "x22");
        }
    }

    mod shunting_yard {
        use super::*;

        #[test]
        fn simple_add() {
            let mut tokens = tokenize("1 + 2").unwrap().into_iter();
            let expected = vec![Token::Num(1), Token::Num(2), Token::Sym('+')];
            assert_eq!(shunting_yard(&mut tokens), expected);
        }

        #[test]
        fn mul_precedence() {
            let mut tokens = tokenize("1 + 2 * 3").unwrap().into_iter();
            let expected = vec![
                Token::Num(1),
                Token::Num(2),
                Token::Num(3),
                Token::Sym('*'),
                Token::Sym('+'),
            ];
            assert_eq!(shunting_yard(&mut tokens), expected);
        }

        #[test]
        fn paren() {
            let mut tokens = tokenize("(1 + 2) * 3").unwrap().into_iter();
            let expected = vec![
                Token::Num(1),
                Token::Num(2),
                Token::Sym('+'),
                Token::Num(3),
                Token::Sym('*'),
            ];
            assert_eq!(shunting_yard(&mut tokens), expected);
        }
    }

    mod eval {
        use super::*;

        struct MaxDiceRoller;

        impl DiceRoller for MaxDiceRoller {
            fn roll(&mut self, sides: u32) -> u32 {
                sides
            }
        }

        #[test]
        fn eval_single_die() {
            assert_eq!(eval_with_roller("1d6", &mut MaxDiceRoller), Some(6));
        }

        #[test]
        fn eval_multiple_dice() {
            assert_eq!(eval_with_roller("2d6", &mut MaxDiceRoller), Some(12));
        }

        #[test]
        fn eval_multiple_different_dice() {
            assert_eq!(eval_with_roller("1d4 + 1d6", &mut MaxDiceRoller), Some(10));
        }

        #[test]
        fn addition() {
            assert_eq!(eval_with_roller("1d6 + 2", &mut MaxDiceRoller), Some(8));
        }

        #[test]
        fn subtraction() {
            assert_eq!(eval_with_roller("1d6 - 2", &mut MaxDiceRoller), Some(4));
        }
    }

    mod tokenizer {
        use super::*;

        #[test]
        fn roll() {
            let s = "2d6";
            let expected = vec![Token::Roll {
                number: 2,
                sides: 6,
            }];
            assert_eq!(tokenize(s), Some(expected));
        }

        #[test]
        fn addition() {
            let s = "1 + 2";
            let expected = vec![Token::Num(1), Token::Sym('+'), Token::Num(2)];
            assert_eq!(tokenize(s), Some(expected));
        }

        #[test]
        fn subtraction() {
            let s = "1 - 2";
            let expected = vec![Token::Num(1), Token::Sym('-'), Token::Num(2)];
            assert_eq!(tokenize(s), Some(expected));
        }

        #[test]
        fn nospace() {
            let s = "1+2";
            let expected = vec![Token::Num(1), Token::Sym('+'), Token::Num(2)];
            assert_eq!(tokenize(s), Some(expected));
        }

        #[test]
        fn roll_and_addition() {
            let s = "2d6 + 1";
            let expected = vec![
                Token::Roll {
                    number: 2,
                    sides: 6,
                },
                Token::Sym('+'),
                Token::Num(1),
            ];
            assert_eq!(tokenize(s), Some(expected));
        }

        #[test]
        fn parens() {
            let s = "(1 + 2)";
            let expected = vec![
                Token::OpenParen,
                Token::Num(1),
                Token::Sym('+'),
                Token::Num(2),
                Token::CloseParen,
            ];
            assert_eq!(tokenize(s), Some(expected));
        }
    }
}
