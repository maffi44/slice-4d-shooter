// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;

use fyrox_core::{
    algebra::Vector3,
    pool::Handle
};
use fyrox_resource::Resource;
use fyrox_sound::{
    buffer::{
        DataSource,
        SoundBuffer,
        SoundBufferResource,
        SoundBufferResourceExtension
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


#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Sound {
    MachinegunShot,
    ShotgunShot,
    ShotgunShotImpact,
    HolegunShot,
    HolegunCharging,
    RotatingAroundW,
    PlayerExplosion,
    PlayerHitSignal,
    PlayerDeathSignal,
    ScannerSound,
    PlayerHited,
    PlayerDied,
    ShiftingAlongW,
    PlayerRespawned,
    TeamWin,
    TeamLoose,
    GetScore,
    LooseScore,
    WShiftStart,
    FlagOnTheBase,
    FlagCuptured,
    NewProjecion,
    SwitchWeapon,
    PlayerGetScanned,
    ChargingWJump,
    WJump,
    ProjectionCaptured,
}
pub struct AudioSystem {
    pub sound_engine: Option<SoundEngine>,
    sounds: HashMap<Sound, Resource<SoundBuffer>>,
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
        sound: Sound,
        gain: f32,
        pitch: f64,
        looping: bool,
        is_play_once: bool,
        status: Status,
    ) -> Option<Handle<SoundSource>> {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            let sound_buffer = self.sounds
                .get(&sound)
                .expect("Some sound is not exist");
    
            let source = SoundSourceBuilder::new()
                .with_buffer(sound_buffer.clone())
                .with_status(status)
                .with_gain(gain*self.master_volume)
                .with_play_once(is_play_once)
                .with_pitch(pitch)
                .with_looping(looping)
                .build()
                .unwrap();
    
            let engine_state = sound_engine.state();
    
            // getting not spatial sounds context
            let mut context_state = engine_state.contexts()[0].state();
    
            let handle = context_state.add_source(source);
    
            Some(handle)
        }
        else
        {
            None    
        }

    }

    pub fn spawn_spatial_sound(
        &mut self,
        sound: Sound,
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
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            let sound_buffer = self.sounds
                .get(&sound)
                .expect("Some sound is not exist");
    
            let position = Vector3::<f32>::new(
                position.x,
                position.y + position.w,
                position.z,
            );
    
            let source = SoundSourceBuilder::new()
                .with_buffer(sound_buffer.clone())
                .with_status(status)
                .with_gain(gain*self.master_volume)
                .with_play_once(is_play_once)
                .with_pitch(pitch)
                .with_looping(looping)
                .with_position(position)
                .with_radius(radius)
                .with_rolloff_factor(rolloff_factor)
                .with_max_distance(max_distance)
                .build()
                .unwrap();
    
            let engine_state = sound_engine.state();
    
            // getting spatial sounds context
            let mut context_state = engine_state.contexts()[1].state();
    
            let handle = context_state.add_source(source);
    
            Some(handle)
        }
        else
        {
            None    
        }

    }


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


    pub fn stop_sound(&mut self, handle: Option<Handle<SoundSource>>, pause: bool) {
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

    pub fn sound_set_position(&mut self, handle: Option<Handle<SoundSource>>, postion: Vec4) {
        if let Some(sound_engine) = self.sound_engine.as_mut()
        {
            if let Some(handle) = handle
            {
                let position = Vector3::<f32>::new(
                    postion.x,
                    postion.y + postion.w,
                    postion.z,
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


    pub async fn new(headless: bool) -> Self {
        
        let sound_engine = if headless
        {
            None
        }
        else
        {
            match SoundEngine::new()
            {
                Ok(sound_engine) =>
                {
                    let not_spatial_context = SoundContext::new();
                    not_spatial_context.state().set_distance_model(DistanceModel::None);
                    
                    let spatial_context = SoundContext::new();
                    spatial_context.state().set_distance_model(DistanceModel::LinearDistance);
                    
                    // index 0 is for not spatial sounds and 1 for spatial sounds
                    sound_engine.state().add_context(not_spatial_context);
                    sound_engine.state().add_context(spatial_context);
                    
                    Some(sound_engine)
                }
                Err(e) =>
                {
                    eprintln!("Can't initialize sound engine with error: {}, game will be initialized without sound engine", e);

                    None
                }
            }
        };

        let mut sounds = HashMap::new();

        let machinegun_shot_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/machinegun_shot.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let shotgun_shot_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/shotgun_shot.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let shotgun_shot_impact_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/shotgun_shot_impact.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let holegun_shot_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/holegun_shot.wav").into()
            )
        ).expect("can't create sound buffer resourse");

        let holegun_charging_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/holegun_charging.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let rotating_around_w_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/rotating_around_w.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let player_explosion = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/player_explosion.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let player_hit_signal = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/player_hit_signal.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let player_death_signal = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/player_death_signal.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let scanner_sound = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/scanner_sound.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let player_hited = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/player_hited.wav").into(),
            )
        ).expect("can't create sound buffer resourse");
        
        let player_died = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/player_died.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let shifting_along_w = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/shifting_along_w.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let player_respawned = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/player_respawned.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let team_win = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/team_win.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let team_loose = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/team_lost.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let get_score = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/get_score.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let loose_score = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/lost_score.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let flag_captured = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/flag_captured.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let flag_on_the_base = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/flag_returned.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let w_shift_start = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/move_to_another_w_level.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let new_projection = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/new_projection.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let switch_weapon = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/switch_weapon.wav").into(),
            )
        ).expect("can't create sound buffer resourse");
        
        let player_get_scanned = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/player_get_scanned.wav").into(),
            )
        ).expect("can't create sound buffer resourse");
        
        let charging_w_jump = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/charging_w_jump.wav").into(),
            )
        ).expect("can't create sound buffer resourse");
        
        let w_jump = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/w_jump.wav").into(),
            )
        ).expect("can't create sound buffer resourse");

        let projection_captured = SoundBufferResource::new_generic(
            DataSource::from_memory(
                include_bytes!("../assets/sounds/projection_captured.wav").into(),
            )
        ).expect("can't create sound buffer resourse");


        sounds.insert(Sound::MachinegunShot, machinegun_shot_sound_resource);
        sounds.insert(Sound::ShotgunShot, shotgun_shot_sound_resource);
        sounds.insert(Sound::ShotgunShotImpact, shotgun_shot_impact_sound_resource);
        sounds.insert(Sound::HolegunShot, holegun_shot_sound_resource);
        sounds.insert(Sound::HolegunCharging, holegun_charging_sound_resource);
        sounds.insert(Sound::RotatingAroundW, rotating_around_w_sound_resource);
        sounds.insert(Sound::PlayerExplosion, player_explosion);
        sounds.insert(Sound::PlayerHitSignal, player_hit_signal);
        sounds.insert(Sound::PlayerDeathSignal, player_death_signal);
        sounds.insert(Sound::ScannerSound, scanner_sound);
        sounds.insert(Sound::PlayerHited, player_hited);
        sounds.insert(Sound::PlayerDied, player_died);
        sounds.insert(Sound::ShiftingAlongW, shifting_along_w);
        sounds.insert(Sound::PlayerRespawned, player_respawned);
        sounds.insert(Sound::TeamWin, team_win);
        sounds.insert(Sound::TeamLoose, team_loose);
        sounds.insert(Sound::GetScore, get_score);
        sounds.insert(Sound::LooseScore, loose_score);
        sounds.insert(Sound::FlagCuptured, flag_captured);
        sounds.insert(Sound::FlagOnTheBase, flag_on_the_base);
        sounds.insert(Sound::WShiftStart, w_shift_start);
        sounds.insert(Sound::NewProjecion, new_projection);
        sounds.insert(Sound::SwitchWeapon, switch_weapon);
        sounds.insert(Sound::PlayerGetScanned, player_get_scanned);
        sounds.insert(Sound::ChargingWJump, charging_w_jump);
        sounds.insert(Sound::WJump, w_jump);
        sounds.insert(Sound::ProjectionCaptured, projection_captured);

        AudioSystem {
            sound_engine,
            sounds,
            master_volume: 0.7,
        }
    }
}