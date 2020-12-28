# NoiceVoice

A demo project, exploring the possibilities of the processing power of WebAssembly in the browser.

Find a live demo [here](https://ltan.de/voice).

## Description:

This project is built in Rust.
It is a voice distortion tool, which uses the WebAudio API to capture sound, processes them through a Fast Fourier Transformation and returns the processed audio.

Note, that the procssing (including FFT) happens entirely in WebAssembly, 



## Technologies:
- **[Yew](https://github.com/yewstack/yew)** Rust PWA framework
- **[Bulma CSS](https://bulma.io/)**
- **[Web Autdio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API)**
- **[rustfft](https://github.com/awelkie/RustFFT)**