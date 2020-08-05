use crate::{
    beats_per_second,
    element::{Element, ElementCommand},
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
    elements: Vec<Entity<Element>>,
}

impl Default for Game {
    fn default() -> Self {
        let click_sound = include_bytes!("../assets/ecton/click.ogg").to_vec();
        let source = rodio::Decoder::new(Cursor::new(click_sound)).unwrap();
        let click = source.buffered();

        Self {
            backdrop: Entity::default(),
            elapsed: 0.,
            click,
            elements: Vec::default(),
            tempo: 60.,
            beats_per_loop: 4,
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
                    vec![0., 1., 2., 3., 3.33, 3.66],
                ),
            )
            .insert()
            .await?,
        );

        Ok(())
    }

    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        if let Some(elapsed) = context.scene().elapsed().await {
            self.elapsed += elapsed.as_secs_f32();

            let absolute_beat = self.elapsed / beats_per_second(self.tempo);
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
