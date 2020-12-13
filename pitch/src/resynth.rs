use alloc::{collections::VecDeque, vec::Vec};
use core::{f64::consts::PI, iter::FromIterator};
use num_complex::Complex64;

use crate::Wavelet;

use alloc::sync::Arc;
use rustfft::{FFTplanner, FFT};

pub struct Resynth {
   sample_rate: usize,
   frame_size: usize,
   step_size: usize,
   freqs_per_bin: f64,
   phase_diff_per_frame: f64,
   oversampling_rate: f64,
   sample_buf: VecDeque<f64>,
   phase_buf: Vec<f64>,
   ifft: Arc<dyn FFT<f64>>,
   last_wavelet: Wavelet,
}

impl Resynth {
   pub fn new(sample_rate: usize, frame_size: usize, step_size: usize) -> Result<Self, ()> {
      if !frame_size.is_power_of_two() {
         return Err(());
      }

      if step_size >= frame_size {
         return Err(());
      }

      Ok(Self {
         sample_rate,
         frame_size,
         step_size,
         freqs_per_bin: sample_rate as f64 / frame_size as f64,
         phase_diff_per_frame: 2.0 * PI * step_size as f64 / frame_size as f64,
         oversampling_rate: frame_size as f64 / step_size as f64,
         sample_buf: VecDeque::from_iter(core::iter::repeat(0.0).take(frame_size)),
         phase_buf: vec![0.0; frame_size],
         ifft: FFTplanner::new(true).plan_fft(frame_size),
         last_wavelet: Wavelet::empty(frame_size / 2),
      })
   }

   pub fn sample_rate(&self) -> usize {
      self.sample_rate
   }

   pub fn step_size(&self) -> usize {
      self.step_size
   }

   pub fn pull_audio(&mut self, audio: &mut [f64], wavelet: Option<Wavelet>) {
      assert!(audio.len() >= self.step_size());

      let wavelet = match wavelet {
         Some(wv) => wv,
         None => self.last_wavelet.clone(),
      };

      // do the reverse steps
      let mut frame = wavelet
         .bins
         .iter()
         // calculate back to amplitude and phase
         .enumerate()
         .map(|(k, bin)| {
            // calculate frequency deviation from frequency
            let mut freq_dev = bin.frequency - k as f64 * self.freqs_per_bin;
            freq_dev = freq_dev / self.freqs_per_bin;

            // calculate phase difference from frequency deviation
            let mut phase_diff = 2.0 * PI * freq_dev / self.oversampling_rate;

            // add possible overlap
            phase_diff += k as f64 * self.phase_diff_per_frame;

            // add phase diff to output phase buffer
            self.phase_buf[k] += phase_diff;

            // return amplitude and phase
            (bin.amplitude, self.phase_buf[k])
         })
         // turn back into complex numbers
         .map(|(amp, phase)| Complex64::from_polar(amp, phase))
         // remove the imaginary part
         //.map(|x| Complex64::new(x.re, 0.0))
         .collect::<Vec<_>>();

      // reverse fft
      //let output = rfft(&output).unwrap();
      // Extend the frame with 0s
      // FIXME: Maybe instead mirror the values?
      frame.extend(core::iter::repeat(Complex64::zero()).take(self.frame_size / 2));

      //let mut frame2 = frame.clone();
      //frame2.extend(frame.iter().rev());
      //frame2.extend(std::iter::repeat(Complex64::zero()).take(self.frame_size / 2));

      use rustfft::num_traits::Zero;
      let mut ifft = vec![Complex64::zero(); self.frame_size];
      self.ifft.process(&mut frame, &mut ifft);

      // apply window and turn into real numbers
      let frame_size = self.frame_size;
      let oversampling_rate = self.oversampling_rate;
      let output = ifft.iter().enumerate().map(|(k, x)| {
         let window = -0.5 * f64::cos(2.0 * PI * k as f64 / frame_size as f64) + 0.5;
         2.0 * window * x.re / ((frame_size / 2) as f64 * oversampling_rate)
         //2.0 * window * x.re / (frame_size / 2) as f64
         //window * x.re
      });

      // drain buffer into audio output
      audio
         .iter_mut()
         .zip(self.sample_buf.drain(..self.step_size()))
         .for_each(|(x, y)| *x = y);

      // extend buffer by stepsize elements
      self
         .sample_buf
         .extend(core::iter::repeat(0.0).take(self.step_size));

      // accumulate output to buffer
      self
         .sample_buf
         .iter_mut()
         .zip(output)
         .for_each(|(x, y)| *x += y);

      debug_assert_eq!(self.sample_buf.len(), self.frame_size);
   }
}
