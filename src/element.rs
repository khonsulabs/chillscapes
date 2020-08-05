use crate::beats_per_second;
use kludgine::prelude::*;
use rodio::{decoder::Decoder, source::Buffered};
use std::{
    io::Cursor,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub enum ElementCommand {
    SetBeat { beat: f32, measure: usize },
}

#[derive(Debug, Clone)]
pub enum ElementEvent {
    LoopLockedIn,
}

pub struct Element {
    beats_per_loop: usize,
    tempo: f32,
    click: Buffered<Decoder<Cursor<Vec<u8>>>>,
    beats: Vec<f32>,
    star: Entity<Image>,
    measure: usize,
    current_beat: Option<usize>,
    alpha_animator: RequiresInitialization<AnimationManager<ImageAlphaAnimation>>,
    frame_animator: RequiresInitialization<AnimationManager<ImageFrameAnimation>>,
}

impl Element {
    pub fn new(
        beats_per_loop: usize,
        tempo: f32,
        click: Buffered<Decoder<Cursor<Vec<u8>>>>,
        beats: Vec<f32>,
    ) -> Self {
        Self {
            beats_per_loop,
            tempo,
            click,
            beats,
            measure: 0,
            current_beat: None,
            star: Entity::default(),
            alpha_animator: Default::default(),
            frame_animator: Default::default(),
        }
    }

    fn next_beat_start(&self, mut current_beat: f32) -> (f32, f32) {
        let beat_start = match self.current_beat {
            Some(index) => {
                if let Some(beat) = self.beats.get(index + 1) {
                    *beat
                } else {
                    current_beat -= self.beats_per_loop as f32;
                    self.beats[0]
                }
            }
            None => self.beats[0],
        };

        (beat_start, current_beat)
    }
}

#[async_trait]
impl Component for Element {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        let sprite = include_aseprite_sprite!("../assets/ecton/star").await?;
        self.star = self
            .new_entity(context, Image::new(sprite))
            .insert()
            .await?;

        self.alpha_animator.initialize_with(AnimationManager::new(
            self.star.animate().alpha(INITIAL_ALPHA, LinearTransition),
        ));

        self.frame_animator
            .initialize_with(AnimationManager::new(self.star.animate().frame(
                Some("Idle"),
                0.,
                LinearTransition,
            )));
        Ok(())
    }

    async fn update(&mut self, _context: &mut SceneContext) -> KludgineResult<()> {
        self.alpha_animator.update().await;
        self.frame_animator.update().await;
        Ok(())
    }
}

const INITIAL_ALPHA: f32 = 0.1;

#[async_trait]
impl InteractiveComponent for Element {
    type Message = ();
    type Input = ElementCommand;
    type Output = ElementEvent;

    async fn receive_input(
        &mut self,
        _context: &mut Context,
        command: Self::Input,
    ) -> KludgineResult<()> {
        match command {
            ElementCommand::SetBeat { beat, measure } => {
                if self.measure != measure {
                    self.current_beat = None;
                }
                self.measure = measure;

                let (next_beat_start, adjusted_beat) = self.next_beat_start(beat);

                if adjusted_beat > next_beat_start {
                    if let Some(device) = rodio::default_output_device() {
                        let sink = rodio::Sink::new(&device);
                        sink.append(self.click.clone());
                        sink.detach();
                    }

                    let mut new_beat_index = self.current_beat.map(|beat| beat + 1).unwrap_or(0);
                    if new_beat_index > self.beats.len() {
                        new_beat_index = 0;
                    }
                    self.current_beat = Some(new_beat_index);

                    let (next_beat, adjusted_beat) = self.next_beat_start(beat);
                    let remaining_beats = next_beat - adjusted_beat;
                    let remaining_seconds = beats_per_second(self.tempo) * remaining_beats;
                    if remaining_seconds > 0. {
                        let next_beat_instant = Instant::now()
                            .checked_add(Duration::from_secs_f32(remaining_seconds))
                            .unwrap();

                        // Start at 10 ms behind when the beat will hit, so that the fade-in happens over 10ms and it
                        // peaks on the beat
                        let next_beat_start = next_beat_instant
                            .checked_sub(Duration::from_millis(10))
                            .unwrap();
                        self.alpha_animator.push_frame(
                            self.star.animate().alpha(INITIAL_ALPHA, LinearTransition),
                            next_beat_start,
                        );

                        // Fade into the target alpha
                        self.alpha_animator.push_frame(
                            self.star.animate().alpha(1.0, LinearTransition),
                            next_beat_instant,
                        );

                        // Fade out over 500ms
                        self.alpha_animator.push_frame(
                            self.star.animate().alpha(INITIAL_ALPHA, LinearTransition),
                            next_beat_instant
                                .checked_add(Duration::from_millis(500))
                                .unwrap(),
                        );

                        // Execute the animation over 1/10th of a second
                        let frame_start = next_beat_instant
                            .checked_sub(Duration::from_millis(50))
                            .unwrap();
                        self.frame_animator.push_frame(
                            self.star
                                .animate()
                                .frame(Some("Normal"), 0., LinearTransition),
                            frame_start,
                        );

                        let frame_end = next_beat_instant
                            .checked_add(Duration::from_millis(50))
                            .unwrap();
                        self.frame_animator.push_frame(
                            self.star
                                .animate()
                                .frame(Some("Normal"), 1., LinearTransition),
                            next_beat_start
                                .checked_add(Duration::from_millis(50))
                                .unwrap(),
                        );

                        self.frame_animator.push_frame(
                            self.star
                                .animate()
                                .frame(Some("Idle"), 0., LinearTransition),
                            frame_end.checked_add(Duration::from_millis(1)).unwrap(),
                        );
                    }
                }
            }
        }
        Ok(())
    }
}
