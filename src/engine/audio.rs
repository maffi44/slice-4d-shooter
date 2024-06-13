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
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{
        ArrayBuffer,
        Uint8Array
    },
    Response
};
use winit::dpi::Position;


#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Sound {
    MachinegunShot,
    HolegunShot,
    HolegunCharging,
    RotatingAroundW,
    PlayerExplosion,
    PlayerHitSignal,
    PlayerDeathSignal,
}
pub struct AudioSystem {
    pub sound_engine: SoundEngine,
    sounds: HashMap<Sound, Resource<SoundBuffer>>
}

impl AudioSystem {

    pub fn set_listener_position_and_look_vector(&mut self, position: Vec4, look: Vec4) {
        let st = self.sound_engine.state();
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

    // pub fn set_listener_basis(&mut self, position: Vec4, look: Vec4) {
    //     let st = self.sound_engine.state();
    //     let mut state = st.contexts()[1].state();

    //     let position = Vector3::<f32>::new(
    //         position.x,
    //         position.y + position.w,
    //         position.z,
    //     );

    //     let look = Vector3::<f32>::new(
    //         look.x,
    //         look.y,
    //         look.z
    //     ).normalize();
        
    //     glam::Mat4::

    //     let matrix: fyrox_core::algebra::Matrix<f32, fyrox_core::algebra::Const<3>, fyrox_core::algebra::Const<3>, fyrox_core::algebra::ArrayStorage<f32, 3, 3>> =

    //     state.listener_mut().set_basis(matrix)
    // }

    pub fn spawn_non_spatial_sound(
        &mut self,
        sound: Sound,
        gain: f32,
        pitch: f64,
        looping: bool,
        is_play_once: bool,
        status: Status,
    ) -> Handle<SoundSource> {
        let sound_buffer = self.sounds
            .get(&sound)
            .expect("Some sound is not exist");

        let source = SoundSourceBuilder::new()
            .with_buffer(sound_buffer.clone())
            .with_status(status)
            .with_gain(gain)
            .with_play_once(is_play_once)
            .with_pitch(pitch)
            .with_looping(looping)
            .build()
            .unwrap();

        let engine_state = self.sound_engine.state();

        // getting not spatial sounds context
        let mut context_state = engine_state.contexts()[0].state();

        let handle = context_state.add_source(source);

        return handle;
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
    ) -> Handle<SoundSource> {
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
            .with_gain(gain)
            .with_play_once(is_play_once)
            .with_pitch(pitch)
            .with_looping(looping)
            .with_position(position)
            .with_radius(radius)
            .with_rolloff_factor(rolloff_factor)
            .with_max_distance(max_distance)
            .build()
            .unwrap();

        let engine_state = self.sound_engine.state();

        // getting spatial sounds context
        let mut context_state = engine_state.contexts()[1].state();

        let handle = context_state.add_source(source);

        return handle;
    }


    pub fn pause_sound(&mut self, handle: Handle<SoundSource>) {
        let st = self.sound_engine.state();
        
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


    pub fn stop_sound(&mut self, handle: Handle<SoundSource>, pause: bool) {
        let st = self.sound_engine.state();
        
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

    pub fn play_sound(&mut self, handle: Handle<SoundSource>) {
        let st = self.sound_engine.state();
        
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


    pub fn sound_set_gain(&mut self, handle: Handle<SoundSource>, gain: f32) {
        let st = self.sound_engine.state();
        
        let mut state = st.contexts()[0].state();
        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_gain(gain);

        } else {
            let mut state = st.contexts()[1].state();
            if state.is_valid_handle(handle) {
                let sound = state.source_mut(handle);

                sound.set_gain(gain);
            }
        }
    }


    pub fn sound_set_pitch(&mut self, handle: Handle<SoundSource>, pitch: f64) {
        let st = self.sound_engine.state();
        
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

    pub fn sound_set_pitch_and_gain(&mut self, handle: Handle<SoundSource>, pitch: f64, gain: f32) {
        let st = self.sound_engine.state();
        
        let mut state = st.contexts()[0].state();
        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_pitch(pitch);
            sound.set_gain(gain);

        } else {
            let mut state = st.contexts()[1].state();
            if state.is_valid_handle(handle) {
                let sound = state.source_mut(handle);

                sound.set_pitch(pitch);
                sound.set_gain(gain);
            }
        }
    }


    pub fn sound_set_looping(&mut self, handle: Handle<SoundSource>, looping: bool) {
        let st = self.sound_engine.state();
        
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

    pub fn sound_set_position(&mut self, handle: Handle<SoundSource>, postion: Vec4) {
        
        let position = Vector3::<f32>::new(
            postion.x,
            postion.y + postion.w,
            postion.z,
        );
        
        let st = self.sound_engine.state();
        let mut state = st.contexts()[1].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_position(position);
        }
    }

    pub fn sound_set_radius(&mut self, handle: Handle<SoundSource>, radius: f32) {
        let st = self.sound_engine.state();
        
        let mut state = st.contexts()[1].state();
        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_radius(radius);
        }
    }

    pub fn sound_set_rolloff_factor(&mut self, handle: Handle<SoundSource>, rolloff_factor: f32) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[1].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_rolloff_factor(rolloff_factor);
        }
    }

    pub fn sound_set_max_distance(&mut self, handle: Handle<SoundSource>, max_distance: f32) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[1].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_max_distance(max_distance);
        }
    }


    pub fn remove_sound(&mut self, handle: Handle<SoundSource>) {
        let st = self.sound_engine.state();
        
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


    pub async fn new() -> Self {

        let sound_engine = SoundEngine::new().expect("Can't initialize sound engine");

        let not_spatial_context = SoundContext::new();
        not_spatial_context.state().set_distance_model(DistanceModel::None);
        
        let spatial_context = SoundContext::new();
        spatial_context.state().set_distance_model(DistanceModel::LinearDistance);


        
        // index 0 is for not spatial sounds and 1 for spatial sounds
        sound_engine.state().add_context(not_spatial_context);
        sound_engine.state().add_context(spatial_context);

        let mut sounds = HashMap::with_capacity(20);

        #[cfg(not(target_arch="wasm32"))]
        let path = "/home/maffi/Dream/web-engine4d".to_string();
        #[cfg(target_arch="wasm32")]
        let path = "http://127.0.0.1:5500".to_string();

        let machinegun_shot_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_file(
                path.clone() + "/src/assets/sounds/machinegun_shot.wav",
                &fyrox_resource::io::FsResourceIo
            )
            .await
            .expect("can't open file")
        ).expect("can't create sound buffer resourse");

        let holegun_shot_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_file(
                path.clone() + "/src/assets/sounds/holegun_shot.wav",
                &fyrox_resource::io::FsResourceIo
            )
            .await
            .expect("can't open file")
        ).expect("can't create sound buffer resourse");

        let holegun_charging_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_file(
                path.clone() + "/src/assets/sounds/holegun_charging.wav",
                &fyrox_resource::io::FsResourceIo
            )
            .await
            .expect("can't open file")
        ).expect("can't create sound buffer resourse");

        let rotating_around_w_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_file(
                path.clone() + "/src/assets/sounds/rotating_around_w.wav",
                &fyrox_resource::io::FsResourceIo
            )
            .await
            .expect("can't open file")
        ).expect("can't create sound buffer resourse");

        let player_explosion = SoundBufferResource::new_generic(
            DataSource::from_file(
                path.clone() + "/src/assets/sounds/player_explosion.wav",
                &fyrox_resource::io::FsResourceIo
            )
            .await
            .expect("can't open file")
        ).expect("can't create sound buffer resourse");

        let player_hit_signal = SoundBufferResource::new_generic(
            DataSource::from_file(
                path.clone() + "/src/assets/sounds/player_hit_signal.wav",
                &fyrox_resource::io::FsResourceIo
            )
            .await
            .expect("can't open file")
        ).expect("can't create sound buffer resourse");

        let player_death_signal = SoundBufferResource::new_generic(
            DataSource::from_file(
                path.clone() + "/src/assets/sounds/player_death_signal.wav",
                &fyrox_resource::io::FsResourceIo
            )
            .await
            .expect("can't open file")
        ).expect("can't create sound buffer resourse");

        sounds.insert(Sound::MachinegunShot, machinegun_shot_sound_resource);
        sounds.insert(Sound::HolegunShot, holegun_shot_sound_resource);
        sounds.insert(Sound::HolegunCharging, holegun_charging_sound_resource);
        sounds.insert(Sound::RotatingAroundW, rotating_around_w_sound_resource);
        sounds.insert(Sound::PlayerExplosion, player_explosion);
        sounds.insert(Sound::PlayerHitSignal, player_hit_signal);
        sounds.insert(Sound::PlayerDeathSignal, player_death_signal);

        AudioSystem {
            sound_engine,
            sounds
        }
    }
}