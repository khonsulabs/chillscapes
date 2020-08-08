use crate::{
    element::{Element, ElementCommand},
    seconds_per_beat,
};
use kludgine::prelude::*;
use rodio::{decoder::Decoder, source::Buffered, Source};
use std::io::Cursor;

pub struct Game {
    backdrop: Entity<Image>,
    elapsed: f32,
    tempo: f32,
    beats_per_loop: usize,
    click: Buffered<Decoder<Cursor<Vec<u8>>>>,
    pads: Buffered<Decoder<Cursor<Vec<u8>>>>,
    elements: Vec<Entity<Element>>,
}

impl Default for Game {
    fn default() -> Self {
        let click_sound = include_bytes!("../assets/pxzel/space/02-ARPs.mp3").to_vec();
        let source = rodio::Decoder::new(Cursor::new(click_sound)).unwrap();
        let click = source.buffered();
        let pads_sound = include_bytes!("../assets/pxzel/space/01-PADS.mp3").to_vec();
        let source = rodio::Decoder::new(Cursor::new(pads_sound)).unwrap();
        let pads = source.buffered();

        Self {
            backdrop: Entity::default(),
            elapsed: 0.,
            click,
            pads,
            elements: Vec::default(),
            tempo: 83.,
            beats_per_loop: 32,
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
                Image::new(sprite)
                    .options(ImageOptions::default().scaling(ImageScaling::AspectFill)),
            )
            .bounds(AbsoluteBounds::from(Surround::uniform(
                Dimension::from_points(0.),
            )))
            .insert()
            .await?;

        self.elements.push(
            self.new_entity(
                context,
                Element::new(
                    self.beats_per_loop,
                    self.tempo,
                    self.click.clone(),
                    (0..32).map(|beat| beat as f32).collect(),
                ),
            )
            .insert()
            .await?,
        );

        if let Some(device) = rodio::default_output_device() {
            let sink = rodio::Sink::new(&device);
            sink.append(self.pads.clone().repeat_infinite());
            sink.detach();
        }

        Ok(())
    }

    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        if let Some(elapsed) = context.scene().elapsed().await {
            self.elapsed += elapsed.as_secs_f32();

            let absolute_beat = self.elapsed / seconds_per_beat(self.tempo);
            let measure = absolute_beat as usize / self.beats_per_loop;
            let beat = absolute_beat % self.beats_per_loop as f32;

            for element in &self.elements {
                element
                    .send(ElementCommand::SetBeat { beat, measure })
                    .await?;
            }
        }

        Ok(())
    }
}
