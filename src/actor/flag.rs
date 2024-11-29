use super::{
    Actor,
    ActorID,
};

pub struct Flag
{

}

impl Actor for Flag
{
    fn get_mut_transform(&mut self) -> &mut crate::transform::Transform {
        todo!()
    }

    fn get_transform(&self) -> &crate::transform::Transform {
        todo!()
    }

    fn get_id(&self) -> Option<ActorID> {
        todo!()
    }

    fn change_id(&mut self, id: ActorID, engine_handle: &mut crate::engine::engine_handle::EngineHandle) {
        todo!()
    }

    fn set_id(&mut self, id: ActorID) {
        todo!()
    }
}

