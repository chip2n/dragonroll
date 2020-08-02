use std::iter::Peekable;

pub struct PeekWhile<'a, I, F>
where
    I: Iterator + 'a,
{
    iter: &'a mut Peekable<I>,
    f: F,
}

impl<'a, I, F> Iterator for PeekWhile<'a, I, F>
where
    I: Iterator + 'a,
    F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let &mut PeekWhile {
            ref mut iter,
            ref mut f,
        } = self;
        if iter.peek().map(f).unwrap_or(false) {
            iter.next()
        } else {
            None
        }
    }
}

pub fn peek_while<'a, I, F>(iter: &'a mut Peekable<I>, f: F) -> PeekWhile<'a, I, F>
where
    I: Iterator + 'a,
    F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
{
    PeekWhile { iter, f }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn keep_first_non_matching() {
        let mut iter = vec![1, 2, 3].into_iter().peekable();
        let result: Vec<u32> = peek_while(&mut iter, |n| *n < 3).collect();
        assert_eq!(result, vec![1, 2]);
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }
}
