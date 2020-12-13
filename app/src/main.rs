#![recursion_limit = "256"]

mod voice;

use yew::prelude::*;

enum State {
    Idle,
    Playing,
}

enum Msg {
    PlayButtonPress,
}

#[allow(dead_code)]
struct Model {
    link: ComponentLink<Self>,
    state: State,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        voice::init_frequencer();
        Self {
            link,
            state: State::Idle,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        #[allow(unreachable_patterns)]
        match msg {
            Msg::PlayButtonPress => {
                match self.state {
                    State::Idle => {
                        voice::start_frequencer();
                        self.state = State::Playing;
                    }
                    State::Playing => {
                        voice::stop_frequencer();
                        self.state = State::Idle;
                    }
                    _ => (),
                }
                true
            }
            _ => false,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let (btn_color, btn_icon) = if self.is_playing() {
            ("is-danger", "fa-stop-circle")
        } else {
            ("is-success", "fa-play-circle")
        };

        html! {
            <div class="container">
                <div class="card">
                    <section class="hero card-header is-primary">
                        <div class="hero-body columns">
                            <div class="container column">
                            <h1 class="title">
                                {"NoiseVoice"}
                            </h1>
                            <h2 class="subtitle">
                                {"Experiments involving Rust + WASM/Yew + Audioprocessing"}
                            </h2>
                            </div>
                            <button
                                id="play"
                                class=("button column mx-4 is-large mt-6", btn_color)
                                onclick = self.link.callback(|_|Msg::PlayButtonPress)
                            >
                            <span class="icon">
                                <i class=("fas is-large", btn_icon)></i>
                            </span>
                        </button>
                        </div>
                    </section>

                    <div class="card-content">
                        {self.slider("volume", "Volume", "1", "0", "100")}
                        {self.slider("pitch", "Pitch", "0.01", "0.5", "2.0")}
                    </div>
                </div>
            </div>
        }
    }
}

impl Model {
    fn slider(&self, id: &str, name: &str, step: &str, min: &str, max: &str) -> Html {
        html! {
            <div class="columns level is-mobile">
                <p class="column level-item is-one-fifths is-hidden-mobile">
                    {name}
                </p>
                <input
                    id=id
                    class="slider column level-item is-large is-four-fifths is-primary"
                    step=step
                    min=min
                    max=max
                    type="range"
                />
            </div>
        }
    }

    fn is_playing(&self) -> bool {
        match self.state {
            State::Playing => true,
            _ => false,
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
