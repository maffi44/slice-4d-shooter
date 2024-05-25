use std::collections::HashMap;

use fyrox_core::pool::Handle;
use fyrox_resource::{core::reflect::GetField, Resource};
use fyrox_sound::{
    buffer::{
        DataSource, SoundBuffer, SoundBufferResource, SoundBufferResourceExtension
    }, context::SoundContext, engine::{self, SoundEngine}, source::{
        SoundSource, SoundSourceBuilder, Status
    }
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::{ArrayBuffer, Uint8Array}, Response};


#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Sound {
    MachinegunShot,
    HolegunShot,
} 
pub struct AudioSystem {
    sound_engine: SoundEngine,
    sounds: HashMap<Sound, Resource<SoundBuffer>>
}


impl AudioSystem {

    pub fn play_sound(&mut self, sound: Sound, gain: f32) {
        let sound_buffer = self.sounds
            .get(&sound)
            .expect("Some sound is not exist");

        let source = SoundSourceBuilder::new()
            .with_buffer(sound_buffer.clone())
            .with_status(Status::Playing)
            .with_gain(gain)
            .with_play_once(true)
            .build()
            .unwrap();

        let engine_state = self.sound_engine.state();

        let mut context_state = engine_state.contexts()[0].state();

        let _ = context_state.add_source(source);
    }

    pub fn play_sound_with_pitch(&mut self, sound: Sound, gain: f32, pitch: f32) {
        let sound_buffer = self.sounds
            .get(&sound)
            .expect("Some sound is not exist");

        let source = SoundSourceBuilder::new()
            .with_buffer(sound_buffer.clone())
            .with_status(Status::Playing)
            .with_pitch(pitch as  f64)
            .with_gain(gain)
            .with_play_once(true)
            .build()
            .unwrap();

        let engine_state = self.sound_engine.state();

        let mut context_state = engine_state.contexts()[0].state();

        let _ = context_state.add_source(source);
    }

    // pub fn stop_sound(&mut self, sound: Sound) {
    //     let handle = self.sounds
    //         .get(&sound)
    //         .expect("Some sounde is not exist");

    //     let engine_state = self.sound_engine.state();

    //     let mut context_state = engine_state.contexts()[0].state();

    //     let sound_source = context_state.source_mut(*handle);

    //     let _ = sound_source.stop();
    // }

    // pub fn set_loop_sound(&mut self, sound: Sound, looping: bool) {
    //     let handle = self.sounds
    //         .get(&sound)
    //         .expect("Some sounde is not exist");

    //     let engine_state = self.sound_engine.state();

    //     let mut context_state = engine_state.contexts()[0].state();

    //     let sound_source = context_state.source_mut(*handle);

    //     sound_source.set_looping(looping);
    // }

    pub async fn new() -> Self {

        let sound_engine = SoundEngine::new().unwrap();

        let context = SoundContext::new();
        
        sound_engine.state().add_context(context);

        let mut sounds = HashMap::with_capacity(20);

        #[cfg(not(target_arch="wasm32"))]
        {
            let machinegun_shot_sound_resource = SoundBufferResource::new_generic(
                DataSource::from_file(
                    "/home/maffi/Dream/web-engine4d/src/assets/sounds/machinegun_shot.wav",
                    &fyrox_resource::io::FsResourceIo
                )
                .await
                .expect("can't open file")
            ).expect("can't create sound buffer resourse");

            let holegun_shot_sound_resource = SoundBufferResource::new_generic(
                DataSource::from_file(
                    "/home/maffi/Dream/web-engine4d/src/assets/sounds/holegun_shot.wav",
                    &fyrox_resource::io::FsResourceIo
                )
                .await
                .expect("can't open file")
            ).expect("can't create sound buffer resourse");

            sounds.insert(Sound::MachinegunShot, machinegun_shot_sound_resource);
            sounds.insert(Sound::HolegunShot, holegun_shot_sound_resource);
        }

        

        // Currently  have very freaky bug on web (connection isn't done)
        #[cfg(target_arch="wasm32")]
        let laser_sound_resource = SoundBufferResource::new_generic(
            DataSource::from_file(
                "../src/assets/sounds/machinegun_shot.wav",
                &fyrox_resource::io::FsResourceIo
            )
            .await
            .expect("can't open file")
        ).expect("can't create sound buffer resourse");

        // #[cfg(target_arch="wasm32")]
        // let sound_buffer = {
        //     let window = web_sys::window().unwrap();
        
        //     let target = "http://127.0.0.1:5500/src/assets/sounds/test.wav";
            
        //     let promise = window.fetch_with_str(target);
        
        //     let result = JsFuture::from(promise).await;

        //     let response: Response = result.unwrap().dyn_into().unwrap();

        //     let res: ArrayBuffer = JsFuture::from(response.array_buffer().unwrap()).await.unwrap().unchecked_into();

        //     let array = Uint8Array::new(&res);

        //     let bytes = array.to_vec();

        //     SoundBufferResource::new_generic(
        //         DataSource::from_memory(
        //             bytes
        //         )
        //     ).expect("can't create sound buffer resourse")
        // };


        
        
        AudioSystem {
            sound_engine,
            sounds
        }
    }
}