use super::*;

struct RunningArgs {
    episode: usize,
    step: usize,
    game_state: GameState,
}

enum EpisodeState {
    Init { episode: usize },
    Running(RunningArgs),
}

pub struct Spatium<T: SpatiumSys> {
    sys: SpatiumSysHelper<T>,
    max_episodes: usize,
    game: Box<Game + Send>,
    network: Box<Network + Send>,
    episode_state: Option<EpisodeState>,
}

impl<T: SpatiumSys> Spatium<T> {
    pub fn new<P: IntoModelParameters, G: IntoGameParameters>(
        game_parameters: G,
        model_parameters: P,
        sys: T,
        rng: RcRng,
        max_episodes: usize,
    ) -> Result<Spatium<T>, String> {
        let game_parameters = game_parameters.into_parameters()?;
        sys.info(&format!("Parsed game params: {:?}", game_parameters));

        let model_parameters = model_parameters.into_parameters()?;
        sys.info(&format!("Parsed model params: {:?}", model_parameters));

        let game = game_parameters.into_game(rng.clone());
        let network = model_parameters.to_model(rng, game.io());

        let n = Spatium {
            sys: SpatiumSysHelper::new(sys),
            max_episodes: max_episodes,
            game: game,
            network: network,
            episode_state: None,
        };
        n.sys.info("Running Spatium");
        Ok(n)
    }
    fn process_inital_state(&mut self, rng: RcRng, episode: usize) -> (EpisodeState, StepResult) {
        let (game_state, _score, _done) = self.game.reset(rng);

        let rendering_info = self.game.rendering_info();
        (
            EpisodeState::Running(RunningArgs {
                episode: episode,
                step: 1,
                game_state: game_state,
            }),
            StepResult::new(episode, 0, "None".into(), false, rendering_info),
        )
    }
    fn process_running_state(
        &mut self,
        rng: RcRng,
        args: RunningArgs,
    ) -> (EpisodeState, StepResult) {
        let sys = self.sys.clone();
        let sys = sys.read();

        let RunningArgs {
            episode,
            step,
            game_state,
        } = args;

        // get next action from model
        let (action, _val) = self.network
            .next_action(&*sys, Some(rng.clone()), &game_state);

        // advance game using action
        let (game_state1, score1, done) = self.game.step(&self.sys, &action);

        // pass result to model and collect any metrics
        let metrics = self.network.result(
            &*sys,
            rng.clone(),
            game_state,
            &action,
            &game_state1,
            score1,
            done,
        );

        let result = StepResult::new(
            episode,
            step,
            format!("{}", action),
            episode > self.max_episodes,
            self.game.rendering_info(),
        ).with_metrics(metrics);

        if done {
            self.sys
                .debug(&format!("Episode {} complete at step {}", episode, step));
            (
                EpisodeState::Init {
                    episode: episode + 1,
                },
                result.with_episode_result(EpisodeResult {
                    steps: step,
                    score: step as f32,
                }),
            )
        } else {
            (
                EpisodeState::Running(RunningArgs {
                    episode: episode,
                    step: step + 1,
                    game_state: game_state1,
                }),
                result,
            )
        }
    }
    pub fn step(&mut self, rng: RcRng) -> StepResult {
        let episode_state = self.episode_state.take();
        let (new_state, result) = match episode_state {
            None => self.process_inital_state(rng, 0),
            Some(EpisodeState::Init { episode }) => self.process_inital_state(rng, episode),
            Some(EpisodeState::Running(args)) => self.process_running_state(rng, args),
        };
        self.episode_state = Some(new_state);
        result
    }
}

#[cfg(test)]
pub mod tests {
    extern crate rand;

    use super::*;
    use game::Game1Parameters;
    use network::SingleLayerNetworkParameters;
    use network::single_layer::DynamicValue;
    use rayon::prelude::*;

    pub struct SpatiumDummy;

    impl SpatiumSys for SpatiumDummy {
        // fn debug(&self, s: &str) {
        //     println!("debug: {}", s);
        // }
        fn info(&self, _s: &str) {
            // println!("info: {}", s);
        }
        fn random(&mut self) -> f64 {
            rand::random()
        }
    }

    #[test]
    fn it_works() {
        let rng = RcRng::new(Box::new(rand::weak_rng()));
        let game = GameParameters::Game1(Default::default());
        let model = ModelParameters::QNetwork(Default::default());
        let mut spat = Spatium::new(game, model, SpatiumDummy {}, rng.clone(), 1000).unwrap();
        loop {
            let result = spat.step(rng.clone());
            // println!("{}", serde_json::to_string(&result).unwrap());
            if let Some(ref ep_result) = result.episode_result {
                println!("{}", serde_json::to_string(&ep_result).unwrap());
            }
            // if result.done {
            //     break;
            // }
        }
    }

    #[test]
    fn it_parameters() {
        // let model = ModelParameters::QNetwork(Default::default());

        // let minibatch_sizes = [1, 10];
        // let expierence_buffer_sizes = [10, 100];
        // let max_steps = [30];

        let minibatch_sizes = [1, 10, 100];
        let expierence_buffer_sizes = [10, 100];
        let max_steps = [10, 20, 30, 40];

        let mut choices = vec![];
        for a in minibatch_sizes.iter() {
            for b in expierence_buffer_sizes.iter() {
                for c in max_steps.iter() {
                    choices.push((*a, *b, *c));
                }
            }
        }

        use std::cell::RefCell;

        let mut results: Vec<_> = choices
            .into_par_iter()
            .map(|choice| {
                let rng = RcRng::new(Box::new(rand::weak_rng()));

                println!("{:?}", choice);
                let minibatch_size = choice.0;
                let expierence_buffer_size = choice.1;
                let max_steps = choice.2;

                let model = ModelParameters::QNetwork(SingleLayerNetworkParameters {
                    minibatch_size: minibatch_size,
                    expierence_buffer_size: expierence_buffer_size,
                    discount_factor: 0.99,
                    learning: DynamicValue {
                        initial_rate: 0.1,
                        final_rate: 0.01,
                        final_episode: 900,
                    },
                    exploration: DynamicValue {
                        initial_rate: 1.0,
                        final_rate: 0.01,
                        final_episode: 900,
                    },
                });
                // println!(
                //     "Model parameters: {}",
                //     serde_json::to_string(&model).unwrap()
                // );

                let game = GameParameters::Game1(Game1Parameters {
                    max_steps: max_steps,
                    size: 4,
                    random: true,
                });
                // println!("Game parameters: {}", serde_json::to_string(&game).unwrap());

                let mut spat =
                    Spatium::new(game, model, SpatiumDummy {}, rng.clone(), 1000).unwrap();

                let mut all_scores = vec![];
                for _ in 0..6 {
                    let mut scores = vec![];
                    loop {
                        let result = spat.step(rng.clone());
                        // println!("{}", serde_json::to_string(&result).unwrap());
                        if let Some(ref ep_result) = result.episode_result {
                            // println!("{}", serde_json::to_string(&ep_result).unwrap());
                            scores.push(ep_result.score);
                            if scores.len() > 30 {
                                scores.remove(0);
                            }
                        }
                        if result.done {
                            break;
                        }
                    }
                    all_scores.extend(scores);
                }

                let score = all_scores.iter().fold(0., |a, n| a + n) / all_scores.len() as f32;
                println!("score={}", score);
                (choice, score)
            })
            .collect();

        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        for r in results.iter() {
            println!("{:?}", r);
        }
    }
}
