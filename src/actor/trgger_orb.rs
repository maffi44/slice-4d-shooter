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

use client_server_protocol::{NetCommand, NetMessageToServer, Team};
use fyrox_sound::source::Status;
use glam::{FloatExt, Vec3, Vec4};
use rand::Rng;

use crate::{
    actor::trigger::TriggerMessage, engine::{
        audio::{AudioSystem, Sound}, effects::EffectsSystem, engine_handle::{Command, CommandType, EngineHandle}, physics::{PhysicsSystem, area::{Area, AreaMessage}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, render::VisualElement, time::TimeSystem, ui::{UIElementType, UISystem}, world::static_object::{SphericalVolumeArea, VisualWave, VolumeArea}
    }, transform::Transform
};

use super::{
    main_player::{BLUE_TEAM_COLOR, RED_TEAM_COLOR}, session_controller::SessionControllerMessage, Actor, ActorID, CommonActorsMessage, Message, MessageType, PhysicsMessages, SpecificActorMessage
};

#[derive(Clone)]
pub enum TriggerOrbMessage
{
    TriggerOrbCapturedByPlayer(ActorID),
    YouInteractingWithTriggerOrb,
    GiveMeTargetPosition,
    SetTargetPosition(Vec4),
    TriggerOrbReachedTheTrigger,
}

#[derive(Clone, Copy)]
pub enum TrggerOrbStatus
{
    Captured(ActorID),
    OnTheTrigger,
    MovingToTrigger(ActorID),
    OnTheSpot,
}

const TIME_TO_CHANGE_NEXT_TARGET_SWING_POSITION: f32 = 0.8;
const FLAG_SWING_RANGE: f32 = 0.07;
const FLAG_UI_TICK_TIME: f32 = 0.5;

const WAVE_TICK_TIME: f32 = 1.5;
const WAVE_TARGET_RADIUS: f32 = 2.0;

fn get_random_vec4(range_min: f32, range_max: f32) -> Vec4
{
    assert!(range_min < range_max);

    let mut rng = rand::thread_rng();
    let x = rng.gen_range(range_min..=range_max);
    let y = rng.gen_range(range_min..=range_max);
    let z = rng.gen_range(range_min..=range_max);
    let w = rng.gen_range(range_min..=range_max);

    return Vec4::new(x, y, z, w);
}

pub struct TriggerOrb
{
    transform: Transform,
    id: Option<ActorID>,
    area: Area,
    status: TrggerOrbStatus,

    next_target_swing_position_in_secs: f32,
    target_flag_swing_position: Vec4,
    current_orb_swing_position: Vec4,
    target_position: Vec4,
    visual_areas: Vec<VolumeArea>,
    radius_mult: f32,
    wave_tick_timer: f32,
    waves: Vec<VisualWave>,
    wave_1_target_rad: f32,
    wave_2_target_rad: f32,

    radius: f32,
    color: Vec3,
    target_trigger_name: String,
}

impl TriggerOrb
{
    pub fn new(
        transform: Transform,
        target_trigger_name: String,
        color: Vec3,
        radius: f32,
    ) -> Self
    {
        let target_flag_swing_position = get_random_vec4(
            -FLAG_SWING_RANGE,
            FLAG_SWING_RANGE
        );

        let area: Area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(
                radius,
                0.0, 0.0, 0.0
            )
        );

        let mut visual_areas = Vec::with_capacity(1);

        let visual_area =  VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                radius: radius,
                translation: Vec4::ZERO,
                color: color,
            }
        );

        visual_areas.push(visual_area);

        let waves = vec![
            VisualWave {
                translation: Vec4::ZERO,
                radius: 0.0,
                color: color,
            },
            VisualWave {
                translation: Vec4::ZERO,
                radius: 1.6,
                color: color,
            }

        ];

        TriggerOrb {
            transform,
            target_flag_swing_position,
            current_orb_swing_position: Vec4::ZERO,
            target_position: transform.get_position(),
            id: None,
            status:TrggerOrbStatus::OnTheSpot,
            next_target_swing_position_in_secs: TIME_TO_CHANGE_NEXT_TARGET_SWING_POSITION,
            area,
            visual_areas,
            waves,
            radius_mult: 1.0,
            wave_tick_timer: 0.0,
            wave_1_target_rad: WAVE_TARGET_RADIUS,
            wave_2_target_rad: 0.0,

            target_trigger_name,
            radius,
            color,
        }
    }

    // pub fn set_flag_on_base_status(
    //     &mut self,
    //     effects_system: &mut EffectsSystem,
    //     audio_system: &mut AudioSystem,
    //     engine_handle: &mut EngineHandle,
    // )
    // {

    //     self.transform = self.transfrom_of_the_base;
    //     self.target_position = self.transfrom_of_the_base.get_position();

    //     match self.status
    //     {
    //         TrggerOrbStatus::Missed(_) =>
    //         {
    //             // effects_system.spawn_wave(
    //             //     engine_handle,
    //             //     self.transform.get_position(),
    //             //     vec![
    //             //         0.0,
    //             //         24.0,
    //             //     ],
    //             //     vec![
    //             //         self.my_color,
    //             //         Vec3::ZERO
    //             //     ],
    //             //     vec![
    //             //         1.5,
    //             //     ]
    //             // );
    //         }
    //         _ => {}
    //     }

    //     audio_system.spawn_non_spatial_sound(
    //         Sound::FlagOnTheBase,
    //         1.0,
    //         1.0,
    //         false,
    //         true,
    //         Status::Playing,
    //     );
        
    //     self.status = TrggerOrbStatus::OnTheBase;
    // }

    // pub fn set_flag_missed_status(
    //     &mut self,
    //     pos: Vec4,
    //     effects_system: &mut EffectsSystem,
    //     engine_handle: &mut EngineHandle,
    // )
    // {
    //     self.target_position = pos;
    //     // effects_system.spawn_wave(
    //     //     engine_handle,
    //     //     self.transform.get_position(),
    //     //     vec![
    //     //         0.0,
    //     //         24.0,
    //     //     ],
    //     //     vec![
    //     //         self.my_color,
    //     //         Vec3::ZERO
    //     //     ],
    //     //     vec![
    //     //         1.5,
    //     //     ]
    //     // );
    //     self.status = TrggerOrbStatus::Missed(pos);
    // }

    // pub fn set_flag_captured_status(
    //     &mut self,
    //     captured_by: ActorID,
    //     engine_handle: &mut EngineHandle,
    //     effects_system: &mut EffectsSystem,
    //     audio_system: &mut AudioSystem,
    // )
    // {
    //     self.area.clear_containing_colliders_list();
        
    //     engine_handle.send_direct_message(
    //         captured_by,
    //         Message {
    //             from: self.id.expect("Flag has no ActorID"),
    //             remote_sender: false,
    //             message: MessageType::SpecificActorMessage(
    //                 SpecificActorMessage::TriggerOrbMessage(
    //                     TriggerOrbMessage::GiveMeTargetPosition
    //                 )
    //             )
    //         }
    //     );

    //     effects_system.spawn_wave(
    //         engine_handle,
    //         self.transform.get_position(),
    //         vec![
    //             0.0,
    //             2.0,
    //             24.0,
    //         ],
    //         vec![
    //             self.my_color,
    //             self.opponent_color,
    //             Vec3::ZERO
    //         ],
    //         vec![
    //             0.3,
    //             1.2,
    //         ]
    //     );
        
    //     audio_system.spawn_spatial_sound(
    //         Sound::FlagCuptured,
    //         1.0,
    //         1.0,
    //         false,
    //         true,
    //         Status::Playing,
    //         self.transform.get_position(),
    //         1.0,
    //         1.0,
    //         15.0
    //     );

    //     self.status = TrggerOrbStatus::Captured(captured_by);
    // }
}

impl Actor for TriggerOrb
{
    fn tick(
        &mut self,
        physic_system: &crate::engine::physics::PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut crate::engine::audio::AudioSystem,
        ui_system: &mut crate::engine::ui::UISystem,
        time_system: &mut crate::engine::time::TimeSystem,
        effects_system: &mut EffectsSystem,
        delta: f32
    )
    {
        match self.status
        {
            TrggerOrbStatus::Captured(_) | TrggerOrbStatus::MovingToTrigger(_) =>
            {
                self.radius_mult = self.radius_mult.lerp(0.25, 1.0 - delta*3.0);
            }

            _ =>
            {
                self.radius_mult = self.radius_mult.lerp(1.0, 1.0 - delta*3.0);
            }
        }

        self.next_target_swing_position_in_secs -= delta;

        if self.next_target_swing_position_in_secs <= 0.0
        {
            self.target_flag_swing_position = get_random_vec4(
                -FLAG_SWING_RANGE*self.radius,
                FLAG_SWING_RANGE*self.radius
            );

            self.next_target_swing_position_in_secs = TIME_TO_CHANGE_NEXT_TARGET_SWING_POSITION;
        }

        self.current_orb_swing_position = self.current_orb_swing_position.lerp(
            self.target_flag_swing_position,
            delta * 0.3
        );

        let mut current_orb_position = self.transform.get_position();

        current_orb_position = current_orb_position.lerp(
            self.target_position,
            delta * 9.0
        );

        current_orb_position += self.current_orb_swing_position;

        self.transform.set_position(current_orb_position);

        match self.status
        {
            TrggerOrbStatus::Captured(id) =>
            {
                engine_handle.send_direct_message(
                    id,
                    Message {
                        from: self.id.expect("TriggerOrb has no ActorID"),
                        remote_sender: false,
                        message: MessageType::SpecificActorMessage(
                            SpecificActorMessage::TriggerOrbMessage(
                                TriggerOrbMessage::GiveMeTargetPosition
                            )
                        )
                    }
                );
            }
            TrggerOrbStatus::MovingToTrigger(trigger_id) =>
            {
                if self.transform.get_position().distance(self.target_position) < 1.9
                {
                    engine_handle.send_direct_message(
                        trigger_id,
                        Message {
                            from: self.id.expect("TriggerOrb has no ActorID"),
                            remote_sender: false,
                            message: MessageType::SpecificActorMessage(
                                SpecificActorMessage::TriggerOrbMessage(
                                    TriggerOrbMessage::TriggerOrbReachedTheTrigger
                                )
                            )
                        }
                    );

                    self.status = TrggerOrbStatus::OnTheTrigger;

                    effects_system.spawn_wave(
                        engine_handle,
                        self.transform.get_position(),
                        vec![
                            0.0,
                            2.0,
                            24.0,
                        ],
                        vec![
                            self.color,
                            Vec3::ONE,
                            Vec3::ZERO
                        ],
                        vec![
                            0.3,
                            1.2,
                        ]
                    );
                    
                    audio_system.spawn_spatial_sound(
                        Sound::GetScore,
                        1.0,
                        1.0,
                        false,
                        true,
                        Status::Playing,
                        self.transform.get_position(),
                        1.0,
                        1.0,
                        15.0
                    );
                }
            }
            _ => {}
        }

        self.wave_tick_timer += delta;

        if self.wave_tick_timer >= WAVE_TICK_TIME
        {
            self.wave_tick_timer = 0.0;

            if self.wave_1_target_rad == 0.0
            {
                self.wave_1_target_rad = WAVE_TARGET_RADIUS*self.radius;
                self.wave_2_target_rad = 0.0;
            }
            else
            {
                self.wave_1_target_rad = 0.0;
                self.wave_2_target_rad = WAVE_TARGET_RADIUS*self.radius;
            }
        }
        
        self.waves[0].radius = self.waves[0].radius.lerp(
            self.wave_1_target_rad * self.radius_mult,
            delta*WAVE_TICK_TIME*3.0
        );
        self.waves[0].color = self.waves[0].color.lerp(
            self.wave_2_target_rad * self.color,
            delta*WAVE_TICK_TIME*3.0
        );

        self.waves[1].radius = self.waves[1].radius.lerp(
            self.wave_2_target_rad  * self.radius_mult,
            delta*WAVE_TICK_TIME*0.9
        );

        self.waves[1].color = self.waves[1].color.lerp(
            self.wave_1_target_rad * self.color,
            delta*WAVE_TICK_TIME*0.9
        );

        match &mut self.visual_areas[0]
        {
            VolumeArea::SphericalVolumeArea(area) =>
            {
                area.radius = self.radius * self.radius_mult;
            }
            _ => {}
        }
    }

    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement>
    {
        match self.status
        {
            TrggerOrbStatus::OnTheSpot =>
            {
                Some(
                    PhysicalElement
                    {
                        id: self.get_id().expect("Actor have not ActorID"),
                        transform: &mut self.transform,
                        kinematic_collider: None,
                        dynamic_colliders: None,
                        static_colliders: None,
                        static_objects: None,
                        area: Some(&mut self.area)
                    }
                )
            }

            _ =>
            {
                None
            }
        }

    }


    fn get_visual_element(&self) -> Option<VisualElement>
    {
        Some(
            VisualElement
            {
                transform: &self.transform,
                static_objects: None,
                coloring_areas: None,
                volume_areas: Some(&self.visual_areas),
                waves: Some(&self.waves),
                player: None,
                child_visual_elem: None,
            }
        )
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn recieve_message(
            &mut self,
            message: Message,
            engine_handle: &mut EngineHandle,
            physics_system: &PhysicsSystem,
            audio_system: &mut AudioSystem,
            ui_system: &mut UISystem,
            time_system: &TimeSystem,
            effects_system: &mut EffectsSystem,
        ) {
        
        let from = message.from;

        match message.message
        {
            MessageType::SpecificActorMessage(message) =>
            {
                match message {
                    SpecificActorMessage::TriggerOrbMessage(message) =>
                    {
                        match message
                        {
                            TriggerOrbMessage::SetTargetPosition(position) =>
                            {
                                match self.status
                                {
                                    TrggerOrbStatus::Captured(id) =>
                                    {
                                        if id == from
                                        {
                                            self.target_position = position;
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            TriggerOrbMessage::TriggerOrbCapturedByPlayer(player_id) =>
                            {
                                self.status = TrggerOrbStatus::Captured(player_id);

                                audio_system.spawn_spatial_sound(
                                    Sound::FlagCuptured,
                                    1.0,
                                    1.0,
                                    false,
                                    true,
                                    Status::Playing,
                                    self.transform.get_position(),
                                    1.0,
                                    1.0,
                                    15.0
                                );
                            }
                            
                            _ => {}
                        }
                    }
                    SpecificActorMessage::TriggerMessage(message) =>
                    {
                        match message
                        {
                            TriggerMessage::ActorEnteredTriggerArea(trigger_name, trigger_id, entered_actor_id, trigger_pos) =>
                            {
                                match self.status {
                                    TrggerOrbStatus::Captured(player_id) =>
                                    {
                                        if player_id == entered_actor_id && trigger_name == self.target_trigger_name
                                        {
                                            self.status = TrggerOrbStatus::MovingToTrigger(trigger_id);
                                            self.target_position = trigger_pos;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            MessageType::CommonActorsMessages(message) =>
            {
                match message {
                    CommonActorsMessage::SetTransform(tr) =>
                    {
                        self.transform = tr;
                    }
                    CommonActorsMessage::ClientDisconnectedFromGameServer =>
                    {
                        // self.set_flag_on_base_status(effects_system, audio_system, engine_handle);
                    }
                    _ => {}
                }
            }
            MessageType::PhysicsMessages(message) =>
            {
                match message {
                    PhysicsMessages::AreaMessage(message) =>
                    {
                        match message
                        {
                            AreaMessage::ActorEnterArea(id) =>
                            {
                                engine_handle.send_direct_message(
                                    id,
                                    Message {
                                        from: self.id.expect("Flag have not ActorID"),
                                        remote_sender: false,
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::TriggerOrbMessage(
                                                TriggerOrbMessage::YouInteractingWithTriggerOrb
                                            )
                                        )
                                    }
                                );
                            }
                            AreaMessage::ActorIsContainedInsideArea(id) =>
                            {

                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
        
    }
}

