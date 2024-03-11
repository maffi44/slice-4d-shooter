use crate::engine::input::ActionsFrameState;



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