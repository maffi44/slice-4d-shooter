// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



// Gameplay values of the player (formerly `PlayerSettings`, loaded from settings2.json).
// They are constants now: gameplay must not be configurable from a local file (that was a cheat vector).
// Used by MainPlayer and by the actors that mirror it (PlayerDoll, Observer, obstacle-course and 2D/3D players).

pub const PLAYER_COLLIDER_RADIUS: f32 = 0.49;

pub const PLAYER_MAX_SPEED: f32 = 40.0;
pub const PLAYER_MAX_ACCEL: f32 = 52.5;

pub const AIR_SPEED_MULT: f32 = 0.232;

pub const JUMP_Y_SPEED: f32 = 9.75;
pub const JUMP_W_SPEED: f32 = 11.5;

pub const GRAVITY_Y_SPEED: f32 = 30.0;
pub const GRAVITY_W_SPEED: f32 = 18.1;

pub const FRICTION_ON_AIR: f32 = 0.16;

pub const MIN_RESPAWN_TIMER: f32 = 6.0;
pub const MAX_RESPAWN_TIMER: f32 = 9.0;

pub const SCANNER_SHOW_ENEMIES_TIME: f32 = 5.5;
pub const SCANNER_RELOADING_TIME: f32 = 7.5;

pub const ENERGY_GUN_HOLE_SIZE_MULT: f32 = 0.12;
pub const ENERGY_GUN_ADD_FORCE_MULT: f32 = 1.05;
pub const ENERGY_GUN_DAMAGE_MULT: f32 = 0.49;
pub const ENERGY_GUN_RESTORING_SPEED: f32 = 14.0;

pub const MACHINEGUN_DAMAGE: f32 = 10.0;
pub const MACHINEGUN_ADD_FORCE: f32 = 0.4;
pub const MACHINEGUN_HEAT_ADD_ON_SHOT: f32 = 5.15;
pub const MACHINEGUN_COOLING_SPEED: f32 = 15.5;
