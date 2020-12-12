use fon::{
   chan::{Ch64, Channel},
   sample::{Sample, Sample1},
   Sink,
};
use pitch::{Frequencer, Wavelet};
use plotters::prelude::*;
use std::{cell::RefCell, rc::Rc};
use twang::Synth;

struct FreqSinkInner {
   buf: Vec<f64>,
   freq: Frequencer,
   current_wavelet: Option<Wavelet>,
   capacity: usize,
}

#[derive(Clone)]
struct FreqSink(Rc<RefCell<FreqSinkInner>>);

impl Sink<Sample1<Ch64>> for FreqSink {
   fn sample_rate(&self) -> u32 {
      self.0.borrow().freq.sample_rate() as u32
   }

   fn sink_sample(&mut self, sample: Sample1<Ch64>) {
      let mut cell = self.0.borrow_mut();
      cell.buf.push(sample.channels()[0].to_f64());
      cell.capacity -= 1;

      if cell.buf.len() == cell.freq.step_size() {
         let buf = cell.buf.clone();
         let wv = cell.freq.feed_audio(&buf);
         cell.current_wavelet = Some(wv);
      }
   }

   fn capacity(&self) -> usize {
      self.0.borrow().capacity
   }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
   let sink = FreqSink(Rc::new(RefCell::new(FreqSinkInner {
      buf: vec![],
      freq: Frequencer::new(48000, 2048, 256).unwrap(),
      current_wavelet: None,
      capacity: (1 << 16),
   })));

   // Generate synth
   let mut synth = Synth::new();
   synth.gen(sink.clone(), |fc| fc.freq(5000.0).sine());

   let bins = sink.0.borrow().current_wavelet.as_ref().unwrap().clone();
   let freq = bins
      .bins
      .iter()
      .map(|x| (x.frequency as f32, x.amplitude as f32));
   //.collect::<Vec<_>>();

   // plot the results
   let root = BitMapBackend::new("test.png", (640, 480)).into_drawing_area();
   root.fill(&WHITE)?;
   let mut chart = ChartBuilder::on(&root)
      .x_label_area_size(30)
      .y_label_area_size(30)
      .build_cartesian_2d(0f32..24_000f32, 0f32..6f32)?;

   chart.configure_mesh().draw()?;

   chart.draw_series(LineSeries::new(freq, &RED))?;

   // chart.draw_series(LineSeries::new(
   //    (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
   //    &RED,
   // ))?;

   Ok(())
}
