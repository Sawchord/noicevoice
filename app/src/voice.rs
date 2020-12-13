use core::sync::atomic::{AtomicBool, Ordering};
use fon::{
   chan::{Ch16, Channel},
   mono::Mono16,
   sample::{Sample, Sample1},
   Sink, Stream,
};
use pasts::prelude::*;
use pitch::{Frequencer, Resynth, Wavelet};
use std::{cell::RefCell, collections::VecDeque};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::future_to_promise;
use wavy::{Microphone, MicrophoneId, SpeakerId};
//use web_sys::{Document, HtmlInputElement, Window};

static RUNNING: AtomicBool = AtomicBool::new(false);

struct State {
   freq: Frequencer,
   resynth: Resynth,
   wavelets: VecDeque<Wavelet>,
}

/// Microphone task (record audio).
async fn microphone_task(state: &RefCell<State>, mut mic: Microphone<Ch16>) {
   let mut buffer = vec![];

   loop {
      // TODO: Stop
      let mut sample = mic.record().await;
      let step_size = state.borrow().freq.step_size();

      while let Some(stream) = sample.stream_sample() {
         let chan = stream.channels()[0];
         buffer.push(chan.to_f64());

         // If there is enough data in the buffer we process it into a wavelet
         if buffer.len() >= step_size {
            let mut state = state.borrow_mut();

            if state.wavelets.len() <= 5 {
               let mut wv = state.freq.feed_audio(&buffer[..]);

               // TODO: Implement pitch shift slider
               wv.pitch_shift(1.0);

               state.wavelets.push_back(wv);
            }
            buffer.clear();
         }
      }

      // Stop the frequencer
      if !RUNNING.load(Ordering::Relaxed) {
         break;
      }
   }
}

/// Speakers task (play recorded audio).
async fn speakers_task(state: &RefCell<State>) {
   // TODO: Stop
   let mut speakers = SpeakerId::default().connect::<Mono16>().unwrap();

   loop {
      // wait for request
      let mut sink = speakers.play().await;
      let mut state = state.borrow_mut();

      // get the new wavelet if we have one
      let wv = state.wavelets.pop_front();

      if state.wavelets.len() <= 5 {
         // allocate new output
         let mut output = vec![0.0f64; state.resynth.step_size()];

         // do the synthesis
         state.resynth.pull_audio(&mut output, wv);

         // Get the gain factor

         // 1.0242687596005495 audio factor
         for s in output.iter() {
            // TODO: Implement volume
            sink.sink_sample(Sample1::new::<Ch16>((*s as f64).into()));
         }
      }

      // Stop the frequencer
      if !RUNNING.load(Ordering::Relaxed) {
         break;
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

pub fn start_frequencer() {
   RUNNING.store(true, Ordering::Relaxed);

   let promise = async move {
      start().await;
      Ok(JsValue::null())
   };
   let _ = future_to_promise(promise);
}

pub fn stop_frequencer() {
   RUNNING.store(false, Ordering::Relaxed);
}
