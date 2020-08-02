use crate::utils;
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum Token {
    Num(u32),
    Sym(char),
    OpenParen,
    CloseParen,
    Roll { number: u32, sides: u32 },
}

pub fn tokenize(s: &str) -> Option<Vec<Token>> {
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

fn consume_num(iter: &mut Peekable<impl Iterator<Item = char>>) -> Option<u32> {
    utils::peek_while(iter, |c| c.is_digit(10))
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

    mod tokenize {
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
