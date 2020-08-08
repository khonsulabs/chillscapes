use kludgine::prelude::*;
mod element;
mod game;
use game::Game;

fn main() {
    SingleWindowApplication::run(Chillscapes::default());
}

fn beats_per_second(tempo: f32) -> f32 {
    tempo / 60.
}

fn seconds_per_beat(tempo: f32) -> f32 {
    1. / beats_per_second(tempo)
}

#[derive(Default)]
struct Chillscapes {
    game: Entity<Game>,
}
impl Window for Chillscapes {}

impl WindowCreator<Chillscapes> for Chillscapes {
    fn window_title() -> String {
        "Chillscapes".to_owned()
    }
}
impl StandaloneComponent for Chillscapes {}

#[async_trait]
impl Component for Chillscapes {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        self.game = self.new_entity(context, Game::default()).insert().await?;
        Ok(())
    }

    async fn layout(
        &mut self,
        _context: &mut StyledContext,
    ) -> KludgineResult<Box<dyn LayoutSolver>> {
        Layout::absolute()
            .child(
                self.game,
                AbsoluteBounds::from(Surround::uniform(Dimension::from_points(0.))),
            )?
            .layout()
    }
}
