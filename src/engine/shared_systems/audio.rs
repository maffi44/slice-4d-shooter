// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use fyrox_core::{
    algebra::Vector3,
    pool::Handle
};
use fyrox_resource::Resource;
use fyrox_sound::{
    buffer::{
        SoundBuffer
    },
    context::{
        DistanceModel,
        SoundContext
    },
    engine::SoundEngine,
    source::{
        SoundSource,
        SoundSourceBuilder,
        Status
    }
};

use glam::Vec4;


pub struct AudioSystem {
    pub sound_engine: Option<SoundEngine>,
    pending_buffers: Vec<(usize, Handle<SoundSource>, Resource<SoundBuffer>)>,
    pub master_volume: f32,
}

impl AudioSystem {

    pub fn increase_sound_volume(&mut self, delta: f32)
    {
        self.master_volume = (self.master_volume + delta*0.9).clamp(0.0, 1.0);
    }

    pub fn decrease_sound_volume(&mut self, delta: f32)
    {
        self.master_volume = (self.master_volume - delta*0.9).clamp(0.0, 1.0);
    }

    pub fn set_listener_position_and_look_vector(&mut self, position: Vec4, look: Vec4) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            let st = sound_engine.state();
            let mut state = st.contexts()[1].state();
    
            let position = Vector3::<f32>::new(
                position.x,
                position.y + position.w,
                position.z,
            );
    
            let look = Vector3::<f32>::new(
                look.x,
                look.y,
                look.z
            ).normalize();
    
            state.listener_mut().set_orientation_rh(
                look,
                Vector3::<f32>::new(0.0, 1.0, 0.0)
            );
            state.listener_mut().set_position(position);
        }
    }


    pub fn spawn_non_spatial_sound(
        &mut self,
        sound_buffer: &Option<Resource<SoundBuffer>>,
        gain: f32,
        pitch: f64,
        looping: bool,
        is_play_once: bool,
        status: Status,
    ) -> Option<Handle<SoundSource>> {
        let master_volume = self.master_volume;
        self.spawn_source(0, sound_buffer.as_ref()?, || {
            SoundSourceBuilder::new()
                .with_status(status)
                .with_gain(gain * master_volume)
                .with_play_once(is_play_once)
                .with_pitch(pitch)
                .with_looping(looping)
        })
    }

    pub fn spawn_spatial_sound(
        &mut self,
        sound_buffer: &Option<Resource<SoundBuffer>>,
        gain: f32,
        pitch: f64,
        looping: bool,
        is_play_once: bool,
        status: Status,
        position: Vec4,
        radius: f32,
        rolloff_factor: f32,
        max_distance: f32,
    ) -> Option<Handle<SoundSource>> {
        let master_volume = self.master_volume;
        self.spawn_source(1, sound_buffer.as_ref()?, || {
            let position = Vector3::<f32>::new(position.x, position.y + position.w, position.z);
            SoundSourceBuilder::new()
                .with_status(status)
                .with_gain(gain * master_volume)
                .with_play_once(is_play_once)
                .with_pitch(pitch)
                .with_looping(looping)
                .with_position(position)
                .with_radius(radius)
                .with_rolloff_factor(rolloff_factor)
                .with_max_distance(max_distance)
        })
    }

    /// `make_builder` runs only when playback is possible: without a sound engine (headless, or the
    /// device failed to open) the call returns `None` before a builder is constructed, because
    /// `SoundSourceBuilder::new` allocates (issues/programming/headless-sound-builder-allocation/).
    fn spawn_source(
        &mut self,
        context: usize,
        buffer: &Resource<SoundBuffer>,
        make_builder: impl FnOnce() -> SoundSourceBuilder,
    ) -> Option<Handle<SoundSource>> {
        let engine = self.sound_engine.as_ref()?;
        let pending = buffer.is_loading();
        let source = make_builder()
            .with_opt_buffer((!pending).then(|| buffer.clone()))
            .build()
            .ok()?;
        let handle = engine.state().contexts()[context]
            .state()
            .add_source(source);
        if pending {
            // A dynamically spawned actor can play before its first decode finishes.
            // The silent source already accepts position, gain and playback commands.
            self.pending_buffers.push((context, handle, buffer.clone()));
        }
        Some(handle)
    }

    pub fn update(&mut self) {
        if self.pending_buffers.is_empty() {
            return;
        }
        let Some(engine) = &self.sound_engine else {
            return;
        };
        let engine = engine.state();
        self.pending_buffers.retain(|(index, handle, buffer)| {
            if buffer.is_loading() {
                return true;
            }
            let mut context = engine.contexts()[*index].state();
            if let Ok(source) = context.try_get_source_mut(*handle) {
                if let Err(error) = source.set_buffer(Some(buffer.clone())) {
                    log::error!("Cannot attach sound buffer: {error:?}");
                    context.remove_source(*handle);
                }
            }
            false
        });
    }

    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn pause_sound(&mut self, handle: Option<Handle<SoundSource>>) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[0].state();
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.pause();
        
                } else {
                    let mut state = st.contexts()[1].state();
                    if state.is_valid_handle(handle) {
                        let sound = state.source_mut(handle);
        
                        sound.pause();
                    }
                }
            }
        }
    }


    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn stop_sound(&mut self, handle: Option<Handle<SoundSource>>, _pause: bool) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[0].state();
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    let _ = sound.stop();
        
                } else {
                    let mut state = st.contexts()[1].state();
                    if state.is_valid_handle(handle) {
                        let sound = state.source_mut(handle);
            
                        let _ = sound.stop();
                    }
                }
            }
        }
    }

    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn play_sound(&mut self, handle: Option<Handle<SoundSource>>) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[0].state();
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.play();
        
                } else {
                    let mut state = st.contexts()[1].state();
                    if state.is_valid_handle(handle) {
                        let sound = state.source_mut(handle);
        
                        sound.play();
                    }
                }
            }
        }
    }


    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn sound_set_gain(&mut self, handle: Option<Handle<SoundSource>>, gain: f32) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[0].state();
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.set_gain(gain*self.master_volume);
        
                } else {
                    let mut state = st.contexts()[1].state();
                    if state.is_valid_handle(handle) {
                        let sound = state.source_mut(handle);
        
                        sound.set_gain(gain*self.master_volume);
                    }
                }
            }
        }
    }


    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn sound_set_pitch(&mut self, handle: Option<Handle<SoundSource>>, pitch: f64) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[0].state();
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.set_pitch(pitch);
        
                } else {
                    let mut state = st.contexts()[1].state();
                    if state.is_valid_handle(handle) {
                        let sound = state.source_mut(handle);
        
                        sound.set_pitch(pitch);
                    }
                }
            }
        }
    }

    pub fn sound_set_pitch_and_gain(&mut self, handle: Option<Handle<SoundSource>>, pitch: f64, gain: f32) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[0].state();
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.set_pitch(pitch);
                    sound.set_gain(gain*self.master_volume);
        
                } else {
                    let mut state = st.contexts()[1].state();
                    if state.is_valid_handle(handle) {
                        let sound = state.source_mut(handle);
        
                        sound.set_pitch(pitch);
                        sound.set_gain(gain*self.master_volume);
                    }
                }
            }
        }
    }


    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn sound_set_looping(&mut self, handle: Option<Handle<SoundSource>>, looping: bool) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[0].state();
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.set_looping(looping);
        
                } else {
                    let mut state = st.contexts()[1].state();
                    if state.is_valid_handle(handle) {
                        let sound = state.source_mut(handle);
            
                        sound.set_looping(looping);
                    }
                }
            }
        }
    }

    pub fn sound_set_position(&mut self, handle: Option<Handle<SoundSource>>, position: Vec4) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let position = Vector3::<f32>::new(
                    position.x,
                    position.y + position.w,
                    position.z,
                );
                
                let st = sound_engine.state();
                let mut state = st.contexts()[1].state();
        
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.set_position(position);
                }
            }
        }
    }

    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn sound_set_radius(&mut self, handle: Option<Handle<SoundSource>>, radius: f32) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[1].state();
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.set_radius(radius);
                }
            }
        }
    }

    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn sound_set_rolloff_factor(&mut self, handle: Option<Handle<SoundSource>>, rolloff_factor: f32) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                let mut state = st.contexts()[1].state();
        
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.set_rolloff_factor(rolloff_factor);
                }
            }
        }
    }

    #[allow(dead_code)] // sound control API, not used by the game yet
    pub fn sound_set_max_distance(&mut self, handle: Option<Handle<SoundSource>>, max_distance: f32) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                let mut state = st.contexts()[1].state();
        
                if state.is_valid_handle(handle) {
                    let sound = state.source_mut(handle);
        
                    sound.set_max_distance(max_distance);
                }
            }
        }
    }


    pub fn remove_sound(&mut self, handle: Option<Handle<SoundSource>>) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let st = sound_engine.state();
                
                let mut state = st.contexts()[0].state();
                if state.is_valid_handle(handle) {
                    state.remove_source(handle);
        
                } else {
                    let mut state = st.contexts()[1].state();
                    if state.is_valid_handle(handle) {
                        state.remove_source(handle);
                    }
                }
            }
        }
    }


    /// Opens the sound device (the `SoundEngine` with its two contexts). If it fails, the game runs
    /// without sound.
    fn create_sound_engine() -> Option<SoundEngine> {
        match SoundEngine::new()
        {
            Ok(sound_engine) =>
            {
                let not_spatial_context = SoundContext::new();
                not_spatial_context.state().set_distance_model(DistanceModel::None);
                
                let spatial_context = SoundContext::new();
                spatial_context.state().set_distance_model(DistanceModel::LinearDistance);
                
                // index 0 is for non-spatial sounds, 1 for spatial sounds
                sound_engine.state().add_context(not_spatial_context);
                sound_engine.state().add_context(spatial_context);
                
                Some(sound_engine)
            }
            Err(e) =>
            {
                eprintln!("can't initialize the sound engine: {}; the game will run without sound", e);

                None
            }
        }
    }


    /// `headless`: no sound device (every play/stop call is a no-op).
    pub fn new(headless: bool) -> Self {
        
        AudioSystem {
            sound_engine: if headless {None} else {Self::create_sound_engine()},
            pending_buffers: Vec::new(),
            master_volume: 0.7,
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::engine::shared_systems::resources::{ResourceSystem, Retention};

    #[test]
    fn unavailable_playback_returns_before_building_a_source()
    {
        let runtime = tokio::runtime::Builder::new_multi_thread().worker_threads(1).build().unwrap();
        let resources = ResourceSystem::new(runtime.handle().clone(), Some(tokio::sync::watch::channel(None).1));
        let buffer = resources.loader()
            .load_media(crate::embedded_asset!("sounds/player_explosion.wav"), Retention::WhileUsed)
            .expect("a sound handle when sound is enabled");
        let mut audio = AudioSystem::new(true);

        let mut built = false;
        assert!(audio.spawn_source(0, &buffer, || { built = true; SoundSourceBuilder::new() }).is_none());
        assert!(!built, "no builder without a sound engine");

        // An absent buffer returns before `spawn_source` and before any spatial computation.
        assert!(audio.spawn_non_spatial_sound(&None, 1.0, 1.0, false, true, Status::Playing).is_none());
        assert!(audio.spawn_spatial_sound(&None, 1.0, 1.0, false, true, Status::Playing, Vec4::ZERO, 1.0, 1.0, 1.0).is_none());
        assert!(audio.spawn_spatial_sound(&Some(buffer.clone()), 1.0, 1.0, false, true, Status::Playing, Vec4::ZERO, 1.0, 1.0, 1.0).is_none());
        assert!(audio.pending_buffers.is_empty());
    }
}
