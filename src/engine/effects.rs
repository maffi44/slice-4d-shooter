use glam::{Vec3, Vec4};


pub enum EffectType {
    DefaultPistolDecal,
}

pub struct EffectsSystem
{

}

impl EffectsSystem
{
    pub fn new() -> Self
    {
        EffectsSystem
        {

        }
    }

    pub fn spawn_wave(
        &mut self,
        origin: Vec4,
        radius: Vec<f32>,
        colors: Vec<Vec3>,
        speeds: Vec<f32>,
    )
    {
        assert!(
            radius.len() >= 2
        );

        assert!(
            radius.len() == colors.len() &&
            radius.len() == (speeds.len() + 1)
        );

        // spawn wave
    }
}