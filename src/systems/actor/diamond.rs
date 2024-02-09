use crate::systems::{
    physics::collider::Area,
    engine_handle::EngineHandle,
    transform::Transform,
};

use super::{
    Actor,
    ActorID,
    Message,
};

const DIAMOND_COLLIDER_RADIUS: f32 = 0.2;

pub struct Diamond {
    id: Option<ActorID>,
    collider: Area,
}

impl Diamond {
    pub fn new(transform: Transform) -> Self {
        let collider = Area::new(transform, DIAMOND_COLLIDER_RADIUS);

        Diamond {
            id: None,
            collider,
        }
    }
}

impl Actor<'_> for Diamond {
    fn recieve_message(&mut self, message: Message, engine_handle: &mut EngineHandle) {
        let from = message.from;

        let message = message.message;
        
        // match message {
        //     MessageType::DealDamage(damage) => {
        //         self.inner_state.hp -= damage as i32;
        //     },
        //     MessageType::SetTransform(transform) => {
        //         self.inner_state.collider.transform = transform;
        //     }
        //     MessageType::EnableCollider(enable) => {
        //         self.inner_state.collider.is_enable = enable;
        //     }
        // }
    }

    fn recieve_boardcast_message(&mut self, message: &Message, engine_handle: &mut EngineHandle) {
        let from = message.from;

        let message = &message.message;
        
        // match message {
        //     MessageType::DealDamage(damage) => {
        //         self.inner_state.hp -= *damage as i32;
        //     },
        //     MessageType::SetTransform(transform) => {
        //         self.inner_state.collider.transform = transform.clone();
        //     }
        //     MessageType::EnableCollider(enable) => {
        //         self.inner_state.collider.is_enable = *enable;
        //     }
        // }
    }

    fn set_id(&mut self, id: ActorID) {
        self.id = Some(id);
    }

    fn get_id(&self) -> Option<ActorID> {
        self.id
    }
}