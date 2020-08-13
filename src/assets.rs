use kludgine::prelude::*;
use once_cell::sync::OnceCell;
use rodio::{
    decoder::Decoder,
    source::{Buffered, Source},
};
use std::io::Cursor;

#[derive(Clone, Debug)]
pub struct Animation {
    pub id: usize,
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
        let crater = include_aseprite_sprite!("../assets/whitevault/space/Crater_1")
            .await
            .unwrap();
        let planet2 = include_aseprite_sprite!("../assets/whitevault/space/Planet2")
            .await
            .unwrap();
        let planet3 = include_aseprite_sprite!("../assets/whitevault/space/Planet3")
            .await
            .unwrap();
        let planet4 = include_aseprite_sprite!("../assets/whitevault/space/Planet4")
            .await
            .unwrap();
        let space_case = include_aseprite_sprite!("../assets/whitevault/space/Space_case")
            .await
            .unwrap();
        let animations = vec![
            Animation {
                sprite: small_planet,
                id: 0,
            },
            Animation {
                sprite: small_blue_planet,
                id: 1,
            },
            Animation {
                sprite: crater,
                id: 2,
            },
            Animation {
                sprite: planet2,
                id: 3,
            },
            Animation {
                sprite: planet3,
                id: 4,
            },
            Animation {
                sprite: planet4,
                id: 5,
            },
            Animation {
                sprite: space_case,
                id: 6,
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
    Bass,
    Piano,
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
                    source: Self::create_source(include_bytes!("../assets/pxzel/space/Pads1.ogg")),
                },
                Loop {
                    kind: LoopKind::PADs,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!("../assets/pxzel/space/Pads2.ogg")),
                },
                Loop {
                    kind: LoopKind::PADs,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Pads2_complex.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::ARPs,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!("../assets/pxzel/space/Arp_es.ogg")),
                },
                Loop {
                    kind: LoopKind::ARPs,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Arp_fths.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::ARPs,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Arp_u_p.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Leads,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/lead1_simple.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Leads,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/lead1_complex.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Leads,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/lead1_med.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Leads,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/lead2_med.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Leads,
                    beats: Vec::default(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/lead2_simple.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Drums,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Drums_perc.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Drums,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Drums_ks.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Bass,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Bass_ft.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Bass,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Bass_oct.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Bass,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Bass_qt.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Bass,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!(
                        "../assets/pxzel/space/Bass_sus.ogg"
                    )),
                },
                Loop {
                    kind: LoopKind::Piano,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!("../assets/pxzel/space/Piano1.ogg")),
                },
                Loop {
                    kind: LoopKind::Piano,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!("../assets/pxzel/space/Piano2.ogg")),
                },
                Loop {
                    kind: LoopKind::Piano,
                    beats: (0..32).map(|beat| beat as f32).collect(),
                    source: Self::create_source(include_bytes!("../assets/pxzel/space/Piano3.ogg")),
                },
            ]
        })
    }
}
