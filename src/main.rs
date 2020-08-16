#![windows_subsystem = "windows"]
use kludgine::prelude::*;
mod assets;
mod clicks;
mod element;
mod game;
mod title;
use assets::{Loop, LoopKind};
use game::{Game, GameCommand};
use rand::prelude::*;
use rodio::Source;
use title::TitleScreen;

fn main() {
    SingleWindowApplication::run(Chillscapes::default());
}

fn beats_per_second(tempo: f32) -> f32 {
    tempo / 60.
}

fn seconds_per_beat(tempo: f32) -> f32 {
    1. / beats_per_second(tempo)
}

struct Chillscapes {
    backdrop: Entity<Image>,
    pads: &'static Loop,
    scene_state: KludgineHandle<SceneState>,
    state: State,
}

pub struct SceneState {
    elapsed: f32,
    beat: f32,
    measure: usize,
    tempo: f32,
    beats_per_loop: usize,
}

enum State {
    TitleScreen(Entity<TitleScreen>),
    InGame(Entity<Game>),
    StartGame,
}

impl Default for Chillscapes {
    fn default() -> Self {
        let pads = {
            let mut rng = thread_rng();
            Loop::all()
                .iter()
                .filter(|p| p.kind == LoopKind::PADs)
                .choose(&mut rng)
                .unwrap()
        };

        Self {
            pads,
            backdrop: Default::default(),
            scene_state: KludgineHandle::new(SceneState {
                elapsed: 0.,
                beat: 0.,
                measure: 0,
                tempo: assets::TEMPO,
                beats_per_loop: assets::BEATS_PER_LOOP,
            }),
            state: State::TitleScreen(Entity::default()),
        }
    }
}

impl Window for Chillscapes {}

impl WindowCreator<Chillscapes> for Chillscapes {
    fn window_title() -> String {
        "Chillscapes".to_owned()
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    StartGame,
}

#[async_trait]
impl InteractiveComponent for Chillscapes {
    type Message = Message;
    type Output = ();
    type Input = ();

    async fn receive_message(
        &mut self,
        context: &mut Context,
        message: Self::Message,
    ) -> KludgineResult<()> {
        match message {
            Message::StartGame => {
                if let State::TitleScreen(title) = &self.state {
                    context.remove(*title).await;
                }

                self.state = State::StartGame;

                Ok(())
            }
        }
    }
}

#[async_trait]
impl Component for Chillscapes {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        context
            .scene_mut()
            .register_font(&include_font!("../assets/fonts/Audiowide-Regular.ttf"))
            .await;

        context
            .set_style_sheet(
                Style {
                    font_family: Some("Audiowide".to_string()),
                    font_size: Some(16.),
                    alignment: Some(Alignment::Center),
                    color: Some(Color::new(1.0, 0.0, 0.9, 1.0)),
                    ..Default::default()
                }
                .into(),
            )
            .await;

        let backdrop_texture = include_texture!("../assets/whitevault/space/SceneOne.png")?;
        let sprite = Sprite::single_frame(backdrop_texture).await;

        self.backdrop = self
            .new_entity(
                context,
                Image::new(sprite)
                    .options(ImageOptions::default().scaling(ImageScaling::AspectFill)),
            )
            .bounds(AbsoluteBounds::from(Surround::uniform(
                Dimension::from_points(0.),
            )))
            .insert()
            .await?;

        if let Some(device) = rodio::default_output_device() {
            let sink = rodio::Sink::new(&device);
            sink.append(self.pads.source.clone().repeat_infinite());
            sink.set_volume(0.6);
            sink.detach();
        }

        self.state = State::TitleScreen(
            self.new_entity(context, TitleScreen::default())
                .callback(|_| Message::StartGame)
                .insert()
                .await?,
        );

        // self.game = self.new_entity(context, Game::default()).insert().await?;
        Ok(())
    }

    async fn layout(
        &mut self,
        _context: &mut StyledContext,
    ) -> KludgineResult<Box<dyn LayoutSolver>> {
        let child = match &self.state {
            State::TitleScreen(title) => title.index(),
            State::InGame(game) => game.index(),
            State::StartGame => unreachable!(),
        };
        Layout::absolute()
            .child(
                self.backdrop,
                AbsoluteBounds::from(Surround::uniform(Dimension::from_points(0.))),
            )?
            .child(
                child,
                AbsoluteBounds::from(Surround::uniform(Dimension::from_points(0.))),
            )?
            .layout()
    }
    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        if let State::StartGame = &self.state {
            self.state = State::InGame(
                self.new_entity(context, Game::new(self.scene_state.clone()))
                    .insert()
                    .await?,
            );
        }

        if let Some(elapsed) = context.scene().elapsed().await {
            let mut scene_data = self.scene_state.write().await;
            scene_data.elapsed += elapsed.as_secs_f32();
            let absolute_beat = scene_data.elapsed / seconds_per_beat(scene_data.tempo);
            let measure = absolute_beat as usize / scene_data.beats_per_loop;
            let is_new_measure = scene_data.measure != measure;
            scene_data.measure = measure;
            scene_data.beat = absolute_beat % scene_data.beats_per_loop as f32;

            if let State::InGame(game) = &self.state {
                game.send(GameCommand::SetBeat {
                    is_new_measure,
                    beat: scene_data.beat,
                    measure,
                })
                .await?;
            }
        }

        Ok(())
    }
}
