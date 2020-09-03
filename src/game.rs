use crate::{
    assets::{Animation, Loop, LoopKind},
    element::{Element, ElementCommand, ElementEvent},
    SceneState,
    clicks::{Clicks, ClickCommand},
};
use kludgine::prelude::*;
use rand::prelude::*;

struct SpawnedElement {
    element: Entity<Element>,
    audio_loop: &'static Loop,
    animation: &'static Animation,
    location: Rect,
    being_destroyed: bool,
}

pub struct Game {
    scene_state: KludgineHandle<SceneState>,
    help_text: Entity<Label>,
    clicks: Entity<Clicks>,
    elements: Vec<SpawnedElement>,
    pending_element: Option<Entity<Element>>,
    lead: Option<rodio::Sink>,
    last_spawned_element_measure: Option<usize>,
    next_loop_to_spawn: Option<&'static Loop>,
    volume: f32,
}

const MAX_VOLUME: f32 = 0.7;
const QUIET_VOLUME: f32 = 0.3;

impl Game {
    pub fn new(scene_state: KludgineHandle<SceneState>) -> Self {
        Self {
            scene_state,
            elements: Vec::default(),
            pending_element: None,
            lead: None,
            last_spawned_element_measure: None,
            next_loop_to_spawn: None,
            volume: MAX_VOLUME,
            help_text: Default::default(),
            clicks: Default::default(),
        }
    }

    fn random_available_loop(&self) -> Option<&'static Loop> {
        let mut rng = thread_rng();
        Loop::all()
            .iter()
            .filter(|l| {
                !l.beats.is_empty()
                    && !self
                        .elements
                        .iter()
                        .any(|el| !el.being_destroyed && el.audio_loop.kind == l.kind)
            })
            .choose(&mut rng)
    }

    fn find_spawn_location(&self, scene_size: Size, frame_size: Size<u32>) -> Rect {
        let mut rng = thread_rng();

        loop {
            let x = rng.gen_range(32., scene_size.width - frame_size.width as f32 - 64.);
            let y = rng.gen_range(32., scene_size.height - frame_size.height as f32 - 64.);

            let rect = Rect::sized(
                Point::new(x, y),
                Size::new(frame_size.width as f32, frame_size.height as f32),
            );

            if !self
                .elements
                .iter()
                .any(|se| !se.being_destroyed && se.location.intersects_with(&rect))
            {
                return rect;
            }
        }
    }

    fn pick_next_spawn(&mut self) {
        self.next_loop_to_spawn = Some({
            if let Some(audio_loop) = self.random_available_loop() {
                audio_loop
            } else {
                let oldest_element = self.elements.get_mut(0).unwrap();
                oldest_element.being_destroyed = true;

                self.random_available_loop().unwrap()
            }
        });
    }

    async fn spawn_new_element(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        let scene_state = self.scene_state.read().await;
        let scene_size = context.scene().size().await.to_f32();
        if scene_size.area() > 0. {
            if let Some(audio_loop) = self.next_loop_to_spawn.take() {


            let animation = {
                let animations = Animation::all().await;
                let mut rng = thread_rng();
                animations
                    .iter()
                    .filter(|a| {
                        !self
                            .elements
                            .iter()
                            .any(|el| !el.being_destroyed && el.animation.id == a.id)
                    })
                    .choose(&mut rng)
                    .unwrap()
            };

            let frame_size = animation.sprite.size().await.unwrap();

            let location = self.find_spawn_location(scene_size, frame_size);

            let element = self
                .new_entity(
                    context,
                    Element::new(
                        scene_state.beats_per_loop,
                        scene_state.tempo,
                        self.volume,
                        animation,
                        audio_loop,
                    ),
                )
                .bounds(AbsoluteBounds {
                    left: Dimension::from_points(location.origin.x),
                    top: Dimension::from_points(location.origin.y),
                    width: Dimension::from_points(location.size.width),
                    height: Dimension::from_points(location.size.height),
                    ..Default::default()
                })
                .callback(GameMessage::ElementEvent)
                .insert()
                .await?;

            self.elements.push(SpawnedElement {
                element: element.clone(),
                audio_loop,
                animation,
                location,
                being_destroyed: false,
            });

            self.pending_element = Some(element);
            self.last_spawned_element_measure = Some(scene_state.measure);
        }
    }

        Ok(())
    }

    async fn generate_leads(&mut self) {
        let scene_state = self.scene_state.read().await;
        if self.last_spawned_element_measure.unwrap_or_default() != scene_state.measure {
            let mut rng = thread_rng();
            // Don't always play leads
            if rng.gen_bool(0.66) {
                let lead_loop = Loop::all()
                    .iter()
                    .filter(|l| l.kind == LoopKind::Leads)
                    .choose(&mut rng)
                    .unwrap();

                if let Some(device) = rodio::default_output_device() {
                    let sink = rodio::Sink::new(&device);
                    sink.append(lead_loop.source.clone());
                    sink.set_volume(self.volume);
                    self.lead = Some(sink);
                }
            } else {
                self.lead = None;
            }
        }
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;

        if let Some(lead) = &self.lead {
            lead.set_volume(volume);
        }
    }
}

#[derive(Clone, Debug)]
pub enum GameMessage {
    ElementEvent(ElementEvent),
}

#[derive(Clone, Debug)]
pub enum GameCommand {
    SetBeat {
        is_new_measure: bool,
        beat: f32,
        measure: usize,
    },
}

#[async_trait]
impl InteractiveComponent for Game {
    type Message = GameMessage;
    type Input = GameCommand;
    type Output = ();

    async fn receive_message(
        &mut self,
        _context: &mut Context,
        message: Self::Message,
    ) -> KludgineResult<()> {
        match message {
            GameMessage::ElementEvent(ElementEvent::LoopLockedIn) => {
                self.pending_element = None;
            }
            GameMessage::ElementEvent(ElementEvent::Soloing(soloing_element)) => {
                self.set_volume(QUIET_VOLUME);
                for element in self.elements.iter() {
                    if element.element.index() != soloing_element {
                        element
                            .element
                            .send(ElementCommand::SetVolume(QUIET_VOLUME))
                            .await?;
                    }
                }
            }
            GameMessage::ElementEvent(ElementEvent::StoppingSolo) => {
                self.set_volume(MAX_VOLUME);
                for element in self.elements.iter() {
                    element
                        .element
                        .send(ElementCommand::SetVolume(self.volume))
                        .await?;
                }
            }
            GameMessage::ElementEvent(ElementEvent::Success(window_position)) => {
                self.clicks.send(ClickCommand::SetStatus { success: true, location: Some(window_position)}).await?;
            }
            GameMessage::ElementEvent(ElementEvent::Failure(window_position)) => {
                self.clicks.send(ClickCommand::SetStatus { success: false, location: window_position}).await?;}
        }

        Ok(())
    }

    async fn receive_input(
        &mut self,
        context: &mut Context,
        command: Self::Input,
    ) -> KludgineResult<()> {
        match command {
            GameCommand::SetBeat {
                is_new_measure,
                beat,
                measure,
            } => {
                if is_new_measure {
                    if self.pending_element.is_none() {
                        self.pick_next_spawn();
                    } else {
                        self.generate_leads().await;
                    }
                }

                for element in &self.elements {
                    if is_new_measure && element.being_destroyed {
                        context.remove(&element.element).await;
                    } else {
                        element
                            .element
                            .send(ElementCommand::SetBeat {
                                beat,
                                measure,
                                is_new_measure,
                            })
                            .await?;
                    }
                }

                if is_new_measure {
                    self.elements.retain(|e| !e.being_destroyed);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Component for Game {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        self.help_text = self.new_entity(context, 
            Label::new("Click on each new element to the rhythm you hear. \nRelax and enjoy the music.")
        ).bounds(AbsoluteBounds {
                left: Dimension::from_points(16.),
                top: Dimension::from_points(16.),
                right: Dimension::from_points(16.),
                ..Default::default()
            }).insert().await?;

            self.clicks = self.new_entity(context, Clicks::default()).bounds(Surround::uniform(Dimension::from_points(0.)).into()).insert().await?;
        Ok(())
    }

    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
            self.spawn_new_element(context).await?;
        Ok(())
    }
}
