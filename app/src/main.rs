#![recursion_limit = "256"]

use yew::prelude::*;

enum Msg {}

#[allow(dead_code)]
struct Model {
    link: ComponentLink<Self>,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        todo!()
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <div class="card">
                    <section class="hero card-header is-primary">
                        <div class="hero-body columns">
                            <div class="container column">
                            <h1 class="title">
                                {{"NoiseVoice"}}
                            </h1>
                            <h2 class="subtitle">
                                {{"Experiments involving Rust + WASM/Yew + Audioprocessing"}}
                            </h2>
                            </div>
                            <button
                                id="play"
                                class="button column mx-4 is-large is-primary mt-6"
                            >
                            // TODO: Use start/wait stop icon
                            // TODO: Change colors
                            // TODO: Interactivity
                            {{"Play/Stop"}}
                        </button>
                        </div>
                    </section>

                    <div class="card-content">
                        <div class="columns level">
                            <p class="column level-item is-one-fifths">
                                {{"Volume: "}}
                            </p>
                            <input
                                id="volume"
                                class="slider column level-item is-primary is-large is-four-fifths"
                                step="1"
                                min="0"
                                max="100"
                                value="50"
                                type="range"
                            />
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
