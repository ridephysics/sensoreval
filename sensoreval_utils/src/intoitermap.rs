pub struct Map<I, F> {
    into_iter: I,
    f: F,
}

pub trait IntoIteratorMap {
    fn map_intoiter<'a, F, B>(&'a self, f: F) -> Map<&'a Self, F>
    where
        F: Copy + FnMut(<&'a Self as std::iter::IntoIterator>::Item) -> B,
        for<'b> &'b Self: std::iter::IntoIterator;
}

impl<I> IntoIteratorMap for I
where
    for<'a> &'a I: std::iter::IntoIterator,
    I: ?Sized,
{
    fn map_intoiter<'a, F, B>(&'a self, f: F) -> Map<&'a Self, F>
    where
        F: Copy + FnMut(<&'a Self as std::iter::IntoIterator>::Item) -> B,
        for<'b> &'b Self: std::iter::IntoIterator,
    {
        Map { into_iter: self, f }
    }
}

impl<'a, I, F, B> serde::Serialize for Map<&'a I, F>
where
    F: Copy + FnMut(<&'a I as std::iter::IntoIterator>::Item) -> B,
    B: serde::Serialize,
    for<'b> &'b I: std::iter::IntoIterator,
    I: ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.into_iter.into_iter().map(self.f))
    }
}
