use num_complex::Complex32;
use std::f32;
use std::f32::consts::PI;

use crate::splat::SplatAccessor;

pub fn fft<B: AsRef<[Complex32]>>(input: B) -> Result<Vec<Complex32>, ()> {
   fft_inner(input, false)
}

pub fn rfft<B: AsRef<[Complex32]>>(input: B) -> Result<Vec<Complex32>, ()> {
   fft_inner(input, true)
}

fn fft_inner<B: AsRef<[Complex32]>>(input: B, is_reverse: bool) -> Result<Vec<Complex32>, ()> {
   let in_ref = input.as_ref();
   if !in_ref.len().is_power_of_two() {
      return Err(());
   }

   let mut output = vec![Complex32::new(0.0, 0.0); in_ref.len()];

   // Calculate omega
   let omega = calc_omega(in_ref.len(), is_reverse);

   // Start the actual fft
   do_fft(&SplatAccessor::new(&input), &mut output, &omega, is_reverse);

   if is_reverse {
      let n = Complex32::new(output.len() as f32, 0.0);
      for i in 0..output.len() {
         output[i] = output[i] / n;
      }
   }

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
   let inner_omega = calc_omega(n, is_reverse);

   // Inner call
   let (left, right) = input.splat();
   do_fft(&left, &mut output[..n / 2], &inner_omega, is_reverse);
   do_fft(&right, &mut output[n / 2..], &inner_omega, is_reverse);

   let mut omega_acc = Complex32::new(1.0, 0.0);
   for i in 0..n / 2 {
      let x = output[i] + omega_acc * output[i + n / 2];
      let y = output[i] - omega_acc * output[i + n / 2];

      output[i] = x;
      output[i + n / 2] = y;

      // Update omega for the next round
      omega_acc = omega_acc * omega;
   }
}

fn calc_omega(n: usize, is_reverse: bool) -> Complex32 {
   if is_reverse {
      Complex32::new(0.0, -2.0 * PI / n as f32).exp()
   } else {
      Complex32::new(0.0, 2.0 * PI / n as f32).exp()
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use float_cmp::approx_eq;
   //use rand::Rng;

   macro_rules! complex_approx {
      ($a: expr, $b: expr) => {
         assert!(approx_eq!(f32, $a.re, $b.re));
         assert!(approx_eq!(f32, $a.im, $b.im));
      };
   }

   #[test]
   fn fft_smoke() {
      let input = [5, 3, 2, 1]
         .iter()
         .map(|x| Complex32::new(*x as f32, 0.0))
         .collect::<Vec<_>>();

      let output = fft(&input).unwrap();

      complex_approx!(output[0], Complex32::new(11.0, 0.0));
      complex_approx!(output[1], Complex32::new(3.0, 2.0));
      complex_approx!(output[2], Complex32::new(3.0, 0.0));
      complex_approx!(output[3], Complex32::new(3.0, -2.0));

      let output = rfft(&output).unwrap();
      complex_approx!(output[0], Complex32::new(5.0, 0.0));
      complex_approx!(output[1], Complex32::new(3.0, 0.0));
      complex_approx!(output[2], Complex32::new(2.0, 0.0));
      complex_approx!(output[3], Complex32::new(1.0, 0.0));
   }

   // #[test]
   // fn fft_inversion() {
   //     // Get random white noise
   //     let mut rng = rand::thread_rng();
   //     let mut test_input = [0; 8192];
   //     rng.fill(&mut test_input[..]);

   //     // Turn it a complex representation
   //     let test_input = test_input
   //         .iter()
   //         .map(|x| x % (2 << 16))
   //         .map(|x| x as f32 / (2 << 16) as f32)
   //         .map(|x| Complex32::new(x, 0.0))
   //         .collect::<Vec<_>>();

   //     // Do fft
   //     let fft_output = fft(&test_input).unwrap();
   //     let rfft_output = rfft(fft_output).unwrap();

   //     for (i, o) in test_input.iter().zip(rfft_output.iter()) {
   //         dbg!(&i, &o);
   //         complex_approx!(i, o);
   //     }
   // }
}
