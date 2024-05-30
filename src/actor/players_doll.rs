use fyrox_core::pool::Handle;
use fyrox_sound::source::SoundSource;
use glam::{Vec3, Vec4};
use matchbox_socket::PeerId;

use crate::{
    engine::{
        audio::{AudioSystem, Sound}, engine_handle::{
            Command,
            CommandType,
            EngineHandle
        }, net::{
            NetCommand,
            NetMessage,
            RemoteMessage
        }, physics::{
            colliders_container::PhysicalElement,
            dynamic_collider::PlayersDollCollider,
            PhysicsSystem
        }, render::VisualElement, world::static_object::{
            SphericalVolumeArea,
            VolumeArea
        }
    },
    transform::Transform
};

use super::{
    device::holegun::HOLE_GUN_COLOR,
    holegun_miss::HoleGunMiss,
    holegun_shot::HoleGunShot,
    machinegun_shot::MachinegunShot,
    player::{
        PlayerMessages,
        TIME_TO_DIE_SLOWLY
    },
    players_death_explosion::PlayersDeathExplosion,
    shooting_impact::ShootingImpact,
    Actor,
    ActorID,
    ActorWrapper,
    CommonActorsMessages,
    Component,
    Message,
    MessageType,
    SpecificActorMessage
};


const PLAYERS_DOLL_COLOR: Vec3 = Vec3::new(0.8, 0.8, 0.8);
pub struct PlayersDoll {
    id: Option<ActorID>,
    transform: Transform,
    masters_peer_id: PeerId,
    weapon_shooting_point: Vec4,
    is_alive: bool,

    volume_area: Vec<VolumeArea>,
    charging_time: f32,

    dynamic_colliders: Vec<PlayersDollCollider>,
    is_enable: bool,

    need_to_die_slowly: bool,
    die_slowly_timer: f32,

    test_sound: Handle<SoundSource>
}

pub enum PlayersDollMessages{
    SpawnHoleGunShotActor(Vec4, f32, Vec3, f32),
    SpawHoleGunMissActor(Vec4, f32, Vec3, f32),
    SpawnMachineGunShot(Vec4, bool),
    HoleGunStartCharging,
    Respawn(Vec4),
}

const VISUAL_BEAM_MULT: f32 = 2.0;
const VISUAL_FIRE_SHPERE_MULT: f32 = 2.4;

impl PlayersDoll {
    pub fn new(
        masters_peer_id: PeerId,
        id: ActorID,
        player_sphere_radius: f32,
        transform: Transform,
        is_alive: bool,
        audio_system: &mut AudioSystem,
    ) -> Self {

        let test_sound = audio_system.spawn_spatial_sound(
            Sound::RotatingAroundW,
            0.6,
            1.0,
            true,
            false,
            fyrox_sound::source::Status::Playing,
            transform.get_position(),
            2.0,
            4.0,
            50.0
        );

        let weapon_offset = {
            Vec4::new(
                1.0,
                0.26,
                0.0,
                0.0
            ).normalize() * (player_sphere_radius * 1.35)
        };

        let dynamic_collider = PlayersDollCollider {
            position: Vec4::ZERO,
            radius: player_sphere_radius,
            friction: 0.0,
            bounce_rate: 0.0,
            actors_id: Some(id),
            weapon_offset,
        };

        let mut dynamic_colliders = Vec::with_capacity(1);

        dynamic_colliders.push(dynamic_collider);

        let weapon_shooting_point = weapon_offset + Vec4::NEG_Z * (player_sphere_radius * 0.49);

        PlayersDoll {
            masters_peer_id,
            weapon_shooting_point,
            id: Some(id),
            transform,
            charging_time: 0.0,
            volume_area: Vec::with_capacity(1),
            is_alive,
            is_enable: is_alive,
            dynamic_colliders,
            need_to_die_slowly: false,
            die_slowly_timer: 0.0,
            test_sound,
        }
    }



    fn die_immediately(&mut self, engine_handle: &mut EngineHandle, audio_system: &mut AudioSystem) {
        if self.is_alive {

            self.volume_area.clear();

            self.is_alive = false;
            self.is_enable = false;
            self.need_to_die_slowly = false;

            self.play_die_effects(engine_handle, audio_system);
        }
    }



    fn play_die_effects(&mut self, engine_handle: &mut EngineHandle, audio_system: &mut AudioSystem) {
        let players_death_explode = PlayersDeathExplosion::new(
            self.get_transform().get_position()
        );
        
        engine_handle.send_command(
            Command {
                sender: self.get_id().expect("Player have not ActorID"),
                command_type: CommandType::SpawnActor(
                    super::ActorWrapper::PlayersDeathExplosion(players_death_explode)
                )
            }
        );

        audio_system.spawn_spatial_sound(
            Sound::PlayerExplosion,
            0.6,
            1.0,
            false,
            true,
            fyrox_sound::source::Status::Playing,
            self.transform.get_position(),
            2.0,
            4.0,
            50.0
        );
    }



    fn die_slowly(&mut self, engine_handle: &mut EngineHandle) {
        if self.is_alive {

            self.volume_area.clear();

            self.is_alive = false;
            self.is_enable = true;
            self.need_to_die_slowly = true;
            self.die_slowly_timer = 0.0;
        }
    }



    fn respawn(
        &mut self,
        spawn_position: Vec4,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem
    ) {
        let collider_radius = self.dynamic_colliders[0].radius;

        let hits = physics_system.sphere_cast_on_dynamic_colliders(spawn_position, collider_radius);

        for hit in hits {
            engine_handle.send_direct_message(
                hit.hited_actors_id.expect("In respawn func in death on resapwn hit have not ActorID"),
                Message {
                    from: self.get_id().expect("Player have not ID in respawn func"),
                    message: MessageType::SpecificActorMessage(
                        SpecificActorMessage::PLayerMessages(
                            PlayerMessages::Telefrag
                        )
                    )
                }
            )
        }

        self.is_alive = true;
        self.is_enable = true;
        self.transform = Transform::from_position(spawn_position);
    }
}



impl Actor for PlayersDoll {
    fn recieve_message(
        &mut self,
        message: &Message,
        engine_handle: &mut EngineHandle,
        physics_system: &PhysicsSystem,
        audio_system: &mut AudioSystem
    ) {
        let from = message.from;

        let message = &message.message;
        
        match message {
            MessageType::CommonActorsMessages(message) => {
                match message {
                    &CommonActorsMessages::SetTransform(transform) => {
                        self.transform = transform.clone();
                    },
                    CommonActorsMessages::Enable(switch) => {
                        self.is_enable = *switch;
                    },

                    CommonActorsMessages::IncrementPosition(increment) => {
                        self.transform.increment_position(*increment);
                    },
                    CommonActorsMessages::IWasChangedMyId(new_id) => {}
                }
            }
            MessageType::PhysicsMessages(message) => {
                match message {
                    _ => {}
                }
            },
            MessageType::SpecificActorMessage(message) => {
                match message {
                    SpecificActorMessage::PLayerMessages(message) => {
                        match message {
                            PlayerMessages::Telefrag => {
                                self.die_immediately(engine_handle, audio_system);

                                engine_handle.send_command(
                                    Command {
                                        sender: self.id.expect("Player's Doll have not Actor's ID"),
                                        command_type: CommandType::NetCommand(
                                            NetCommand::SendDirectNetMessageReliable(
                                                NetMessage::RemoteDirectMessage(
                                                    self.id.expect("Player's Doll have not Actor's ID"),
                                                    RemoteMessage::DieImmediately
                                                ),
                                                self.masters_peer_id
                                            )
                                        )
                                    }
                                )
                            }
                            PlayerMessages::DieImmediately => {
                                self.die_immediately(engine_handle, audio_system);
                            }
                            PlayerMessages::DieSlowly => {
                                self.die_slowly(engine_handle);
                            }
                            PlayerMessages::DealDamageAndAddForce(damage, force, impact_pos) => {
                                engine_handle.send_command(
                                    Command {
                                        sender: self.id.expect("Player's Doll have not Actor's ID"),
                                        command_type: CommandType::NetCommand(
                                            NetCommand::SendDirectNetMessageReliable(
                                                NetMessage::RemoteDirectMessage(
                                                    self.id.expect("Player's Doll have not Actor's ID"),
                                                    RemoteMessage::DealDamageAndAddForce(
                                                        *damage,
                                                        force.to_array(),
                                                        impact_pos.to_array(),
                                                    )
                                                ),
                                                self.masters_peer_id
                                            )
                                        )
                                    }
                                );

                                engine_handle.send_command(Command {
                                    sender: 0u128,
                                    command_type: CommandType::SpawnActor(
                                        ActorWrapper::ShootingImpact(
                                            ShootingImpact::new(*impact_pos, *damage)
                                        )
                                    )
                                })

                            }
                            PlayerMessages::NewPeerConnected(_) => {}
                        }
                    },
                    SpecificActorMessage::PlayersDollMessages(message) => {
                        match message {
                            PlayersDollMessages::HoleGunStartCharging => {

                                if self.volume_area.is_empty() {

                                    let volume_area = VolumeArea::SphericalVolumeArea(
                                        SphericalVolumeArea {
                                            color: HOLE_GUN_COLOR,
                                            translation: self.transform.get_rotation().inverse() * self.weapon_shooting_point,
                                            radius: 0.1 * VISUAL_FIRE_SHPERE_MULT,
                                        }
                                    );
                    
                                    self.volume_area.push(volume_area);
                                }
                            }
                            PlayersDollMessages::Respawn(spawn_position) => {
                                self.respawn(*spawn_position, engine_handle, physics_system)
                            }
                            PlayersDollMessages::SpawnHoleGunShotActor(
                                position,
                                radius,
                                color,
                                charging_volume_area
                            ) => {
                                self.volume_area.clear();
                                self.charging_time = 0.0;

                                let shooted_from = self.transform.get_position() + self.transform.get_rotation().inverse() * self.weapon_shooting_point;

                                let charging_volume_area = VolumeArea::SphericalVolumeArea(
                                    SphericalVolumeArea {
                                        translation: shooted_from,
                                        radius: (*charging_volume_area + 0.05) *VISUAL_FIRE_SHPERE_MULT,
                                        color: *color,
                                    }
                                );
    
                                let holegun_shot = HoleGunShot::new(
                                    *position,
                                    shooted_from,
                                    *radius,
                                    *color,
                                    charging_volume_area,
                                    VISUAL_BEAM_MULT,
                                );
    
                                let actor = ActorWrapper::HoleGunShot(holegun_shot);
    
                                engine_handle.send_command(Command {
                                    sender: 0u128,
                                    command_type: CommandType::SpawnActor(actor)
                                })
                            },


                            PlayersDollMessages::SpawHoleGunMissActor(
                                position,
                                radius,
                                color,
                                charging_volume_area
                            ) => {

                                self.volume_area.clear();
                                self.charging_time = 0.0;

                                let shooted_from = self.transform.get_position() + self.transform.get_rotation().inverse() * self.weapon_shooting_point;

                                let charging_volume_area = VolumeArea::SphericalVolumeArea(
                                    SphericalVolumeArea {
                                        translation: shooted_from,
                                        radius: (*charging_volume_area + 0.05) *VISUAL_FIRE_SHPERE_MULT,
                                        color: *color,
                                    }
                                );

                                let holegun_miss = HoleGunMiss::new(
                                    *position,
                                    shooted_from,
                                    *radius,
                                    *color,
                                    charging_volume_area,
                                    VISUAL_BEAM_MULT,
                                );

                                let actor = ActorWrapper::HoleGunMiss(holegun_miss);

                                engine_handle.send_command(Command {
                                    sender: 0u128,
                                    command_type: CommandType::SpawnActor(actor)
                                })
                            },

                            PlayersDollMessages::SpawnMachineGunShot(position, it_is_miss) => {
                                let shooted_from = self.transform.get_position() + self.transform.get_rotation().inverse() * self.weapon_shooting_point;

                                let machinegun_shot = MachinegunShot::new(
                                    *position,
                                    shooted_from,
                                    2.0,
                                    2.0,
                                    *it_is_miss,
                                );

                                let actor = ActorWrapper::MachinegunShot(machinegun_shot);

                                engine_handle.send_command(Command {
                                    sender: 0u128,
                                    command_type: CommandType::SpawnActor(actor)
                                })
                            }
                        }
                    }
                }

            }  
        }
    }


    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
    }


    fn get_transform(&self) -> &Transform {
        &self.transform
    }


    fn init(&mut self, id: ActorID) {
        self.id = Some(id);

        for collider in self.dynamic_colliders.iter_mut() {
            collider.init(id);
        }
    }


    fn get_id(&self) -> Option<ActorID> {
        self.id
    }

    fn set_id(&mut self, id: ActorID, engine_handle: &mut EngineHandle) {
        
        if let Some(prev_id) = self.id {
            engine_handle.send_boardcast_message(Message {
                from: prev_id,
                message: MessageType::CommonActorsMessages(
                    CommonActorsMessages::IWasChangedMyId(
                        id
                    )
                )
            });
        }

        self.id = Some(id);
    }

    fn get_physical_element(&mut self) -> Option<PhysicalElement> {
        if self.is_enable {
            Some(
                PhysicalElement {
                    transform: &mut self.transform,
                    kinematic_collider: None,
                    static_colliders: None,
                    dynamic_colliders: Some(&mut self.dynamic_colliders),
                    static_objects: None,
                    area: None,
                }
            )
        } else {
            None
        }
    }

    fn get_visual_element(&self) -> Option<VisualElement> {
        if self.is_enable {
            Some(
                VisualElement {
                    transform: &self.transform,
                    static_objects: None,
                    coloring_areas: None,
                    volume_areas: Some(&self.volume_area),
                    player: Some(&self.dynamic_colliders[0])
                }
            )
        } else {
            None
        }
    }

    fn tick(
        &mut self,
        physic_system: &PhysicsSystem,
        engine_handle: &mut EngineHandle,
        audio_system: &mut AudioSystem,
        delta: f32
    ) {
        if self.is_alive {
            
            if !self.volume_area.is_empty() {
    
                self.charging_time += delta * 1.6;
    
                match &mut self.volume_area[0] {
    
                    VolumeArea::SphericalVolumeArea(area) => {
                        if self.charging_time < 3.4 {
                            area.radius = self.charging_time * 0.08 * VISUAL_FIRE_SHPERE_MULT;
                        }
                        area.translation = self.transform.get_rotation().inverse() * self.weapon_shooting_point;
                    }
                    _ => {
                        panic!("charging volume area in PLayersDoll is not SphericalVolumeArea")
                    }
                }
            } 
        } else {
            if self.need_to_die_slowly {
                self.die_slowly_timer += delta;

                if self.die_slowly_timer >= TIME_TO_DIE_SLOWLY {
                    self.is_enable = false;
                    self.need_to_die_slowly = false;
                    self.play_die_effects(engine_handle, audio_system);
                }
            }
        }

        audio_system.sound_set_position(
            self.test_sound,
            self.transform.get_position()
        );
    }
}