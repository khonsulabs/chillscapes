use kludgine::prelude::*;
use rodio::{decoder::Decoder, source::Buffered, Source};
use std::{io::Cursor, time::Duration};

// [loop]
// category = "fill"
// beats = [1.375, ]

pub struct Beats {
    beats: Vec<f32>,
    current_beat: Option<usize>,
}

impl Beats {
    pub fn new(beats: Vec<f32>) -> Self {
        assert!(!beats.is_empty());

        Self {
            beats,
            current_beat: None,
        }
    }

    pub fn next_measure(&mut self) {
        self.current_beat = None;
    }

    pub fn next_beat(&self, beats_per_loop: usize) -> Option<Duration> {
        todo!()
    }

    pub fn advance(
        &mut self,
        mut beat: f32,
        beats_per_loop: usize,
        click: &Buffered<Decoder<Cursor<Vec<u8>>>>,
    ) {
        let next_beat = match self.current_beat {
            Some(index) => {
                if let Some(beat) = self.beats.get(index + 1) {
                    *beat
                } else {
                    beat -= beats_per_loop as f32;
                    self.beats[0]
                }
            }
            None => self.beats[0],
        };

        if beat > next_beat {
            println!("Next beat {}", beat);
            if let Some(device) = rodio::default_output_device() {
                let sink = rodio::Sink::new(&device);
                sink.append(click.clone());
                sink.detach();
            }

            let mut next_beat = self.current_beat.map(|beat| beat + 1).unwrap_or(0);
            if next_beat > self.beats.len() {
                next_beat = 0;
            }
            self.current_beat = Some(next_beat);
        }
    }
}

pub struct Scene {
    tempo: f32,
    beats_per_loop: usize,
    beats: Beats,
    measure: usize,
}

impl Scene {
    pub fn loop_duration(&self) -> f32 {
        let beats_per_second = self.tempo * 60.;
        beats_per_second * self.beats_per_loop as f32
    }
    pub fn beats_per_second(&self) -> f32 {
        60. / self.tempo
    }
}

pub struct Game {
    backdrop: Entity<Image>,
    star: Entity<Image>,
    elapsed: f32,
    click: Buffered<Decoder<Cursor<Vec<u8>>>>,
    scene: Scene,
}

impl Default for Game {
    fn default() -> Self {
        let click_sound = include_bytes!("../assets/ecton/click.ogg").to_vec();
        let scene = Scene {
            tempo: 60.,
            beats_per_loop: 4,
            beats: Beats::new(vec![0., 1., 2., 3.]),
            measure: 0,
        };

        let source = rodio::Decoder::new(Cursor::new(click_sound)).unwrap();
        let click = source.buffered();
        Self {
            backdrop: Entity::default(),
            star: Entity::default(),
            elapsed: 0.,
            click,
            scene,
        }
    }
}

impl StandaloneComponent for Game {}

#[async_trait]
impl Component for Game {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        let backdrop_texture = include_texture!("../assets/ecton/backdrop.png")?;
        let sprite = Sprite::single_frame(backdrop_texture).await;
        self.backdrop = self
            .new_entity(
                context,
                Image::new(sprite).options(ImageOptions {
                    scaling: Some(ImageScaling::AspectFill),
                }),
            )
            .insert()
            .await?;

        let sprite = include_aseprite_sprite!("../assets/ecton/star").await?;
        sprite.set_current_tag(Some("Normal")).await?;
        self.star = self
            .new_entity(context, Image::new(sprite))
            .insert()
            .await?;

        Ok(())
    }

    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        if let Some(elapsed) = context.scene().elapsed().await {
            self.elapsed += elapsed.as_secs_f32();

            let absolute_beat = self.elapsed / self.scene.beats_per_second();
            let measure = absolute_beat as usize / self.scene.beats_per_loop;
            let beat = absolute_beat % self.scene.beats_per_loop as f32;

            if self.scene.measure != measure {
                self.scene.measure = measure;
                self.scene.beats.next_measure();
            }

            self.scene
                .beats
                .advance(beat, self.scene.beats_per_loop, &self.click);
        }

        Ok(())
    }

    async fn layout(
        &mut self,
        _context: &mut StyledContext,
    ) -> KludgineResult<Box<dyn LayoutSolver>> {
        Layout::absolute()
            .child(
                self.backdrop,
                AbsoluteBounds::from(Surround::uniform(Dimension::Points(0.))),
            )?
            .child(
                self.star,
                AbsoluteBounds {
                    left: Dimension::Points(50.),
                    top: Dimension::Points(50.),
                    ..Default::default()
                },
            )?
            .layout()
    }
}
