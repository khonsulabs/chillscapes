use crate::{
    assets::{Animation, Loop, LoopKind},
    element::{Element, ElementCommand, ElementEvent},
    seconds_per_beat,
};
use kludgine::prelude::*;
use rand::prelude::*;
use rodio::Source;

struct SpawnedElement {
    element: Entity<Element>,
    audio_loop: &'static Loop,
    animation: &'static Animation,
}

pub struct Game {
    backdrop: Entity<Image>,
    elapsed: f32,
    measure: usize,
    tempo: f32,
    beats_per_loop: usize,
    pads: &'static Loop,
    elements: Vec<SpawnedElement>,
    pending_element: Option<Entity<Element>>,
    lead: Option<rodio::Sink>,
}

impl Default for Game {
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
            backdrop: Entity::default(),
            elapsed: 0.,
            measure: 0,
            pads,
            elements: Vec::default(),
            tempo: 83.,
            beats_per_loop: 32,
            pending_element: None,
            lead: None,
        }
    }
}

impl Game {
    async fn spawn_new_element(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        let scene_size = context.scene().size().await.to_f32();
        if scene_size.area() > 0. {
            let audio_loop = {
                let mut rng = thread_rng();
                if let Some(audio_loop) = Loop::all()
                    .iter()
                    .filter(|l| {
                        !l.beats.is_empty()
                            && !self.elements.iter().any(|el| el.audio_loop.kind == l.kind)
                    })
                    .choose(&mut rng)
                {
                    audio_loop
                } else {
                    // Can't spawn a new one
                    // TODO: Get rid of the oldest element and spawn a new one
                    // But until we get more loops, it's kinda pointless, it rotates in new artwork
                    return Ok(());
                }
            };

            let animation = {
                let animations = Animation::all().await;
                let mut rng = thread_rng();
                animations
                    .iter()
                    .filter(|a| !self.elements.iter().any(|el| el.animation.id == a.id))
                    .choose(&mut rng)
                    .unwrap()
            };

            let frame_size = animation.sprite.size().await.unwrap();

            let location = {
                let mut rng = thread_rng();
                let x = rng.gen_range(0., scene_size.width - frame_size.width as f32);
                let y = rng.gen_range(0., scene_size.height - frame_size.height as f32);
                println!("Left, Top: {}, {}", x, y);
                AbsoluteBounds {
                    left: Dimension::from_points(x),
                    top: Dimension::from_points(y),
                    width: Dimension::from_points(frame_size.width as f32),
                    height: Dimension::from_points(frame_size.height as f32),
                    ..Default::default()
                }
            };

            let element = self
                .new_entity(
                    context,
                    Element::new(self.beats_per_loop, self.tempo, animation, audio_loop),
                )
                .bounds(location)
                .callback(GameMessage::ElementEvent)
                .insert()
                .await?;

            self.elements.push(SpawnedElement {
                element,
                audio_loop,
                animation,
            });

            self.pending_element = Some(element);
        }

        Ok(())
    }

    fn generate_leads(&mut self) {
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
                self.lead = Some(sink);
            }
        } else {
            self.lead = None;
        }
    }
}

#[derive(Clone, Debug)]
pub enum GameMessage {
    ElementEvent(ElementEvent),
}

#[async_trait]
impl InteractiveComponent for Game {
    type Message = GameMessage;
    type Input = ();
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
        }

        Ok(())
    }
}

#[async_trait]
impl Component for Game {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
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
            sink.detach();
        }

        self.spawn_new_element(context).await?;

        Ok(())
    }

    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        if self.pending_element.is_none() {
            self.spawn_new_element(context).await?;
        }

        if let Some(elapsed) = context.scene().elapsed().await {
            self.elapsed += elapsed.as_secs_f32();

            let absolute_beat = self.elapsed / seconds_per_beat(self.tempo);
            let measure = absolute_beat as usize / self.beats_per_loop;
            if measure != self.measure {
                self.generate_leads();
                self.measure = measure;
            }
            let beat = absolute_beat % self.beats_per_loop as f32;

            for element in &self.elements {
                element
                    .element
                    .send(ElementCommand::SetBeat { beat, measure })
                    .await?;
            }
        }

        Ok(())
    }
}
