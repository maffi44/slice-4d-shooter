use glam::{Vec3, Vec4};

use crate::{
    engine::{
        physics::{area::{Area, AreaMessages}, colliders_container::PhysicalElement, physics_system_data::ShapeType}, render::VisualElement, time::TimeSystem, world::static_object::{SphericalVolumeArea, StaticObject, VolumeArea}
    },
    transform::Transform
};

use super::{
    session_controller::SessionControllerMessage,
    Actor,
    ActorID,
    CommonActorsMessages,
    Message,
    MessageType,
    PhysicsMessages,
    SpecificActorMessage
};

#[derive(Clone)]
pub enum MoveWBonusSpotMessage
{
    SetBonusStatus(
        // index
        u32,
        // status
        BonusSpotStatus,
    ),

    YouTryingToGetMoveWBonus(
        // index
        u32
    ),
}

#[derive(Clone)]
pub enum BonusSpotStatus
{
    BonusOnTheSpot,
    BonusCollected(
        // ActorID of a player collected the bonus
        u128
    )
}

impl From<client_server_protocol::BonusSpotStatus> for BonusSpotStatus
{
    fn from(value: client_server_protocol::BonusSpotStatus) -> Self {
        match value
        {
            client_server_protocol::BonusSpotStatus::BonusOnTheSpot =>
            {
                BonusSpotStatus::BonusOnTheSpot
            }
            client_server_protocol::BonusSpotStatus::BonusCollected(id) =>
            {
                BonusSpotStatus::BonusCollected(id)
            }
        }
    }
}

pub const MoveWBonusAreaRadius: f32 = 0.4;

pub struct MoveWBonusSpot
{
    transform: Transform,
    id: Option<ActorID>,
    status: BonusSpotStatus,
    index: u32,
    area: Area,
    visual_areas: Vec<VolumeArea>,
}

impl MoveWBonusSpot
{
    
    pub fn new(transform: Transform, index: u32) -> Self
    {
        let area = Area::new(
            Vec4::ZERO,
            ShapeType::Sphere,
            Vec4::new(
                MoveWBonusAreaRadius,
                0.0, 0.0, 0.0
            )
        );

        let mut visual_areas = Vec::with_capacity(1);

        let test_visual_area =  VolumeArea::SphericalVolumeArea(
            SphericalVolumeArea {
                radius: MoveWBonusAreaRadius,
                translation: Vec4::ZERO,
                color: Vec3::ONE,
            }
        );

        visual_areas.push(test_visual_area);

        MoveWBonusSpot {
            transform,
            id: None,
            status: BonusSpotStatus::BonusOnTheSpot,
            index,
            area,
            visual_areas,
        }
    }
}

impl Actor for MoveWBonusSpot
{
    fn get_mut_transform(&mut self) -> &mut Transform {
        &mut self.transform
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


    fn get_physical_element(&mut self) -> Option<PhysicalElement>
    {
        match self.status
        {
            BonusSpotStatus::BonusOnTheSpot =>
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

            BonusSpotStatus::BonusCollected(_) =>
            {
                None
            }
        }

    }


    fn get_visual_element(&self) -> Option<VisualElement>
    {
        match self.status
        {
            BonusSpotStatus::BonusOnTheSpot =>
            {
                Some(
                    VisualElement
                    {
                        transform: &self.transform,
                        static_objects: None,
                        coloring_areas: None,
                        volume_areas: Some(&self.visual_areas),
                        player: None,
                    }
                )
            }

            BonusSpotStatus::BonusCollected(_) =>
            {
                None
            }
        }
    }


    fn recieve_message(
            &mut self,
            message: Message,
            engine_handle: &mut crate::engine::engine_handle::EngineHandle,
            physics_system: &crate::engine::physics::PhysicsSystem,
            audio_system: &mut crate::engine::audio::AudioSystem,
            ui_system: &mut crate::engine::ui::UISystem,
            time_system: &TimeSystem,
        ) {
        
        let Message {
            from,
            message
        } = message;

        match message
        {
            MessageType::CommonActorsMessages(message) =>
            {
                match message
                {
                    CommonActorsMessages::IWasChangedMyId(new_id) =>
                    {
                        match self.status {
                            BonusSpotStatus::BonusCollected(id) =>
                            {
                                if from == id
                                {
                                    self.status = BonusSpotStatus::BonusCollected(new_id);
                                }
                            }
                            _ => {}
                        }
                    }

                    CommonActorsMessages::SetTransform(tr) =>
                    {
                        self.transform = tr;
                    }

                    CommonActorsMessages::IncrementPosition(incr) =>
                    {
                        self.transform.increment_position(incr);
                    }

                    CommonActorsMessages::Enable(switch) =>
                    {

                    }
                }
            }

            MessageType::SpecificActorMessage(message) =>
            {
                match message
                {
                    SpecificActorMessage::SessionControllerMessage(message) =>
                    {
                        match message {
                            SessionControllerMessage::JoinedToSession(
                                _,
                                _,
                                _,
                                bonus_status,
                                _,
                                _
                            ) =>
                            {
                                self.area.clear_containing_colliders_list();
                                self.status = BonusSpotStatus::from(bonus_status);
                            }
                            
                            SessionControllerMessage::NewSessionStarted(_) =>
                            {
                                self.area.clear_containing_colliders_list();
                                self.status = BonusSpotStatus::BonusOnTheSpot;
                            }
                            
                            _ => {}
                        }
                    }

                    SpecificActorMessage::MoveWBonusSpotMessage(message) =>
                    {
                        match message {
                            MoveWBonusSpotMessage::SetBonusStatus(
                                index,
                                new_status,
                            ) =>
                            {
                                if self.index == index
                                {
                                    self.area.clear_containing_colliders_list();
                                    self.status = new_status;
                                }
                            }

                            MoveWBonusSpotMessage::YouTryingToGetMoveWBonus(_) => {}
                        }
                    }

                    _ => {}
                }
            }

            MessageType::PhysicsMessages(message) =>
            {
                match message {
                    PhysicsMessages::AreaMessage(message) =>
                    {
                        match message {
                            AreaMessages::ActorEnterArea(id) =>
                            {
                                engine_handle.send_direct_message(
                                    id,
                                    Message {
                                        from: self.get_id().expect("move w bonus spot have not ActorId"),
                                        message: MessageType::SpecificActorMessage(
                                            SpecificActorMessage::MoveWBonusSpotMessage(
                                                MoveWBonusSpotMessage::YouTryingToGetMoveWBonus(self.index)
                                            )
                                        )
                                    }
                                );
                            }

                            AreaMessages::ActorIsContainedInsideArea(id) =>
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