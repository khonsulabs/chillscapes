use crate::{
    assets::{Animation, Loop},
    seconds_per_beat,
};
use kludgine::prelude::*;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub enum ElementCommand {
    SetBeat {
        is_new_measure: bool,
        beat: f32,
        measure: usize,
    },
    SetVolume(f32),
}

#[derive(Debug, Clone)]
pub enum ElementMessage {
    ImageEvent(ControlEvent),
}

#[derive(Debug, Clone)]
pub enum ElementEvent {
    LoopLockedIn,
    Soloing(Index),
    StoppingSolo,
    Success(Point<Points>),
    Failure(Option<Point<Points>>),
}

#[derive(Debug, Clone, Copy)]
enum ElementProgress {
    Pending(f32),
    LockedIn,
}

impl ElementProgress {
    pub fn percent(&self) -> f32 {
        match self {
            ElementProgress::Pending(value) => *value,
            ElementProgress::LockedIn => 1.,
        }
    }

    pub fn min_percent(&self) -> f32 {
        match self {
            ElementProgress::Pending(value) => value / 3. * 0.9 + 0.1,
            ElementProgress::LockedIn => 1.,
        }
    }
}

pub struct Element {
    animation: &'static Animation,
    beats_per_loop: usize,
    tempo: f32,
    volume: f32,
    audio_loop: &'static Loop,
    image: Entity<Image>,
    measure: Option<usize>,
    current_beat: Option<usize>,
    beats_to_hit: VecDeque<Instant>,
    progress: ElementProgress,
    alpha_animator: RequiresInitialization<AnimationManager<ImageAlphaAnimation>>,
    frame_animator: RequiresInitialization<AnimationManager<ImageFrameAnimation>>,
    playing_audio: Option<rodio::Sink>,
}

impl Element {
    pub fn new(
        beats_per_loop: usize,
        tempo: f32,
        volume: f32,
        animation: &'static Animation,
        audio_loop: &'static Loop,
    ) -> Self {
        Self {
            animation,
            beats_per_loop,
            tempo,
            audio_loop,
            measure: None,
            progress: ElementProgress::Pending(0.),
            current_beat: None,
            beats_to_hit: VecDeque::default(),
            image: Entity::default(),
            alpha_animator: Default::default(),
            frame_animator: Default::default(),
            playing_audio: None,
            volume,
        }
    }

    fn next_beat_start(&self, mut current_beat: f32) -> (f32, f32) {
        let beat_start = match self.current_beat {
            Some(index) => {
                if let Some(beat) = self.audio_loop.beats.get(index + 1) {
                    *beat
                } else {
                    current_beat -= self.beats_per_loop as f32;
                    self.audio_loop.beats[0]
                }
            }
            None => self.audio_loop.beats[0],
        };

        (beat_start, current_beat)
    }

    async fn increment_progress(&mut self, context: &mut Context, factor: f32) {
        if let ElementProgress::Pending(current_progress) = self.progress {
            // Add to progress so that 2 measures of perfect hits = 1.0

            let mut progress =
                current_progress + 1. / (self.audio_loop.beats.len() as f32 / 2.) * factor;

            if progress < 0. {
                progress = 0.;
            } else if progress >= 1. {
                self.progress = ElementProgress::LockedIn;
                self.callback(context, ElementEvent::LoopLockedIn).await;
                return;
            }
            self.progress = ElementProgress::Pending(progress);
        }
    }

    async fn deduct_missed_beats(&mut self, context: &mut Context) {
        if self.progress.percent() < 1. {
            let now = Instant::now();

            if let Some(beat_instant) = self.beats_to_hit.front() {
                let delta = instant_delta_in_millis(*beat_instant, now);

                if delta < -100 {
                    self.increment_progress(context, -0.5).await;
                    self.beats_to_hit.pop_front();
                }
            }
        }
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        if let Some(playing_audio) = self.playing_audio.as_mut() {
            playing_audio.set_volume(self.volume);
        }
    }
}

#[async_trait]
impl Component for Element {
    async fn initialize(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        self.image = self
            .new_entity(context, Image::new(self.animation.sprite.clone()))
            .callback(ElementMessage::ImageEvent)
            .insert()
            .await?;

        self.alpha_animator.initialize_with(
            AnimationManager::new(
                self.image
                    .animate()
                    .alpha(self.progress.min_percent(), LinearTransition),
            )
            .await,
        );

        self.frame_animator.initialize_with(
            AnimationManager::new(self.image.animate().frame(0., LinearTransition)).await,
        );
        Ok(())
    }

    async fn update(&mut self, context: &mut SceneContext) -> KludgineResult<()> {
        self.deduct_missed_beats(context).await;
        self.alpha_animator.update().await;
        self.frame_animator.update().await;
        Ok(())
    }

    async fn hovered(&mut self, context: &mut Context) -> KludgineResult<()> {
        self.callback(context, ElementEvent::Soloing(context.index()))
            .await;
        Ok(())
    }

    async fn unhovered(&mut self, context: &mut Context) -> KludgineResult<()> {
        self.callback(context, ElementEvent::StoppingSolo).await;
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
            ElementCommand::SetBeat {
                is_new_measure,
                beat,
                measure,
            } => {
                if self.playing_audio.is_none() || is_new_measure {
                    if let Some(device) = rodio::default_output_device() {
                        let sink = rodio::Sink::new(&device);
                        sink.append(self.audio_loop.source.clone());
                        sink.set_volume(self.volume);
                        self.playing_audio = Some(sink);
                    }

                    self.current_beat = None;
                    self.measure = Some(measure);
                }

                let (next_beat_start, adjusted_beat) = self.next_beat_start(beat);

                if adjusted_beat > next_beat_start {
                    let mut new_beat_index = self.current_beat.map(|beat| beat + 1).unwrap_or(0);
                    if new_beat_index > self.audio_loop.beats.len() {
                        new_beat_index = 0;
                    }
                    self.current_beat = Some(new_beat_index);

                    let (next_beat, adjusted_beat) = self.next_beat_start(beat);
                    let remaining_beats = next_beat - adjusted_beat;
                    let remaining_seconds = seconds_per_beat(self.tempo) * remaining_beats;
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
                            self.image
                                .animate()
                                .alpha(self.progress.min_percent(), LinearTransition),
                            next_beat_start,
                        );

                        // Fade into the target alpha
                        self.alpha_animator.push_frame(
                            self.image
                                .animate()
                                .alpha(self.progress.percent() * 0.7 + 0.3, LinearTransition),
                            next_beat_instant,
                        );

                        // Fade out over 500ms
                        self.alpha_animator.push_frame(
                            self.image
                                .animate()
                                .alpha(self.progress.min_percent(), LinearTransition),
                            next_beat_instant
                                .checked_add(Duration::from_millis(500))
                                .unwrap(),
                        );

                        // Execute the animation over 1/10th of a second
                        let frame_start = next_beat_instant
                            .checked_sub(Duration::from_millis(150))
                            .unwrap();
                        self.frame_animator.push_frame(
                            self.image.animate().frame(0., LinearTransition),
                            frame_start,
                        );

                        let frame_end = next_beat_instant
                            .checked_add(Duration::from_millis(150))
                            .unwrap();
                        self.frame_animator.push_frame(
                            self.image.animate().frame(1., LinearTransition),
                            frame_end,
                        );

                        self.frame_animator.push_frame(
                            self.image.animate().frame(0., LinearTransition),
                            frame_end.checked_add(Duration::from_millis(1)).unwrap(),
                        );
                    }
                }
            }
            ElementCommand::SetVolume(volume) => {
                self.set_volume(volume);
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
            ElementMessage::ImageEvent(ControlEvent::Clicked {
                window_position, ..
            }) => {
                let now = Instant::now();

                if let Some(beat_instant) = self.beats_to_hit.pop_front() {
                    let delta = instant_delta_in_millis(beat_instant, now);
                    match delta {
                        i128::MIN..=-151 | 151..=200 => {
                            // Missed the beat entirely or clicked a bit too soon
                            self.callback(context, ElementEvent::Failure(Some(window_position)))
                                .await;
                            self.increment_progress(context, -0.5).await;
                        }
                        -150..=150 => {
                            self.callback(context, ElementEvent::Success(window_position))
                                .await;
                            self.increment_progress(context, 1.).await;
                        }
                        201..=i128::MAX => {
                            // Far in the future, the click should count against the player
                            // but the beat should still be clickable.
                            self.callback(context, ElementEvent::Failure(Some(window_position)))
                                .await;
                            self.increment_progress(context, -0.5).await;
                            self.beats_to_hit.push_front(beat_instant);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
