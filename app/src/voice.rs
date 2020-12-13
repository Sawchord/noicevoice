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
      let mut sample = mic.record().await;
      let step_size = state.borrow().freq.step_size();

      // Stop the frequencer
      if !RUNNING.load(Ordering::Relaxed) {
         // Pull out the empty data
         while let Some(_) = sample.stream_sample() {}
         continue;
      }

      while let Some(stream) = sample.stream_sample() {
         let chan = stream.channels()[0];
         buffer.push(chan.to_f64());

         // If there is enough data in the buffer we process it into a wavelet
         if buffer.len() >= step_size {
            let mut state = state.borrow_mut();

            if state.wavelets.len() <= 1000 {
               let mut wv = state.freq.feed_audio(&buffer[..]);

               // Get the pitch shift
               let pitch = get_slider_value("pitch").unwrap_or(1.0);
               wv.pitch_shift(pitch);

               state.wavelets.push_back(wv);
            }
            buffer.clear();
         }
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

      if state.wavelets.len() <= 10 {
         // allocate new output
         let mut output = vec![0.0f64; state.resynth.step_size()];

         // do the synthesis
         state.resynth.pull_audio(&mut output, wv);

         // Get the gain factor

         let volume = get_slider_value("volume").unwrap_or(50.0);
         let gain = 1.0242687596005495f64.powf(volume) - 1.0;
         for s in output.iter() {
            // TODO: Implement volume
            sink.sink_sample(Sample1::new::<Ch16>((*s * gain).into()));
         }
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

pub fn init_frequencer() {
   let promise = async move {
      start().await;
      Ok(JsValue::null())
   };
   let _ = future_to_promise(promise);
}

pub fn start_frequencer() {
   RUNNING.store(true, Ordering::Relaxed);
}

pub fn stop_frequencer() {
   RUNNING.store(false, Ordering::Relaxed);
}

fn get_slider_value<T>(name: &str) -> Option<T>
where
   T: core::str::FromStr,
{
   use wasm_bindgen::JsCast;
   let val = web_sys::window()?
      .document()?
      .get_element_by_id(name)?
      .dyn_into::<web_sys::HtmlInputElement>()
      .ok()?
      .value();

   val.parse().ok()
}
