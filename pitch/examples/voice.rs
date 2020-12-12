#![allow(unused_imports)]

use fon::{
   chan::{Ch16, Channel},
   mono::Mono16,
   sample::{Sample, Sample1},
   Sink, Stream,
};
use pasts::prelude::*;
use std::{cell::RefCell, collections::VecDeque};

use wavy::{Microphone, MicrophoneId, SpeakerId};

use pitch::{Frequencer, Resynth, Wavelet};

/// The program's shared state.
struct State {
   freq: Frequencer,
   resynth: Resynth,
   wavelets: VecDeque<Wavelet>,
}

/// Microphone task (record audio).
async fn microphone_task(state: &RefCell<State>, mut mic: Microphone<Ch16>) {
   let mut buffer = vec![];

   loop {
      let mut sample = mic.record().await;
      let step_size = state.borrow().freq.step_size();

      while let Some(stream) = sample.stream_sample() {
         let chan = stream.channels()[0];
         buffer.push(chan.to_f64());

         // If there is enough data in the buffer we process it into a wavelet
         if buffer.len() >= step_size {
            let mut state = state.borrow_mut();
            let mut wv = state.freq.feed_audio(&buffer[..]);

            wv.pitch_shift(1.0 / 1.6);

            state.wavelets.push_back(wv);
            buffer.clear();
         }
      }
   }
}

/// Speakers task (play recorded audio).
async fn speakers_task(state: &RefCell<State>) {
   let mut speakers = SpeakerId::default().connect::<Mono16>().unwrap();

   loop {
      // wait for request
      let mut sink = speakers.play().await;
      let mut state = state.borrow_mut();

      // get the new wavelet if we have one
      let wv = state.wavelets.pop_front();
      println!("stored {} wavelets", state.wavelets.len());

      // allocate new output
      let mut output = vec![0.0f64; state.resynth.step_size()];

      // do the synthesis
      state.resynth.pull_audio(&mut output, wv);

      for s in output.iter() {
         sink.sink_sample(Sample1::new::<Ch16>((*s as f64).into()));
      }
   }
}

// /// Program start.
async fn start() {
   let microphone = MicrophoneId::default().connect().unwrap();
   let sample_rate = microphone.sample_rate();
   println!(
      "Microphone connected, sample rate {}",
      microphone.sample_rate()
   );

   let state = RefCell::new(State {
      freq: Frequencer::new(sample_rate as usize, 4096, 1024).unwrap(),
      resynth: Resynth::new(sample_rate as usize, 4096, 1024).unwrap(),
      wavelets: VecDeque::new(),
   });
   // Create speaker and microphone tasks.
   task! {
       let speakers = speakers_task(&state);
       let microphone = microphone_task(&state, microphone)
   }
   // Wait for first task to complete.
   poll![speakers, microphone].await;
}

/// Start the async executor.
fn main() {
   exec!(start());
}
