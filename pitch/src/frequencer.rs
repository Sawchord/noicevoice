use alloc::{collections::VecDeque, sync::Arc, vec::Vec};
use core::{f64::consts::PI, iter::FromIterator};
use num_complex::Complex64;
use rustfft::{FFTplanner, FFT};

use crate::{FrequencyBin, Wavelet};

pub struct Frequencer {
   sample_rate: usize,
   frame_size: usize,
   step_size: usize,
   freqs_per_bin: f64,
   phase_diff_per_frame: f64,
   oversampling_rate: f64,
   sample_buf: VecDeque<f64>,
   phase_buf: Vec<f64>,
   fft: Arc<dyn FFT<f64>>,
}

impl Frequencer {
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
         fft: FFTplanner::new(false).plan_fft(frame_size),
      })
   }

   pub fn sample_rate(&self) -> usize {
      self.sample_rate
   }

   pub fn step_size(&self) -> usize {
      self.step_size
   }

   pub fn feed_audio(&mut self, audio: &[f64]) -> Wavelet {
      // We can only accept slices that are exact step size long
      assert_eq!(audio.len(), self.step_size);

      // Add the new audio to the end of the buffer
      self.sample_buf.extend(audio.iter());
      self.sample_buf.drain(..self.step_size());
      debug_assert_eq!(self.sample_buf.len(), self.frame_size);

      let mut frame = self
         .sample_buf
         .iter()
         // apply windowing
         .enumerate()
         .map(|(k, x)| {
            let window = -0.5 * f64::cos(2.0 * PI * k as f64 / self.frame_size as f64) + 0.5;
            window * x
         })
         // map to complex numbers
         .map(|x| Complex64::new(x, 0.0))
         .collect::<Vec<_>>();

      // do the actual transformation
      //let fft1 = crate::fft::fft(&frame).unwrap();
      use rustfft::num_traits::Zero;
      let mut fft = vec![Complex64::zero(); self.frame_size];
      self.fft.process(&mut frame, &mut fft);

      // for i in 0..self.frame_size {
      //    assert_eq!(fft1[i], fft[i]);
      // }

      // transform
      let bins = fft
         .iter()
         // FFT is symetric, therefore we only need lower half
         .take(self.frame_size / 2)
         // transform into polar
         // now r is amplitutde and theta is phase
         .map(|x| x.to_polar())
         .enumerate()
         .map(|(k, (amp, phase))| {
            // get the phase difference to prior frame and update
            let mut phase_diff = phase - self.phase_buf[k];
            self.phase_buf[k] = phase_diff;

            // calculate difference to expected phase
            phase_diff -= k as f64 * self.phase_diff_per_frame;

            let n = (f64::abs(phase_diff) / PI) as usize;

            // map back onto rad
            if phase_diff > 0.0 {
               phase_diff -= n as f64 * PI;
               if phase_diff > PI {
                  phase_diff = PI;
               }
            } else {
               phase_diff += n as f64 * PI;
               if phase_diff < -PI {
                  phase_diff = -PI;
               }
            }
            assert!(phase_diff <= PI && phase_diff >= -PI);

            // compute frequency deviation
            let freq_dev = self.oversampling_rate * phase_diff / (2.0 * PI);

            // compute frequency
            let freq = (k as f64 + freq_dev) * self.freqs_per_bin;

            FrequencyBin {
               amplitude: amp,
               frequency: freq,
            }
         })
         .collect::<Vec<_>>();

      Wavelet { bins }
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   use alloc::rc::Rc;
   use core::cell::RefCell;
   use fon::{chan::Channel, mono::Mono64, Sample, Sink};
   use twang::Synth;

   struct FreqSinkInner {
      buf: Vec<f64>,
      freq: Frequencer,
      current_wavelet: Option<Wavelet>,
      capacity: usize,
   }

   #[derive(Clone)]
   struct FreqSink(Rc<RefCell<FreqSinkInner>>);

   impl Sink<Mono64> for FreqSink {
      fn sample_rate(&self) -> u32 {
         self.0.borrow().freq.sample_rate() as u32
      }

      fn sink_sample<Z: Sample>(&mut self, sample: Z) {
         let mut cell = self.0.borrow_mut();
         cell.buf.push(sample.channels()[0].to_f64());
         cell.capacity -= 1;

         if cell.buf.len() == cell.freq.step_size() {
            let buf = cell.buf.clone();
            let wv = cell.freq.feed_audio(&buf);
            cell.current_wavelet = Some(wv);
         }
      }

      fn capacity(&self) -> usize {
         self.0.borrow().capacity
      }
   }

   #[test]
   fn frequencer() {
      let sink = FreqSink(Rc::new(RefCell::new(FreqSinkInner {
         buf: vec![],
         freq: Frequencer::new(48000, 2048, 256).unwrap(),
         current_wavelet: None,
         capacity: (1 << 16),
      })));

      // Generate synth
      let mut synth = Synth::new();
      synth.gen(sink.clone(), |fc| fc.freq(1000.0).sine());

      let base_freq = sink
         .0
         .borrow()
         .current_wavelet
         .as_ref()
         .unwrap()
         .base_freq();

      assert!(base_freq >= 995.0 && base_freq <= 1005.0);
   }
}
