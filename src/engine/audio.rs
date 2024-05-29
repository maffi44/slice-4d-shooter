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
    HolegunCharging,
    RotatingAroundW,
}
pub struct AudioSystem {
    pub sound_engine: SoundEngine,
    sounds: HashMap<Sound, Resource<SoundBuffer>>
}

// #[cfg(target_arch = "wasm32")]
// struct StartAudioContextOnWebFuture {
//     sound_engine: SoundEngine
// }
// #[cfg(target_arch = "wasm32")]
// impl Future for StartAudioContextOnWebFuture {
//     type Output = ();

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         self.sound_engine.
//         if self.window.inner_size().width == 0 || self.window.inner_size().height == 0 {
//             let wait_fn = {
//                 let waker = Rc::new(cx.waker().clone());
//                 Closure::wrap(Box::new(move || {
//                     waker.as_ref().clone().wake();
//                 }) as Box<dyn Fn()>)
//             };
//             let _ = web_sys::window()
//                 .unwrap()
//                 .set_timeout_with_callback_and_timeout_and_arguments_0(
//                     wait_fn.as_ref().unchecked_ref(),
//                     50,
//                 );
//             wait_fn.forget();

//             Poll::Pending
//         } else {
//             Poll::Ready(())
//         }
//     }
// }


impl AudioSystem {

    pub fn spawn_sound(
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

        let mut context_state = engine_state.contexts()[0].state();

        let handle = context_state.add_source(source);

        return handle;
    }


    pub fn pause_sound(&mut self, handle: Handle<SoundSource>) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[0].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.pause();
        }
    }


    pub fn stop_sound(&mut self, handle: Handle<SoundSource>, pause: bool) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[0].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            let _ = sound.stop();
        }
    }

    pub fn play_sound(&mut self, handle: Handle<SoundSource>) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[0].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.play();
        }
    }


    pub fn sound_set_gain(&mut self, handle: Handle<SoundSource>, gain: f32) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[0].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_gain(gain);
        }
    }


    pub fn sound_set_pitch(&mut self, handle: Handle<SoundSource>, pitch: f64) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[0].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_pitch(pitch);
        }
    }

    pub fn sound_set_pitch_and_gain(&mut self, handle: Handle<SoundSource>, pitch: f64, gain: f32) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[0].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_pitch(pitch);
            sound.set_gain(gain);
        }
    }



    pub fn sound_set_looping(&mut self, handle: Handle<SoundSource>, looping: bool) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[0].state();

        if state.is_valid_handle(handle) {
            let sound = state.source_mut(handle);

            sound.set_looping(looping);
        }
    }


    pub fn remove_sound(&mut self, handle: Handle<SoundSource>) {
        let st = self.sound_engine.state();
        let mut state = st.contexts()[0].state();

        if state.is_valid_handle(handle) {
            state.remove_source(handle);
        }
    }


    pub async fn new() -> Self {

        let sound_engine = SoundEngine::new().expect("Can't initialize sound engine");

        let context = SoundContext::new();
        
        sound_engine.state().add_context(context);

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

        sounds.insert(Sound::MachinegunShot, machinegun_shot_sound_resource);
        sounds.insert(Sound::HolegunShot, holegun_shot_sound_resource);
        sounds.insert(Sound::HolegunCharging, holegun_charging_sound_resource);
        sounds.insert(Sound::RotatingAroundW, rotating_around_w_sound_resource);

        AudioSystem {
            sound_engine,
            sounds
        }
    }
}