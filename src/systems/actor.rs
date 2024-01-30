pub mod player;

use player::Player;

use super::transform::Transform;


pub type ActorID = u64;

pub enum ActorWrapper {
    Player(Player),
    Diamond,
    Exit,
}

pub enum Message {
    DealDamage(u32),
    SetTransform(Transform),
    EnableCollider(bool)
}