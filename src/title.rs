use kludgine::prelude::*;

#[derive(Default)]
pub struct TitleScreen {
    logo: Entity<Label>,
    start_button: Entity<Button>,
    music_by: Entity<Label>,
    art_by: Entity<Label>,
    code_by: Entity<Label>,
}

#[derive(Clone, Debug)]
pub enum TitleScreenEvent {
    StartGame,
}

#[derive(Clone, Debug)]
pub enum Message {
    MusicByClicked,
    ArtByClicked,
    CodeByClicked,
    StartClicked,
}

#[async_trait]
impl Component for TitleScreen {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        self.logo = self
            .new_entity(context, Label::new("Chillscapes"))
            .style(Style {
                font_size: Some(60.),
                ..Default::default()
            })
            .insert()
            .await?;

        self.music_by = self
            .new_entity(context, Label::new("Music by \nPxzel"))
            .callback(|_| Message::MusicByClicked)
            .hover(Style {
                color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
                ..Default::default()
            })
            .insert()
            .await?;

        self.art_by = self
            .new_entity(context, Label::new("Art by \nWhiteVault"))
            .callback(|_| Message::ArtByClicked)
            .hover(Style {
                color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
                ..Default::default()
            })
            .insert()
            .await?;

        self.code_by = self
            .new_entity(context, Label::new("Code by \nKhonsu Labs"))
            .callback(|_| Message::CodeByClicked)
            .hover(Style {
                color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
                ..Default::default()
            })
            .insert()
            .await?;

        self.start_button = self
            .new_entity(context, Button::new("Start"))
            .callback(|_| Message::StartClicked)
            .style(Style {
                color: Some(Color::new(1.0, 1.0, 1.0, 1.0)),
                background_color: Some(Color::new(1.0, 0.0, 0.9, 1.0)),
                ..Default::default()
            })
            .insert()
            .await?;

        Ok(())
    }

    async fn layout(
        &mut self,
        context: &mut StyledContext,
    ) -> KludgineResult<Box<dyn LayoutSolver>> {
        let window_size = context.scene().size().await.to_f32();

        Layout::absolute()
            .child(
                self.logo,
                AbsoluteBounds {
                    top: Dimension::from_points(window_size.height / 3.),
                    ..Default::default()
                },
            )?
            .child(
                self.start_button,
                AbsoluteBounds {
                    top: Dimension::from_points(window_size.height / 3. * 2.),
                    ..Default::default()
                },
            )?
            .child(
                self.code_by,
                AbsoluteBounds {
                    left: Dimension::from_points(16.),
                    bottom: Dimension::from_points(16.),
                    ..Default::default()
                },
            )?
            .child(
                self.music_by,
                AbsoluteBounds {
                    right: Dimension::from_points(16.),
                    bottom: Dimension::from_points(16.),
                    ..Default::default()
                },
            )?
            .child(
                self.art_by,
                AbsoluteBounds {
                    bottom: Dimension::from_points(16.),
                    ..Default::default()
                },
            )?
            .layout()
    }
}

#[async_trait]
impl InteractiveComponent for TitleScreen {
    type Message = Message;
    type Input = ();
    type Output = TitleScreenEvent;

    async fn receive_message(
        &mut self,
        context: &mut Context,
        message: Self::Message,
    ) -> KludgineResult<()> {
        match message {
            Message::ArtByClicked => {
                let _ = webbrowser::open("https://whitevault.tv/");
            }
            Message::CodeByClicked => {
                let _ = webbrowser::open("https://community.khonsulabs.com/");
            }
            Message::MusicByClicked => {
                let _ = webbrowser::open("http://pxzel.com/");
            }
            Message::StartClicked => {
                self.callback(context, TitleScreenEvent::StartGame).await;
            }
        }
        Ok(())
    }
}
