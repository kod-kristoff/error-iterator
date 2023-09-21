pub mod io;
pub mod utf8;

pub enum Step<T, E> {
    Done,
    Yield(T),
    Skip,
    Error(E),
}

pub trait EIterator {
    type Item;
    type Error;

    fn enext(&mut self) -> Step<Self::Item, Self::Error>;

    fn step<F, B, E>(&mut self, mut f: F) -> Step<B, E>
    where
        F: FnMut(Self::Item) -> Step<B, E>,
        E: From<Self::Error>,
    {
        match self.enext() {
            Step::Done => Step::Done,
            Step::Error(e) => Step::Error(From::from(e)),
            Step::Skip => Step::Skip,
            Step::Yield(x) => f(x),
        }
    }

    fn step_option<F, B, E>(&mut self, mut f: F) -> Step<B, E>
    where
        F: FnMut(Option<Self::Item>) -> Step<B, E>,
        E: From<Self::Error>,
    {
        match self.enext() {
            Step::Done => f(None),
            Step::Error(e) => Step::Error(From::from(e)),
            Step::Skip => Step::Skip,
            Step::Yield(x) => f(Some(x)),
        }
    }

    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    {
        Map {
            iter: self,
            func: f,
        }
    }

    fn map_error<E2, F>(self, f: F) -> MapError<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E2,
    {
        MapError {
            iter: self,
            func: f,
        }
    }

    fn map_error_from<E2>(self) -> MapError<Self, fn(Self::Error) -> E2>
    where
        Self: Sized,
        E2: From<Self::Error>,
    {
        self.map_error(From::from)
    }

    fn iter(self) -> ToResultIterator<Self>
    where
        Self: Sized,
    {
        ToResultIterator(self)
    }
}

pub trait ToEIter
where
    Self: Sized,
{
    fn eiter(self) -> ResultIterator<Self> {
        ResultIterator(self)
    }
}

impl<I, T, E> ToEIter for I where I: Iterator<Item = Result<T, E>> {}

pub struct Map<I, F> {
    iter: I,
    func: F,
}

impl<B, I: EIterator, F> EIterator for Map<I, F>
where
    F: FnMut(I::Item) -> B,
{
    type Item = B;
    type Error = I::Error;

    fn enext(&mut self) -> Step<Self::Item, Self::Error> {
        let f = &mut self.func;
        self.iter.step(|x| Step::Yield(f(x)))
    }
}

pub struct MapError<I, F> {
    iter: I,
    func: F,
}

impl<E, I: EIterator, F> EIterator for MapError<I, F>
where
    F: FnMut(I::Error) -> E,
{
    type Item = I::Item;
    type Error = E;

    fn enext(&mut self) -> Step<Self::Item, Self::Error> {
        match self.iter.enext() {
            Step::Done => Step::Done,
            Step::Skip => Step::Skip,
            Step::Error(e) => Step::Error((self.func)(e)),
            Step::Yield(x) => Step::Yield(x),
        }
    }
}

pub struct ToResultIterator<I>(I);

impl<I> Iterator for ToResultIterator<I>
where
    I: EIterator,
{
    type Item = Result<I::Item, I::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.enext() {
                Step::Done => {
                    return None;
                }
                Step::Skip => (),
                Step::Error(e) => {
                    return Some(Err(e));
                }
                Step::Yield(x) => {
                    return Some(Ok(x));
                }
            }
        }
    }
}

pub struct ResultIterator<I>(I);
impl<I, T, E> EIterator for ResultIterator<I>
where
    I: Iterator<Item = Result<T, E>>,
{
    type Item = T;
    type Error = E;

    fn enext(&mut self) -> Step<Self::Item, Self::Error> {
        match self.0.next() {
            Some(Ok(x)) => Step::Yield(x),
            Some(Err(e)) => Step::Error(e),
            None => Step::Done,
        }
    }
}
