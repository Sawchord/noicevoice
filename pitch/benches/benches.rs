use criterion::{criterion_group, criterion_main, Criterion};
use num_complex::Complex32;
use pitch::{fft, rfft};
use rand::Rng;

fn bench_fft(c: &mut Criterion) {
   let mut rng = rand::thread_rng();
   let mut test_input = [0; 8192];
   rng.fill(&mut test_input[..]);

   // Turn it a complex representation
   let test_input = test_input
      .iter()
      .map(|x| x % (2 << 16))
      .map(|x| x as f32 / (2 << 16) as f32)
      .map(|x| Complex32::new(x, 0.0))
      .collect::<Vec<_>>();

   c.bench_function("fft", |b| b.iter(|| fft(&test_input)));
   c.bench_function("rfft", |b| b.iter(|| rfft(&test_input)));
}

criterion_group!(benches, bench_fft);
criterion_main!(benches);
