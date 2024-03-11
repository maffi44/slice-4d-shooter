use crate::actor::{
    Component,
    ActorID,
};

pub enum DynamicColliderMessages {
    
}

pub struct DynamicCollider {
    actors_id: Option<ActorID>,
}

impl DynamicCollider {
    pub fn new() -> Self {
        DynamicCollider {
            actors_id: None
        }
    }
}

impl Component for DynamicCollider {
    fn init(&mut self, id: ActorID) {
        self.actors_id = Some(id);
    }

    fn get_id(&self) -> Option<ActorID> {
        let id = self.actors_id.expect("Component was not initialised");

        Some(id)
    }
}