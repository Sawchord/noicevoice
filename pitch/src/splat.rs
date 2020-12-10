#[derive(Debug, Clone)]
pub struct SplatAccessor<'a, B> {
   factor: usize,
   index: usize,
   inner: &'a [B],
}

impl<'a, B> SplatAccessor<'a, B> {
   pub fn new<A: 'a + AsRef<[B]>>(inner: &'a A) -> Self {
      Self {
         factor: 1,
         index: 0,
         inner: inner.as_ref(),
      }
   }

   pub fn splat(&self) -> (Self, Self) {
      (
         Self {
            factor: self.factor * 2,
            index: self.index,
            inner: self.inner,
         },
         Self {
            factor: self.factor * 2,
            index: self.index + self.factor,
            inner: self.inner,
         },
      )
   }

   pub fn len(&self) -> usize {
      self.inner.len() / self.factor
   }
}

impl<B> core::ops::Index<usize> for SplatAccessor<'_, B> {
   type Output = B;
   fn index(&self, index: usize) -> &B {
      &self.inner.as_ref()[index * self.factor + self.index]
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn splat() {
      let vec = vec![0, 1, 2, 3, 4, 5, 6, 7];
      let splat = SplatAccessor::new(&vec);

      assert_eq!(splat[1], 1);
      assert_eq!(splat[6], 6);

      let (left, right) = splat.splat();
      assert_eq!(left[0], 0);
      assert_eq!(right[0], 1);
      assert_eq!(left[2], 4);
      assert_eq![right[3], 7];

      let (left, right) = right.splat();
      assert_eq!(left[0], 1);
      assert_eq!(left[1], 5);
      assert_eq!(right[0], 3);
      assert_eq!(right[1], 7);
   }
}
