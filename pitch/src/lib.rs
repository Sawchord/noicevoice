use num_complex::Complex32;
use std::f32;
use std::f32::consts::PI;

pub fn real_fft<B: AsRef<[Complex32]>>(input: B) -> Result<Vec<Complex32>, ()> {
    fft_inner(input, false)
}

pub fn real_rfft<B: AsRef<[Complex32]>>(input: B) -> Result<Vec<Complex32>, ()> {
    fft_inner(input, true)
}

fn fft_inner<B: AsRef<[Complex32]>>(input: B, is_reverse: bool) -> Result<Vec<Complex32>, ()> {
    let in_ref = input.as_ref();
    if !in_ref.len().is_power_of_two() {
        return Err(());
    }

    let mut output = vec![Complex32::new(0.0, 0.0); in_ref.len()];

    // Calculate omega
    let omega = Complex32::new(PI / in_ref.len() as f32, 0.0).exp();

    // Start the actual fft
    do_fft(&SplatAccessor::new(&input), &mut output, &omega, is_reverse);
    Ok(output)
}

fn do_fft(
    input: &SplatAccessor<Complex32>,
    output: &mut [Complex32],
    omega: &Complex32,
    is_reverse: bool,
) {
    let n = input.len();

    // check that the input is always a power of two
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(n, output.len());

    // base case
    if n == 1 {
        output[0] = input[0];
        return;
    }

    // Calculate omega
    let inner_omega = Complex32::new(PI / n as f32, 0.0).exp();

    // Inner call
    let (left, right) = input.splat();
    do_fft(&left, &mut output[..n / 2], &inner_omega, is_reverse);
    do_fft(&right, &mut output[n / 2..], &inner_omega, is_reverse);

    let mut omega_acc = *omega;
    for i in 0..n / 2 {
        let x = output[i] + omega_acc * output[i + n / 2];
        let y = output[i] - omega_acc * output[i + n / 2];

        output[i] = x;
        output[i + n / 2] = y;

        // Update omega for the next round
        omega_acc *= omega;
    }
}

#[derive(Debug, Clone)]
struct SplatAccessor<'a, B> {
    factor: usize,
    index: usize,
    inner: &'a [B],
}

impl<'a, B> SplatAccessor<'a, B> {
    fn new<A: 'a + AsRef<[B]>>(inner: &'a A) -> Self {
        Self {
            factor: 1,
            index: 0,
            inner: inner.as_ref(),
        }
    }

    fn splat(&self) -> (Self, Self) {
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

    fn len(&self) -> usize {
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
