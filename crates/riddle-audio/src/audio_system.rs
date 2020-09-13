use crate::*;

use rodio::{Device, Sink};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
    time::{Duration, Instant},
};

pub struct AudioSystem {
    weak_self: Weak<AudioSystem>,
    pub(super) device: Device,

    fades: RefCell<std::collections::HashMap<FadeKey, Fade>>,
}

impl AudioSystem {
    pub fn new() -> Result<Rc<AudioSystem>, AudioError> {
        let device = rodio::default_output_device().ok_or(AudioError::UnknownError)?;
        Ok(Rc::new_cyclic(|weak_self| AudioSystem {
            weak_self: weak_self.clone(),
            device,
            fades: RefCell::new(HashMap::new()),
        }))
    }

    pub fn process_frame(&self) {
        let now = Instant::now();
        self.tick_fades(now);
    }

    pub(crate) fn register_fade(&self, fade: Fade) {
        let mut fades = self.fades.borrow_mut();
        let existing = fades.remove(&fade.key());
        match existing {
            Some(old) => fades.insert(fade.key(), Fade::merge_pair(old, fade)),
            None => fades.insert(fade.key(), fade),
        };
    }

    pub fn tick_fades(&self, now: Instant) {
        let mut fades = self.fades.borrow_mut();
        fades.retain(|_, f| f.update(now));
    }
}

struct FadeKey {
    sink: Rc<Sink>,
}

impl std::hash::Hash for FadeKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::ptr::hash(&*self.sink, state);
    }
}

impl std::cmp::PartialEq for FadeKey {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.sink, &other.sink)
    }
}

impl std::cmp::Eq for FadeKey {}

pub(crate) enum FadeType {
    Pause,
    Resume,
    AlterVolume,
}

pub(crate) struct Fade {
    sink: Rc<Sink>,
    start_volume: f32,
    dest_volume: f32,
    start_time: Instant,
    duration: Duration,
    fade_type: FadeType,
}

impl Fade {
    pub(crate) fn new(
        sink: Rc<Sink>,
        dest_volume: f32,
        duration: Duration,
        fade_type: FadeType,
    ) -> Self {
        let start_volume = sink.volume();
        let start_time = Instant::now();
        Self {
            sink,
            start_volume,
            dest_volume,
            start_time,
            duration,
            fade_type,
        }
    }

    fn merge_pair(old: Self, new: Self) -> Self {
        use FadeType::*;
        match (&old.fade_type, &new.fade_type) {
            (AlterVolume, _) => new,
            (Pause, _) => old,
            (Resume, _) => old,
        }
    }

    fn update(&self, now: Instant) -> bool {
        let current_duration = now.duration_since(self.start_time);
        let t = current_duration.as_secs_f32() / self.duration.as_secs_f32();
        let new_volume = self.start_volume + ((self.dest_volume - self.start_volume) * t.min(1.0));
        self.sink.set_volume(new_volume);

        let finished = t >= 1.0;
        if finished {
            match &self.fade_type {
                FadeType::Pause => {
                    self.sink.pause();
                }
                _ => (),
            }
        }

        !finished
    }

    fn key(&self) -> FadeKey {
        FadeKey {
            sink: self.sink.clone(),
        }
    }
}

impl CloneHandle for AudioSystem {
    type Handle = Rc<Self>;
    type WeakHandle = Weak<Self>;

    #[inline]
    fn clone_handle(&self) -> Option<Rc<Self>> {
        std::rc::Weak::upgrade(&self.clone_weak_handle())
    }

    fn clone_weak_handle(&self) -> Weak<Self> {
        self.weak_self.clone()
    }
}
