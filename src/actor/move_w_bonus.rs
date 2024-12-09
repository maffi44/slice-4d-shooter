use crate::{engine::{physics::area::AreaMessages, time::TimeSystem}, transform::Transform};

use super::{session_controller::SessionControllerMessage, Actor, ActorID, CommonActorsMessages, Message, MessageType, PhysicsMessages, SpecificActorMessage};

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

pub struct MoveWBonusSpot
{
    transform: Transform,
    id: Option<ActorID>,
    status: BonusSpotStatus,
    index: u32,
}

impl MoveWBonusSpot
{
    pub fn new(transform: Transform, index: u32) -> Self
    {
        MoveWBonusSpot {
            transform,
            id: None,
            status: BonusSpotStatus::BonusOnTheSpot,
            index,
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
                                self.status = BonusSpotStatus::from(bonus_status);
                            }
                            
                            SessionControllerMessage::NewSessionStarted(_) =>
                            {
                                self.status = BonusSpotStatus::BonusOnTheSpot;
                            }
                            
                            _ => {}
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

                            _ => {}
                        }
                    }

                    _ => {}
                }
            }
        }

    }
}