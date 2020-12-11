use fon::{
   chan::Ch16,
   mono::Mono16,
   sample::{Sample, Sample1},
   Sink, Stream,
};
use pasts::prelude::*;
use std::cell::RefCell;
use wavy::{Microphone, MicrophoneId, SpeakerId};

use pitch::PitchShifter;

/// The program's shared state.
struct State {
   /// Temporary buffer for holding real-time audio samples.
   //buffer: Audio<Mono16>,
   pitch: PitchShifter,
}

/// Microphone task (record audio).
async fn microphone_task(state: &RefCell<State>, mut mic: Microphone<Ch16>) {
   let mut buffer = vec![];

   loop {
      let mut sample = mic.record().await;
      match sample.stream_sample() {
         Some(stream) => {
            let chan = stream.channels()[0];
            buffer.push(i16::from(chan));
         }
         None => continue,
      };

      let mut state = state.borrow_mut();
      state.pitch.feed_audio(&buffer);
      buffer.clear();
   }
}

/// Speakers task (play recorded audio).
async fn speakers_task(state: &RefCell<State>) {
   let mut speakers = SpeakerId::default().connect::<Mono16>().unwrap();

   loop {
      let mut sink = speakers.play().await;
      let mut state = state.borrow_mut();
      let output = state.pitch.pull_audio(1);

      for s in output {
         sink.sink_sample(Sample1::new::<Ch16>(s.into()));
      }
   }
}

/// Program start.
async fn start() {
   let microphone = MicrophoneId::default().connect().unwrap();
   println!(
      "Microphone connected, sample rate {}",
      microphone.sample_rate()
   );

   let pitch = PitchShifter::new(microphone.sample_rate() as usize, 4096, 1024).unwrap();

   let state = RefCell::new(State {
      //buffer: Audio::with_silence(microphone.sample_rate(), 0),
      pitch,
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
