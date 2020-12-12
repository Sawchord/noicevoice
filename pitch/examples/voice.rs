#![allow(unused_imports)]

use fon::{
   chan::{Ch16, Channel},
   mono::Mono16,
   sample::{Sample, Sample1},
   Sink, Stream,
};
use pasts::prelude::*;
use std::cell::RefCell;

use wavy::{Microphone, MicrophoneId, SpeakerId};

fn main() {}
// /// The program's shared state.
// struct State {
//    /// Temporary buffer for holding real-time audio samples.
//    pitch: PitchShifter,
// }

// /// Microphone task (record audio).
// async fn microphone_task(state: &RefCell<State>, mut mic: Microphone<Ch16>) {
//    let mut buffer = vec![];

//    loop {
//       let mut sample = mic.record().await;
//       while let Some(stream) = sample.stream_sample() {
//          let chan = stream.channels()[0];
//          buffer.push(chan.to_f64() as f32);
//       }

//       let mut state = state.borrow_mut();
//       state.pitch.feed_audio(&buffer);
//       buffer.clear();
//    }
// }

// /// Speakers task (play recorded audio).
// async fn speakers_task(state: &RefCell<State>) {
//    let mut speakers = SpeakerId::default().connect::<Mono16>().unwrap();

//    loop {
//       let mut output: [f32; 128] = [0.0; 128];

//       let mut sink = speakers.play().await;
//       let mut state = state.borrow_mut();
//       let _num_bytes = state.pitch.pull_audio(&mut output);

//       for s in output.iter() {
//          sink.sink_sample(Sample1::new::<Ch16>((*s as f64).into()));
//       }
//    }
// }

// /// Program start.
// async fn start() {
//    let microphone = MicrophoneId::default().connect().unwrap();
//    println!(
//       "Microphone connected, sample rate {}",
//       microphone.sample_rate()
//    );

//    let mut pitch = PitchShifter::new(microphone.sample_rate() as usize, 4096, 512).unwrap();
//    pitch.set_pitch_shift(1.0);

//    let state = RefCell::new(State { pitch });
//    // Create speaker and microphone tasks.
//    task! {
//        let speakers = speakers_task(&state);
//        let microphone = microphone_task(&state, microphone)
//    }
//    // Wait for first task to complete.
//    poll![speakers, microphone].await;
// }

// /// Start the async executor.
// fn main() {
//    exec!(start());
// }
