use kludgine::prelude::*;
mod game;
use game::Game;

fn main() {
    SingleWindowApplication::run(Chillscapes::default());
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
                AbsoluteBounds::from(Surround::uniform(Dimension::Points(0.))),
            )?
            .layout()
    }
}
