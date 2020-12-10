use core::marker::PhantomData;

#[derive(Debug, Clone)]
struct SplatAccessor<A: AsRef<[B]>, B> {
    factor: usize,
    index: usize,
    inner: A,
    _data: PhantomData<B>,
}

impl<A: AsRef<[B]>, B> SplatAccessor<A, B> {
    fn new(inner: A) -> Self {
        Self {
            factor: 1,
            index: 0,
            inner,
            _data: PhantomData,
        }
    }

    fn splat(self) -> (Self, Self) {
        //(Self { ..self.clone() }, Self { ..self })
        todo!()
    }
}

impl<A, B> core::ops::Index<usize> for SplatAccessor<A, B>
where
    A: AsRef<[B]>,
{
    type Output = B;
    fn index(&self, index: usize) -> &B {
        &self.inner.as_ref()[index * self.factor + self.index]
    }
}

impl<A, B> core::ops::IndexMut<usize> for SplatAccessor<A, B>
where
    A: AsMut<[B]> + AsRef<[B]>,
{
    fn index_mut(&mut self, index: usize) -> &mut B {
        &mut self.inner.as_mut()[index * self.factor + self.index]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
