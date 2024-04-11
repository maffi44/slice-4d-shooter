use glam::{Vec3, Vec4};

use crate::{
    actor::{
        device::{
            Device,
            DeviceType,
        },
        holegun_miss::HoleGunMiss,
        holegun_shot::HoleGunShot,
        player::{
            PlayerInnerState,
            PlayerMessages
        },
        ActorID,
        ActorWrapper,
        Message,
        MessageType,
        SpecificActorMessage
    },
    engine::{
        engine_handle::{
            Command,
            CommandType,
            EngineHandle,
        },
        input::ActionsFrameState,
        net::{
            NetCommand,
            NetMessage,
            RemoteCommand
        },
        physics::PhysicsSystem,
        render::VisualElement,
        world::static_object::{
            SphericalVolumeArea,
            VolumeArea
        }
    },
    transform::Transform
};

pub struct HoleGun {
    charging_time: f32,
    shooted_on_this_charge: bool,
    is_charging: bool,
    color: Vec3,
    volume_area: Vec<VolumeArea>,
    shooted_from_pivot_point_dir: Vec4,
}


impl HoleGun {
    pub fn new() -> Self {
        let shooted_from_pivot_point_mult = Vec4::new(
            1.0,
            -0.42,
            -1.0,
            0.0
        );

        HoleGun {
            charging_time: 0.0,
            shooted_on_this_charge: false,
            color: Vec3::new(1.0, 0.6, 0.0),
            volume_area: Vec::with_capacity(1),
            shooted_from_pivot_point_dir: shooted_from_pivot_point_mult,
            is_charging: false,
        }
    }

    fn shoot(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        charging_time: f32,
        color: Vec3,
    ) {
        let from = player.transform.get_position() + Vec4::Y * player.collider.get_collider_radius() * 0.98;
                
        let direction = player.transform.rotation.inverse() * Vec4::NEG_Z;
    
        let weapon_offset = {
            (Vec4::Y * player.collider.get_collider_radius() * 0.98) +
            (player.transform.rotation.inverse() *
            (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
        };

        let volume_area = self.volume_area.pop().expect("Hole Gun doesn't have volume area on shoot");
        
        let volume_area_radius = match &volume_area {
            VolumeArea::SphericalVolumeArea(area) => {
                area.radius
            },
            _ => {panic!("Charging volume area in holegun is not SphericalVolumeArea")}
        };

        let hit = physic_system.ray_cast(from, direction, 700.0);

        if let Some(hit) = hit {

            let position = hit.hit_point;
            let shooted_from = player.transform.get_position() + weapon_offset;
            let radius = charging_time*1.2;

            let hited_players = physic_system.sphere_cast_on_dynamic_colliders(
                position,
                radius,
            );

            for hit in hited_players {
                let dist_to_hited_point = {
                    hit.hit_point.distance(position)
                };

                let damage = (radius * 100.0) / (1.0 + dist_to_hited_point*10.0);

                let force = hit.hit_normal * damage / -4.5;

                log::error!("SET DAMAGE and FORCE: {}, {}", damage, force);

                engine_handle.send_direct_message(
                    hit.hited_actors_id.expect("Hited Player have not Actor's ID"),
                    Message {
                        from: player_id,
                        message: MessageType::SpecificActorMessage(
                            SpecificActorMessage::PLayerMessages(
                                PlayerMessages::DealDamageAndAddForce(
                                    damage as u32,
                                    force,
                                )
                            )
                        )
                    }
                );
            }

            let hole = HoleGunShot::new(
                position,
                shooted_from,
                radius,
                color,
                volume_area,
            );
    
            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::SpawnActor(
                        ActorWrapper::HoleGunShot(hole)
                    )
                }
            );

            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessage::RemoteCommand(
                                RemoteCommand::SpawnHoleGunShotActor(
                                    position.to_array(),
                                    shooted_from.to_array(),
                                    radius,
                                    color.to_array(),
                                    volume_area_radius,
                                )
                            )
                        )
                    )
                }
            )

        } else {
            let position = from + (direction * 700.0);
            let shooted_from = player.transform.get_position() + weapon_offset;
            let radius = charging_time*1.2;

            let miss = HoleGunMiss::new(
                position,
                shooted_from,
                radius,
                color,
                volume_area,
            );

            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::SpawnActor(
                        ActorWrapper::HoleGunMiss(miss)
                    )
                }
            );

            engine_handle.send_command(
                Command {
                    sender: player_id,
                    command_type: CommandType::NetCommand(
                        NetCommand::SendBoardcastNetMessageReliable(
                            NetMessage::RemoteCommand(
                                RemoteCommand::SpawHoleGunMissActor(
                                    position.to_array(),
                                    shooted_from.to_array(),
                                    radius,
                                    color.to_array(),
                                    volume_area_radius,
                                )
                            )
                        )
                    )
                }
            )
        }
    }
}


impl Device for HoleGun {
    fn get_device_type(&self) -> DeviceType {
        DeviceType::Gun
    }

    fn get_visual_element<'a>(&'a self, transform: &'a Transform) -> Option<VisualElement<'a>> {
        Some(
            VisualElement {
                transform,
                static_objects: None,
                coloring_areas: None,
                volume_areas: Some(&self.volume_area),
                player: None,
            }
        )
    }

    fn process_input(
        &mut self,
        player_id: ActorID,
        player: &mut PlayerInnerState,
        input: &ActionsFrameState,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        delta: f32,
    )
    {
        if input.first_mouse.is_action_pressed() {

            if !self.shooted_on_this_charge {

                if !self.is_charging {
                    self.is_charging = true;
    
                    let shooted_from_offset = {
                        (Vec4::Y * player.collider.get_collider_radius() * 0.98) +
                        (player.transform.rotation.inverse() *
                        (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
                    };
    
                    let volume_area = VolumeArea::SphericalVolumeArea(
                        SphericalVolumeArea {
                            color: self.color,
                            translation: shooted_from_offset,
                            radius: 0.05,
                        }
                    );
    
                    self.volume_area.push(volume_area);
                }

                self.charging_time += delta * 1.6;
                
                match &mut self.volume_area[0] {
                    
                    VolumeArea::SphericalVolumeArea(area) => {
                        let shooted_from_offset = {
                            (Vec4::Y * player.collider.get_collider_radius() * 0.98) +
                            (player.transform.rotation.inverse() *
                            (self.shooted_from_pivot_point_dir.normalize() * player.collider.get_collider_radius()))
                        };

                        area.radius = self.charging_time * 0.08;
                        area.translation = shooted_from_offset;
                    }
                    _ => {
                        panic!("charging volume area in HoleGun is not SphericalVolumeArea")
                    }
                }
    
                if self.charging_time > 3.4 {
    
                    self.shooted_on_this_charge = true;
                    
                    self.shoot(
                        player_id,
                        player,
                        physic_system,
                        engine_handle,
                        self.charging_time,
                        self.color,
                    );
    
                    self.charging_time = 0.0;
                    self.is_charging = false;

                }
            }
        } else {

            self.shooted_on_this_charge = false;

            if self.charging_time > 0.0 {

                self.shoot(
                    player_id,
                    player,
                    physic_system,
                    engine_handle,
                    self.charging_time,
                    self.color,
                );

                self.charging_time = 0.0;
                self.is_charging = false;
            }
        }
    }

}