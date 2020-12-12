use fon::{
   chan::{Ch64, Channel},
   sample::{Sample, Sample1},
   Sink,
};
use pitch::{Frequencer, Resynth, Wavelet};
use plotters::prelude::*;
use std::{cell::RefCell, rc::Rc};
use twang::Synth;

struct FreqSinkInner {
   buf: Vec<f64>,
   current_wavelet: Option<Wavelet>,
   capacity: usize,
   freq: Frequencer,
   resynth: Resynth,
   original: Vec<f64>,
   output: Vec<f64>,
}

#[derive(Clone)]
struct FreqSink(Rc<RefCell<FreqSinkInner>>);

impl Sink<Sample1<Ch64>> for FreqSink {
   fn sample_rate(&self) -> u32 {
      self.0.borrow().freq.sample_rate() as u32
   }

   fn sink_sample(&mut self, sample: Sample1<Ch64>) {
      let mut cell = self.0.borrow_mut();
      cell.original.push(sample.channels()[0].to_f64());
      cell.buf.push(sample.channels()[0].to_f64());
      cell.capacity -= 1;

      if cell.buf.len() == cell.freq.step_size() {
         // feed audio and clear buffer
         let buf = cell.buf.clone();
         let wv = cell.freq.feed_audio(&buf);
         cell.buf.clear();

         cell.current_wavelet = Some(wv.clone());
         let mut audio = vec![0.0; cell.resynth.step_size()];
         cell.resynth.pull_audio(&mut audio, Some(wv));
         cell.output.extend_from_slice(&audio);
      }
   }

   fn capacity(&self) -> usize {
      self.0.borrow().capacity
   }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
   let sink = FreqSink(Rc::new(RefCell::new(FreqSinkInner {
      buf: vec![],
      current_wavelet: None,
      capacity: (1 << 16),
      freq: Frequencer::new(48000, 4096, 1024).unwrap(),
      resynth: Resynth::new(48000, 4096, 1024).unwrap(),
      original: vec![],
      output: vec![],
   })));

   // Generate synth
   let mut synth = Synth::new();
   synth.gen(sink.clone(), |fc| fc.freq(1000.0).sine());

   let sink = sink.0.borrow();
   let mut bins = sink.current_wavelet.as_ref().unwrap().clone();
   bins.pitch_shift(1.4);
   let freq = bins
      .bins
      .iter()
      .filter(|x| x.amplitude != 0.0)
      .map(|x| (x.frequency as f32, x.amplitude as f32));

   let waves = sink
      .output
      .iter()
      .enumerate()
      .map(|(x, y)| (x as f32, *y as f32));

   let original = sink
      .original
      .iter()
      .enumerate()
      .map(|(x, y)| (x as f32, *y as f32));

   let root = SVGBackend::new("freq.svg", (1024, 768)).into_drawing_area();
   root.fill(&WHITE)?;
   let (upper, lower) = root.split_vertically(512);

   let mut freq_chart = ChartBuilder::on(&upper)
      .x_label_area_size(30)
      .y_label_area_size(30)
      .build_cartesian_2d(0f32..24_000f32, 0f32..10f32)?;

   freq_chart.configure_mesh().draw()?;
   freq_chart.draw_series(LineSeries::new(freq, &RED))?;

   let mut wave_chart = ChartBuilder::on(&lower)
      .x_label_area_size(30)
      .y_label_area_size(30)
      .build_cartesian_2d(10000f32..12400f32, -2f32..2f32)?;

   wave_chart.configure_mesh().draw()?;
   wave_chart.draw_series(LineSeries::new(waves, &RED))?;
   wave_chart.draw_series(LineSeries::new(original, &BLUE))?;

   Ok(())
}
