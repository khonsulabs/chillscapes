use crate::beats_per_second;
use kludgine::prelude::*;
use rodio::{decoder::Decoder, source::Buffered};
use std::{
    collections::VecDeque,
    io::Cursor,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub enum ElementCommand {
    SetBeat { beat: f32, measure: usize },
}

#[derive(Debug, Clone)]
pub enum ElementMessage {
    ImageEvent(ControlEvent),
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
    beats_to_hit: VecDeque<Instant>,
    progress: f32,
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
            progress: 0.,
            current_beat: None,
            beats_to_hit: VecDeque::default(),
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

    async fn increment_progress(&mut self, context: &mut Context, factor: f32) {
        // Add to progress so that 2 measures of perfect hits = 1.0
        self.progress += 1. / (self.beats.len() as f32 * 2.) * factor;

        println!(
            "Something happened {}, new progress: {}",
            factor, self.progress
        );
        if self.progress < 0. {
            self.progress = 0.;
        } else if self.progress >= 1. {
            self.progress = 1.;
            self.callback(context, ElementEvent::LoopLockedIn).await;
        }
    }

    async fn deduct_missed_beats(&mut self, context: &mut Context) {
        let now = Instant::now();

        if let Some(beat_instant) = self.beats_to_hit.front() {
            let delta = instant_delta_in_millis(*beat_instant, now);

            if delta < -50 {
                self.increment_progress(context, -1.).await;
                self.beats_to_hit.pop_front();
            }
        }
    }
}

#[async_trait]
impl Component for Element {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        let sprite = include_aseprite_sprite!("../assets/ecton/star").await?;
        self.star = self
            .new_entity(context, Image::new(sprite))
            .callback(ElementMessage::ImageEvent)
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

    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        self.deduct_missed_beats(context).await;
        self.alpha_animator.update().await;
        self.frame_animator.update().await;
        Ok(())
    }
}

fn instant_delta_in_millis(a: Instant, b: Instant) -> i128 {
    if let Some(delta) = a.checked_duration_since(b) {
        delta.as_millis() as i128
    } else {
        -(b.checked_duration_since(a).unwrap().as_millis() as i128)
    }
}

const INITIAL_ALPHA: f32 = 0.1;

#[async_trait]
impl InteractiveComponent for Element {
    type Message = ElementMessage;
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

                        self.beats_to_hit.push_back(next_beat_instant);

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
                            self.star
                                .animate()
                                .alpha(self.progress * 0.7 + 0.3, LinearTransition),
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

    async fn receive_message(
        &mut self,
        context: &mut Context,
        message: Self::Message,
    ) -> KludgineResult<()> {
        match message {
            ElementMessage::ImageEvent(ControlEvent::Clicked(_)) => {
                let now = Instant::now();

                if let Some(beat_instant) = self.beats_to_hit.pop_front() {
                    let delta = instant_delta_in_millis(beat_instant, now);
                    println!("Click delta: {}", delta);
                    match delta {
                        i128::MIN..=-51 | 51..=150 => {
                            // Missed the beat entirely or clicked a bit too soon
                            self.increment_progress(context, -1.).await;
                        }
                        -50..=50 => {
                            self.increment_progress(context, 1.).await;
                        }
                        151..=i128::MAX => {
                            // Far in the future, the click should count against the player
                            // but the beat should still be clickable.
                            self.increment_progress(context, -1.).await;
                            self.beats_to_hit.push_front(beat_instant);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
