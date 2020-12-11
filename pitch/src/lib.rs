#![allow(dead_code)]

pub mod fft;
pub(crate) mod splat;

use core::f32::consts::PI;
use num_complex::Complex32;

use crate::fft::fft;

pub struct PitchShifter {
    sample_rate: usize,
    frame_size: usize,
    step_size: usize,
    pitch_shift: f32,
    freqs_per_bin: f32,
    phase_diff_per_frame: f32,
    oversampling_rate: f32,
    in_buf: Vec<i16>,
    out_buf: Vec<i16>,
    output_index: usize,
    phase_buf: Vec<f32>,
}

impl PitchShifter {
    pub fn new(sample_rate: usize, frame_size: usize, step_size: usize) -> Result<Self, ()> {
        if !frame_size.is_power_of_two() {
            return Err(());
        }
        Ok(Self {
            sample_rate,
            frame_size,
            step_size,
            pitch_shift: 1.0,
            freqs_per_bin: sample_rate as f32 / frame_size as f32,
            phase_diff_per_frame: 2.0 * PI * step_size as f32 / frame_size as f32,
            oversampling_rate: frame_size as f32 / step_size as f32,
            in_buf: vec![],
            out_buf: vec![],
            output_index: 0,
            phase_buf: vec![0.0; frame_size],
        })
    }

    pub fn set_pitch_shift(&mut self, pitch: f32) {
        self.pitch_shift = pitch;
    }

    pub fn feed_audio(&mut self, audio: &mut [i16]) {
        // Add the new audio to the end of the buffer
        self.in_buf.extend_from_slice(audio);

        // if we have enough audio, process data
        if self.in_buf.len() >= self.frame_size {
            // normalize, apply windowing function and make comples
            let frame = self.in_buf[..self.frame_size]
                .iter()
                // normalize
                //.map(|x| *x as f32 / (2 << 15) as f32)
                .map(|x| *x as f32)
                // apply windowing
                .enumerate()
                .map(|(k, x)| {
                    (-0.5 * f32::cos(2.0 * PI * k as f32 / self.frame_size as f32) + 0.5) * x
                })
                // map to complex numbers
                .map(|x| Complex32::new(x, 0.0))
                .collect::<Vec<_>>();

            // do the actual transformation
            let fft = fft(&frame).unwrap();

            // transform
            let freqs = fft
                .iter()
                // transform into polar
                // now r is amplitutde and theta is phase
                .map(|x| x.to_polar())
                .enumerate()
                .map(|(k, (amp, phase))| {
                    // get the phase difference to prior frame and update
                    let phase_diff = phase - self.phase_buf[k];
                    self.phase_buf[k] = phase_diff;

                    // calculate the new phase difference

                    (amp, phase)
                })
                .collect::<Vec<_>>();

            todo!()
        }

        // Copy audio from output buffer to the audio buffer
        // Output index is a pointer into the out_buf, which creases every
        // time we poll for output
        if self.output_index + audio.len() >= self.out_buf.len() {
            // If there is not enough data, we extend the buffer with 0 bytes
            let bytes_left = self.out_buf.len() - self.output_index;
            audio[..bytes_left].copy_from_slice(&self.out_buf[self.output_index..]);
            audio[bytes_left..].iter_mut().for_each(|x| *x = 0);
            self.output_index = self.out_buf.len();
        } else {
            let new_output_index = self.output_index + audio.len();
            audio.copy_from_slice(&self.out_buf[self.output_index..new_output_index]);
            self.output_index = new_output_index;
        }
    }
}
