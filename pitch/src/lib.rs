#![allow(dead_code)]

// TODO: #1 Use Complex<T> instead of Complex32 in FFT
// TODO: #2 Use Index<> instead AsRef<> constraint in SplatAccessor
// TODO: #3 <-(#2) Use VecDequeue in Input and Output Buffer
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

use crate::fft::{fft, rfft};

pub struct FrequencyBin {
    amplitude: f32,
    frequency: f32,
}

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
            in_buf: vec![],
            out_buf: vec![],
            output_index: 0,
            in_phase_buf: vec![0.0; frame_size],
            out_phase_buf: vec![0.0; frame_size],
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

                    // map back onto rad
                    if phase_diff > PI {
                        phase_diff -= PI;
                    }
                    if phase_diff < -PI {
                        phase_diff += PI;
                    }
                    debug_assert!(phase_diff < PI && phase_diff > -PI);

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
            let output = output
                .iter()
                .enumerate()
                .map(|(k, x)| {
                    let window =
                        -0.5 * f32::cos(2.0 * PI * k as f32 / self.frame_size as f32) + 0.5;
                    2.0 * window * x.im * (self.frame_size as f32 / self.oversampling_rate as f32)
                })
                // quantize
                .map(|x| x as i16)
                .collect::<Vec<_>>();

            // add new output to buffer
            self.out_buf.extend_from_slice(&output[..]);

            // remove data from input
            self.in_buf.drain(..audio.len());
        }

        // Copy audio from output buffer to the audio buffer
        if audio.len() >= self.out_buf.len() {
            // If there is not enough data, we extend the buffer with 0 bytes
            let bytes_left = self.out_buf.len();
            audio[..bytes_left].copy_from_slice(&self.out_buf[..]);
            audio[bytes_left..].iter_mut().for_each(|x| *x = 0);

            // empty the buffer completely
            self.out_buf.clear();
        } else {
            audio.copy_from_slice(&self.out_buf[..audio.len()]);
            self.out_buf.drain(..audio.len());
        }
    }
}
