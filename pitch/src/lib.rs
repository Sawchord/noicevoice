// TODO: #4 Fixed Size Frames using own Types
// TODO: #7 <- (#5) Factor Resynth into Wavelet
// TODO: #8 <- (#6, #7, #4) Factor out input and output buffer
// TODO: #9 <- (#5) Custom algs

pub mod fft;
mod frequencer;
pub use frequencer::Frequencer;
pub mod resynth;
pub use resynth::Resynth;
pub(crate) mod splat;

#[derive(Debug, Clone)]
pub struct FrequencyBin {
    pub amplitude: f64,
    pub frequency: f64,
}

#[derive(Debug, Clone)]
pub struct Wavelet {
    #[allow(dead_code)]
    pub bins: Vec<FrequencyBin>,
}

impl Wavelet {
    pub fn empty(frames: usize) -> Self {
        Wavelet {
            bins: (0..frames)
                .map(|_| FrequencyBin {
                    amplitude: 0.0,
                    frequency: 0.0,
                })
                .collect::<Vec<_>>(),
        }
    }

    // TODO: Make this fancy with iterators
    pub fn base_freq(&self) -> f64 {
        let mut max_freq = 0.0;
        let mut max_amp = 0.0;
        for bin in &self.bins {
            if bin.amplitude > max_amp {
                max_amp = bin.amplitude;
                max_freq = bin.frequency;
            }
        }
        max_freq
    }
}

// TODO: Funky functions on Wavelets
