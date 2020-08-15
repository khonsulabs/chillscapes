use kludgine::prelude::*;
use std::time::Instant;

#[derive(Default)]
pub struct Clicks {
    clicks: Entity<Image>,
    location: Point<Points>,
    last_click: Option<Instant>,
}

#[derive(Clone, Debug)]
pub enum ClickCommand {
    SetStatus {
        success: bool,
        location: Option<Point<Points>>,
    },
}

#[async_trait]
impl Component for Clicks {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        let clicks = include_aseprite_sprite!("../assets/whitevault/space/clicks")
            .await
            .unwrap();

        self.clicks = self
            .new_entity(
                context,
                Image::new(clicks).options(ImageOptions::default().alpha(0.0)),
            )
            .insert()
            .await?;
        Ok(())
    }
    async fn layout(
        &mut self,
        _context: &mut StyledContext,
    ) -> KludgineResult<Box<dyn LayoutSolver>> {
        Layout::absolute()
            .child(
                self.clicks,
                AbsoluteBounds {
                    left: Dimension::from_points(self.location.x),
                    top: Dimension::from_points(self.location.y),
                    ..Default::default()
                },
            )?
            .layout()
    }

    async fn update(&mut self, _context: &mut SceneContext) -> KludgineResult<()> {
        if let Some(last_click) = self.last_click {
            if let Some(duration) = Instant::now().checked_duration_since(last_click) {
                if duration.as_millis() > 250 {
                    self.clicks.send(ImageCommand::SetTag(None)).await?;
                    self.clicks.send(ImageCommand::SetAlpha(0.)).await?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl InteractiveComponent for Clicks {
    type Message = ();
    type Input = ClickCommand;
    type Output = ();

    async fn receive_input(
        &mut self,
        _context: &mut Context,
        command: Self::Input,
    ) -> KludgineResult<()> {
        match command {
            ClickCommand::SetStatus { success, location } => {
                if let Some(location) = location {
                    self.location =
                        location - Point::new(Points::from_f32(24.), Points::from_f32(48.));
                    self.last_click = Some(Instant::now());
                    self.clicks.send(ImageCommand::SetAlpha(1.)).await?;
                    if success {
                        self.clicks
                            .send(ImageCommand::SetTag(Some("Yes".to_string())))
                            .await?;
                    } else {
                        self.clicks
                            .send(ImageCommand::SetTag(Some("No".to_string())))
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
