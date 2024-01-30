use crate::systems::input::ActionsFrameState;

const MAX_INPUTS_CAPASITY: usize = 20;

type FrameNumber = u64;

pub enum InputMaster {
    LocalMaster(LocalMaster),
    RemoteMaster(RemoteMaster),
}

pub struct LocalMaster {
    pub current_input: ActionsFrameState
}

impl LocalMaster {
    pub fn new(init_atctions: ActionsFrameState) -> Self {

        LocalMaster {
            current_input: init_atctions,
        }
    }
}



pub struct RemoteMaster {
    pub current_input: ActionsFrameState
}


// pub struct WrappedVec<T> {
//     buf: Vec<T>,
    
// }