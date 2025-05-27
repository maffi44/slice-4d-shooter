mod engine;
mod actor;
mod main_loop;
mod transform;

use std::{collections::HashMap, ops::RangeInclusive, time::Duration};

use engine::{input::{ButtonActions, InputSystem, SomeButton}, HeadlessEngine};
use pollster;
use blink_alloc::UnsafeGlobalBlinkAlloc;

use actor::{flag::Flag, main_player::{player_input_master::{InputMaster, LocalMaster}, MainPlayer, PlayerMessage}, session_controller::{self, SessionController}, ActorWrapper, Message, SpecificActorMessage};
use client_server_protocol::Team;
use engine::input::ActionsFrameState;
use rand::{rngs::ThreadRng, Rng};
use winit::event::{ElementState, KeyEvent};


#[global_allocator]
static GLOBAL_ALLOC: UnsafeGlobalBlinkAlloc = unsafe {
    UnsafeGlobalBlinkAlloc::new()
};

fn main() {
    let mut systems = pollster::block_on(
        HeadlessEngine::new()
    );

    spawn_actors(&mut systems);

    let mut imitated_input_state = ImitatedInputState::new(&systems.input); 

    loop
    {
        game_loop_tick(
            &mut systems,
            &mut imitated_input_state
        );

        std::thread::sleep(Duration::from_secs_f64(
            (0.01666666 - systems.time.current_frame_duration).max(0.0)
        ))
    }
}


fn spawn_actors(
    systems: &mut HeadlessEngine
)
{
    let main_player = MainPlayer::new(
        InputMaster::LocalMaster(
            LocalMaster::new(ActionsFrameState::empty())
        ),
        systems.world.players_settings.clone(),
        &mut systems.audio,
        systems.world.level.blue_base_position,
        systems.world.level.red_base_position,
    );

    let main_player_id = systems.world.add_actor_to_world(
        ActorWrapper::MainPlayer(main_player),
        &mut systems.engine_handle,
    );

    systems.engine_handle.send_boardcast_message(
        Message {
            from: 0u128,
            remote_sender: false,
            message: crate::actor::MessageType::SpecificActorMessage(
                SpecificActorMessage::PlayerMessage(
                    PlayerMessage::SetNewTeam(
                        session_controller::DEFAULT_TEAM
                    )
                )
            )
        }
    );

    let red_flag = Flag::new(
        Team::Red,
        systems.world.level.red_flag_base
    );

    systems.world.add_actor_to_world(
        ActorWrapper::Flag(red_flag),
        &mut systems.engine_handle,
    );

    let blue_flag = Flag::new(
        Team::Blue,
        systems.world.level.blue_flag_base
    );

    systems.world.add_actor_to_world(
        ActorWrapper::Flag(blue_flag),
        &mut systems.engine_handle,
    );

    let session_controller = SessionController::new(
        &mut systems.ui,
        systems.world.level.red_flag_base.get_position(),
        systems.world.level.blue_flag_base.get_position(),
        false,
    );
    
    systems.world.add_actor_to_world(
        ActorWrapper::SessionController(session_controller),
        &mut systems.engine_handle,
    );

    systems.world.main_player_id = main_player_id;
}


fn game_loop_tick
(
    systems : &mut HeadlessEngine,
    imitated_input_state: &mut ImitatedInputState,
) {
    systems.time.start_of_frame();

    imitated_input_state.imitate_user_input(
        &mut systems.input,
        systems.time.prev_frame_duration
    );

    #[cfg(target_arch= "wasm32")]
    systems.net.tick(
        &mut systems.engine_handle,
        &mut systems.audio
    );

    #[cfg(not(target_arch= "wasm32"))]
    systems.net.tick(
        systems.input.get_input(),
        &mut systems.engine_handle,
        &mut systems.runtime,
        &mut systems.audio,
        &mut systems.ui,
    );

    systems.input.set_input_to_controlled_actors(&mut systems.world, &mut systems.net);

    systems.world.tick(
        &systems.physic,
        &mut systems.engine_handle,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.time,
        &mut systems.effects,
    );

    systems.world.send_messages_and_process_commands(
        &mut systems.net,
        &systems.physic,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.engine_handle,
        &mut systems.time,
        &mut systems.effects,
    );

    systems.physic.process_physics(
        &mut systems.world, 
        systems.time.prev_frame_duration,
        &mut systems.engine_handle
    );

    systems.world.send_messages_and_process_commands(
        &mut systems.net,
        &systems.physic,
        &mut systems.audio,
        &mut systems.ui,
        &mut systems.engine_handle,
        &mut systems.time,
        &mut systems.effects,
    );

    systems.input.reset_input();

    systems.time.end_of_frame();
}




pub struct ImitatedInputState
{
    shooting_timer: f32,
    no_shooting_timer: f32,
    next_jump_timer: f32,
    next_w_jump_timer: f32,
    next_weapon_change_timer: f32,
    next_use_scanner_timer: f32,
    connect_to_server_button_pressed: bool,

    next_weapon_action: ButtonActions,

    rng: ThreadRng,

    buttons_to_actions_table: HashMap<ButtonActions, SomeButton>
}

const SHOOTING_RNG_RANGE: RangeInclusive<f32> = 5.0..=7.0;
const NO_SHOOTING_RNG_RANGE: RangeInclusive<f32> = 6.0..=12.0;
const NEXT_JUMP_RANG_RANGE: RangeInclusive<f32> = 2.0..=5.0;
const NEXT_W_JUMP_RANG_RANGE: RangeInclusive<f32> = 3.0..=4.0;
const NEXT_WEAPON_CHANGE_RNG_RANGE: RangeInclusive<f32> = 2.0..=5.0;
const NEXT_USE_SCANNER_RNG_RANGE: RangeInclusive<f32> = 12.0..=15.0;

impl ImitatedInputState
{
    pub fn new(input: &InputSystem) -> Self
    {
        let mut rng = fyrox_core::rand::thread_rng();

        let mut buttons_to_actions_table = HashMap::new();
        
        let actions_table = input.get_actions_table();

        for (button, (action, _)) in actions_table.iter()
        {
            match action
            {
                &ButtonActions::ConnectToServer =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                &ButtonActions::MoveLeft =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                &ButtonActions::HandSlot0 =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                &ButtonActions::HandSlot1 =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                &ButtonActions::HandSlot2 =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                &ButtonActions::HandSlot3 =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                &ButtonActions::Jump =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                &ButtonActions::JumpW =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                &ButtonActions::WScanner =>
                {
                    buttons_to_actions_table.insert(*action, *button);
                }
                _ => {}
            }
        }

        ImitatedInputState {
            shooting_timer: rng.gen_range(SHOOTING_RNG_RANGE),
            no_shooting_timer: rng.gen_range(NO_SHOOTING_RNG_RANGE),
            next_jump_timer: rng.gen_range(NEXT_JUMP_RANG_RANGE),
            next_w_jump_timer: rng.gen_range(NEXT_W_JUMP_RANG_RANGE),
            next_weapon_change_timer: rng.gen_range(NEXT_WEAPON_CHANGE_RNG_RANGE),
            next_use_scanner_timer: rng.gen_range(NEXT_USE_SCANNER_RNG_RANGE),
            connect_to_server_button_pressed: false,

            next_weapon_action: ButtonActions::HandSlot0,

            rng,

            buttons_to_actions_table,
        }
    }

    pub fn imitate_user_input(
        &mut self,
        input: &mut InputSystem,
        delta: f32,
    )
    {
        if self.connect_to_server_button_pressed
        {
            match self.buttons_to_actions_table.get(&ButtonActions::ConnectToServer).unwrap()
            {
                SomeButton::KeyCode(key) =>
                {
                    input.set_keyboard_input_by_keycode(
                        *key,
                        ElementState::Released
                    );
                }
                SomeButton::MouseButton(key) =>
                {
                    input.set_mouse_button_input(
                        key,
                        &winit::event::ElementState::Released
                    );
                }
            }
            self.connect_to_server_button_pressed = false;
        }
        else
        {
            match self.buttons_to_actions_table.get(&ButtonActions::ConnectToServer).unwrap()
            {
                SomeButton::KeyCode(key) =>
                {
                    input.set_keyboard_input_by_keycode(
                        *key,
                        ElementState::Pressed
                    );
                }
                SomeButton::MouseButton(key) =>
                {
                    input.set_mouse_button_input(
                        key,
                        &ElementState::Pressed
                    );
                }
            }
            self.connect_to_server_button_pressed = true;
        }

        if self.no_shooting_timer > 0.0
        {
            self.no_shooting_timer -= delta;

            if self.no_shooting_timer <= 0.0
            {
                self.shooting_timer = self.rng.gen_range(SHOOTING_RNG_RANGE);
            }
        }
        else
        {
            self.shooting_timer -= delta;    

            if self.shooting_timer <= 0.0
            {
                input.set_mouse_button_input(
                    &winit::event::MouseButton::Left,
                    &winit::event::ElementState::Released
                );

                self.no_shooting_timer = self.rng.gen_range(NO_SHOOTING_RNG_RANGE);
            }
            else
            {
                input.set_mouse_button_input(
                    &winit::event::MouseButton::Left,
                    &winit::event::ElementState::Pressed
                );
            }
        }

        if self.next_jump_timer <= 0.0
        {
            match self.buttons_to_actions_table.get(&ButtonActions::Jump).unwrap()
            {
                SomeButton::KeyCode(key) =>
                {
                    input.set_keyboard_input_by_keycode(
                        *key,
                        ElementState::Released
                    );
                }
                SomeButton::MouseButton(key) =>
                {
                    input.set_mouse_button_input(
                        key,
                        &winit::event::ElementState::Released
                    );
                }
            }

            self.next_jump_timer = self.rng.gen_range(NEXT_JUMP_RANG_RANGE);
        }
        else
        {
            self.next_jump_timer -= delta;

            if self.next_jump_timer <= 0.0
            {
                match self.buttons_to_actions_table.get(&ButtonActions::Jump).unwrap()
                {
                    SomeButton::KeyCode(key) =>
                    {
                        input.set_keyboard_input_by_keycode(
                            *key,
                            ElementState::Pressed
                        );
                    }
                    SomeButton::MouseButton(key) =>
                    {
                        input.set_mouse_button_input(
                            key,
                            &winit::event::ElementState::Pressed
                        );
                    }
                }
            }
        }


        if self.next_w_jump_timer <= 0.0
        {
            match self.buttons_to_actions_table.get(&ButtonActions::JumpW).unwrap()
            {
                SomeButton::KeyCode(key) =>
                {
                    input.set_keyboard_input_by_keycode(
                        *key,
                        ElementState::Released
                    );
                }
                SomeButton::MouseButton(key) =>
                {
                    input.set_mouse_button_input(
                        key,
                        &winit::event::ElementState::Released
                    );
                }
            }

            self.next_w_jump_timer = self.rng.gen_range(NEXT_W_JUMP_RANG_RANGE);
        }
        else
        {
            self.next_w_jump_timer -= delta;

            if self.next_w_jump_timer <= 0.0
            {
                match self.buttons_to_actions_table.get(&ButtonActions::JumpW).unwrap()
                {
                    SomeButton::KeyCode(key) =>
                    {
                        input.set_keyboard_input_by_keycode(
                            *key,
                            ElementState::Pressed
                        );
                    }
                    SomeButton::MouseButton(key) =>
                    {
                        input.set_mouse_button_input(
                            key,
                            &winit::event::ElementState::Pressed
                        );
                    }
                }
            }
        }


        if self.next_use_scanner_timer <= 0.0
        {
            match self.buttons_to_actions_table.get(&ButtonActions::WScanner).unwrap()
            {
                SomeButton::KeyCode(key) =>
                {
                    input.set_keyboard_input_by_keycode(
                        *key,
                        ElementState::Released
                    );
                }
                SomeButton::MouseButton(key) =>
                {
                    input.set_mouse_button_input(
                        key,
                        &winit::event::ElementState::Released
                    );
                }
            }

            self.next_use_scanner_timer = self.rng.gen_range(NEXT_USE_SCANNER_RNG_RANGE);
        }
        else
        {
            self.next_use_scanner_timer -= delta;

            if self.next_use_scanner_timer <= 0.0
            {
                match self.buttons_to_actions_table.get(&ButtonActions::WScanner).unwrap()
                {
                    SomeButton::KeyCode(key) =>
                    {
                        input.set_keyboard_input_by_keycode(
                            *key,
                            ElementState::Pressed
                        );
                    }
                    SomeButton::MouseButton(key) =>
                    {
                        input.set_mouse_button_input(
                            key,
                            &winit::event::ElementState::Pressed
                        );
                    }
                }
            }
        }



        if self.next_weapon_change_timer <= 0.0
        {
            match self.buttons_to_actions_table.get(&self.next_weapon_action).unwrap()
            {
                SomeButton::KeyCode(key) =>
                {
                    input.set_keyboard_input_by_keycode(
                        *key,
                        ElementState::Released
                    );
                }
                SomeButton::MouseButton(key) =>
                {
                    input.set_mouse_button_input(
                        key,
                        &winit::event::ElementState::Released
                    );
                }
            }

            self.next_weapon_action = match self.next_weapon_action {
                ButtonActions::HandSlot0 => ButtonActions::HandSlot1,
                ButtonActions::HandSlot1 => ButtonActions::HandSlot2,
                ButtonActions::HandSlot2 => ButtonActions::HandSlot0,
                _ => ButtonActions::HandSlot0,
            };

            self.next_weapon_change_timer = self.rng.gen_range(NEXT_WEAPON_CHANGE_RNG_RANGE);
        }
        else
        {
            self.next_weapon_change_timer -= delta;

            if self.next_weapon_change_timer <= 0.0
            {
                match self.buttons_to_actions_table.get(&self.next_weapon_action).unwrap()
                {
                    SomeButton::KeyCode(key) =>
                    {
                        input.set_keyboard_input_by_keycode(
                            *key,
                            ElementState::Pressed
                        );
                    }
                    SomeButton::MouseButton(key) =>
                    {
                        input.set_mouse_button_input(
                            key,
                            &winit::event::ElementState::Pressed
                        );
                    }
                }
            }
        }
    }
}