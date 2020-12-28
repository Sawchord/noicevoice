use core::{
   cell::RefCell,
   str::FromStr,
   sync::atomic::{AtomicBool, Ordering},
};
use fon::{
   chan::{Ch16, Channel},
   mono::Mono16,
   Sample, Sink, Stream,
};
use pasts::prelude::*;
use pitch::{
   notes::{frequency_to_approx_note, Note},
   Frequencer, Resynth, Wavelet,
};
use std::collections::VecDeque;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::future_to_promise;
use wavy::{Microphone, MicrophoneId, SpeakerId};

static RUNNING: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

struct State {
   freq: Frequencer,
   resynth: Resynth,
   wavelets: VecDeque<Wavelet>,
   freq_avg: RunningAvg,
   update_counter: usize,
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
            state.update_counter += 1;

            // Skip processing wavelets if we are getting overflowed
            if state.wavelets.len() <= 1000 {
               let mut wv = state.freq.feed_audio(&buffer[..]);

               let freq = wv.base_freq();
               let freq = state.freq_avg.update(freq);

               if state.update_counter % 20 == 0 {
                  let note = frequency_to_approx_note(freq);
                  let (note, _prec) = Note::from_approx(note);
                  set_text("note_name", &format!("{:?}", note));
                  set_text("frequency", &format!("{:.2}Hz", freq));
               }

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
            sink.sink_sample(Mono16::new::<Ch16>((*s * gain).into()));
         }
      }
   }
}

// /// Program start.
async fn start() {
   let microphone = MicrophoneId::default().connect().unwrap();
   let sample_rate = microphone.sample_rate();

   let state = RefCell::new(State {
      freq: Frequencer::new(sample_rate as usize, 4096, 1024).unwrap(),
      resynth: Resynth::new(sample_rate as usize, 4096, 1024).unwrap(),
      wavelets: VecDeque::new(),
      freq_avg: RunningAvg::with_len(20),
      update_counter: 0,
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
   INITIALIZED.store(true, Ordering::Relaxed);

   let promise = async move {
      start().await;
      Ok(JsValue::null())
   };
   let _ = future_to_promise(promise);
}

pub fn start_frequencer() {
   if !INITIALIZED.load(Ordering::Relaxed) {
      init_frequencer();
   }

   RUNNING.store(true, Ordering::Relaxed);
}

pub fn stop_frequencer() {
   RUNNING.store(false, Ordering::Relaxed);
}

fn get_slider_value<T>(name: &str) -> Option<T>
where
   T: FromStr,
{
   let val = web_sys::window()?
      .document()?
      .get_element_by_id(name)?
      .dyn_into::<web_sys::HtmlInputElement>()
      .ok()?
      .value();

   val.parse().ok()
}

fn set_text(elem: &str, value: &str) {
   let val = web_sys::window()
      .unwrap()
      .document()
      .unwrap()
      .get_element_by_id(elem)
      .unwrap()
      .dyn_into::<web_sys::Node>()
      .unwrap();

   val.set_text_content(Some(value));
}

struct RunningAvg {
   len: usize,
   vals: VecDeque<f64>,
}

impl RunningAvg {
   fn with_len(len: usize) -> Self {
      Self {
         len,
         vals: VecDeque::new(),
      }
   }

   fn update(&mut self, val: f64) -> f64 {
      self.vals.push_back(val);

      if self.vals.len() >= self.len {
         self.vals.pop_front();
      }

      self.vals.iter().sum::<f64>() / self.len as f64
   }
}
