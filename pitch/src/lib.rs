#![allow(dead_code)]

// TODO: #1 Use Complex<T> instead of Complex32 in FFT
// TODO: #4 Fixed Size Frames using own Types
// TODO: #5 Wavelet Type
// TODO: #6 <- (#5) Factor out Frontend into Frequencer
// TODO: #7 <- (#5) Factor Resynth into Wavelet
// TODO: #8 <- (#6, #7, #4) Factor out input and output buffer
// TODO: #9 <- (#5) Custom algs

pub mod fft;
pub(crate) mod splat;

use core::f32::consts::PI;
use num_complex::Complex32;
use std::{collections::VecDeque, iter::FromIterator};

use crate::fft::{fft, rfft};

pub struct FrequencyBin {
    amplitude: f32,
    frequency: f32,
}

pub struct Wavelet {
    frame_size: usize,
    bins: Vec<FrequencyBin>,
}

pub struct Frequencer {
    sample_rate: usize,
    frame_size: usize,
    step_size: usize,
    freqs_per_bin: f32,
    phase_diff_per_frame: f32,
    oversampling_rate: f32,
    expected_phase: f32,
    phase_buf: Vec<f32>,
}

pub struct PitchShifter {
    sample_rate: usize,
    frame_size: usize,
    step_size: usize,
    pitch_shift: f32,
    freqs_per_bin: f32,
    phase_diff_per_frame: f32,
    oversampling_rate: f32,
    in_buf: VecDeque<f32>,
    out_buf: VecDeque<f32>,
    in_phase_buf: Vec<f32>,
    out_phase_buf: Vec<f32>,
}

impl PitchShifter {
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
            pitch_shift: 1.0,
            freqs_per_bin: sample_rate as f32 / frame_size as f32,
            phase_diff_per_frame: 2.0 * PI * step_size as f32 / frame_size as f32,
            oversampling_rate: frame_size as f32 / step_size as f32,
            in_buf: VecDeque::from_iter(std::iter::repeat(0.0).take(frame_size)),
            out_buf: VecDeque::from_iter(std::iter::repeat(0.0).take(frame_size)),
            in_phase_buf: vec![0.0; frame_size],
            out_phase_buf: vec![0.0; frame_size],
        })
    }

    pub fn set_pitch_shift(&mut self, pitch: f32) {
        self.pitch_shift = pitch;
    }

    pub fn feed_audio(&mut self, audio: &[f32]) {
        // We are making local copies so we don't have to worry about ownership
        let buf_len = self.out_buf.len();
        let frame_size = self.frame_size;
        let oversampling_rate = self.oversampling_rate;

        // Add the new audio to the end of the buffer
        self.in_buf.extend(audio.iter());

        // if we have enough audio, process data
        if self.in_buf.len() >= self.frame_size {
            // normalize, apply windowing function and make comples
            let frame = self
                .in_buf
                .iter()
                .take(self.frame_size)
                // apply windowing
                .enumerate()
                .map(|(k, x)| {
                    let window =
                        -0.5 * f32::cos(2.0 * PI * k as f32 / self.frame_size as f32) + 0.5;
                    window * x
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
                    let mut phase_diff = phase - self.in_phase_buf[k];
                    self.in_phase_buf[k] = phase_diff;

                    // calculate difference to expected phase
                    phase_diff -=
                        k as f32 * 2.0 * PI * (self.step_size as f32 / self.frame_size as f32);

                    let n = (f32::abs(phase_diff) / PI) as usize;

                    // map back onto rad
                    if phase_diff > 0.0 {
                        phase_diff -= n as f32 * PI;
                        if phase_diff > PI {
                            phase_diff = PI;
                        }
                    } else {
                        phase_diff += n as f32 * PI;
                        if phase_diff < -PI {
                            phase_diff = -PI;
                        }
                    }
                    assert!(phase_diff <= PI && phase_diff >= -PI);

                    // compute frequency deviation
                    let freq_dev = self.oversampling_rate * phase_diff / (2.0 * PI);

                    // compute frequency
                    let freq = (k as f32 + freq_dev) * self.freqs_per_bin;

                    FrequencyBin {
                        amplitude: amp,
                        frequency: freq,
                    }
                })
                .collect::<Vec<_>>();

            // manipulate the frequencies
            let new_freqs = freqs
                .iter()
                .map(|bin| FrequencyBin {
                    amplitude: bin.amplitude,
                    frequency: bin.frequency * self.pitch_shift,
                })
                .collect::<Vec<_>>();

            // do the reverse steps
            let output = new_freqs
                .iter()
                // calculate back to amplitude and phase
                .enumerate()
                .map(|(k, bin)| {
                    // calculate frequency deviation from frequency
                    let freq_dev = bin.frequency / self.freqs_per_bin - k as f32;

                    // calculate phase difference from frequency deviation
                    let mut phase_diff = 2.0 * PI * freq_dev / self.oversampling_rate;

                    // add possible overlap
                    phase_diff +=
                        k as f32 * 2.0 * PI * (self.step_size as f32 / self.frame_size as f32);

                    // add phase diff to output phase buffer
                    self.out_phase_buf[k] += phase_diff;

                    // return amplitude and phase
                    (bin.amplitude, self.out_phase_buf[k])
                })
                // turn back into complex numbers
                .map(|(amp, phase)| Complex32::from_polar(amp, phase))
                // remove the imaginary part
                .map(|x| Complex32::new(x.re, 0.0))
                .collect::<Vec<_>>();

            // reverse fft
            let output = rfft(&output).unwrap();

            // apply window and turn into real numbers
            let output = output.iter().enumerate().map(|(k, x)| {
                let window = -0.5 * f32::cos(2.0 * PI * k as f32 / frame_size as f32) + 0.5;
                2.0 * window * x.im * (frame_size as f32 / oversampling_rate as f32)
                //2.0 * window * x.im
            });

            // extend buffer by stepsize elements
            self.out_buf
                .extend(std::iter::repeat(0.0).take(self.step_size));

            // accumulate output to buffer
            self.out_buf
                .iter_mut()
                .skip((buf_len - frame_size) + frame_size)
                .zip(output)
                .for_each(|(x, y)| *x += y);

            // remove data from input
            self.in_buf.drain(..self.step_size);
        }
    }

    pub fn pull_audio(&mut self, output: &mut [f32]) -> usize {
        // We can only output bytes that are outside of the frame
        // because they are permanent.
        let bytes_left = self.out_buf.len().saturating_sub(self.frame_size);
        // Calculate, how many bytes we need to output
        let num_samples = usize::min(bytes_left, output.len());

        // Fill the output with samples from the buffer
        output
            .iter_mut()
            .zip(self.out_buf.drain(..num_samples))
            .for_each(|(x, y)| *x = y);

        num_samples
    }
}
