use kludgine::prelude::*;
use once_cell::sync::OnceCell;
use rodio::{
    decoder::Decoder,
    source::{Buffered, Source},
};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct Animation {
    pub sprite: Sprite,
}

impl Animation {
    pub async fn all() -> &'static Vec<Animation> {
        static ANIMATIONS: OnceCell<Vec<Animation>> = OnceCell::new();
        if let Some(animations) = ANIMATIONS.get() {
            return animations;
        }

        let small_planet = include_aseprite_sprite!("../assets/whitevault/space/SmallPlanet")
            .await
            .unwrap();
        let small_blue_planet =
            include_aseprite_sprite!("../assets/whitevault/space/SmallPlanet-Blue")
                .await
                .unwrap();
        let animations = vec![
            Animation {
                sprite: small_planet,
            },
            Animation {
                sprite: small_blue_planet,
            },
        ];
        ANIMATIONS.set(animations).unwrap();
        ANIMATIONS.get().unwrap()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoopKind {
    PADs,
    ARPs,
    Leads,
    Drums,
    Shakers,
}

#[derive(Clone)]
pub struct Loop {
    pub kind: LoopKind,
    pub beats: Vec<f32>,
    pub source: Buffered<Decoder<Cursor<&'static [u8]>>>,
}

impl Loop {
    fn create_source(bytes: &'static [u8]) -> Buffered<Decoder<Cursor<&'static [u8]>>> {
        let source = rodio::Decoder::new(Cursor::new(bytes)).unwrap();
        source.buffered()
    }

    pub fn all() -> &'static Vec<Loop> {
        static LOOPS: OnceCell<Vec<Loop>> = OnceCell::new();
        LOOPS.get_or_init(|| {
            vec![
                Loop {
                    kind: LoopKind::PADs,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/01-PADS.mp3"
                    )),
                },
                Loop {
                    kind: LoopKind::ARPs,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/02-ARPs.mp3"
                    )),
                },
                Loop {
                    kind: LoopKind::Leads,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/03-Lead-A.mp3"
                    )),
                },
                Loop {
                    kind: LoopKind::Leads,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/03-Lead-B.mp3"
                    )),
                },
                Loop {
                    kind: LoopKind::Leads,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/03-Lead-C.mp3"
                    )),
                },
                Loop {
                    kind: LoopKind::Drums,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/04-Drums.mp3"
                    )),
                },
                Loop {
                    kind: LoopKind::Shakers,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/05-Shakers.mp3"
                    )),
                },
            ]
        })
    }
}
