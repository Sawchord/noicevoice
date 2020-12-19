pub fn frequency_to_approx_note(frequency: f64) -> f64 {
   12.0 * f64::log2(frequency / 440.0)
}

pub fn note_to_frequency(note: f64) -> f64 {
   440.0 * 2.0f64.powf(note / 12.0)
}

#[derive(Clone)]
pub struct Note(i16);

impl Note {
   pub fn from_approx(note: f64) -> (Self, f64) {
      let prec_note = note.round() as i16;
      let deriv = note / prec_note as f64;

      (Note(prec_note), deriv)
   }

   pub fn value(&self) -> i16 {
      self.0
   }

   fn get_name(&self) -> &'static str {
      match self.0.rem_euclid(12) {
         0 => "A",
         1 => "A#",
         2 => "B",
         3 => "C",
         4 => "C#",
         5 => "D",
         6 => "D#",
         7 => "E",
         8 => "F",
         9 => "F#",
         10 => "G",
         11 => "G#",
         _ => panic!(),
      }
   }

   fn get_octave(&self) -> i8 {
      (self.0 as f32 / 12.0).floor() as i8 + 4
   }
}

impl core::fmt::Debug for Note {
   fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
      write!(f, "{}{}", self.get_name(), self.get_octave())
   }
}
