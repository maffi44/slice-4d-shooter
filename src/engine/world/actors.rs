// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::collections::{hash_map, HashMap};

use crate::actor::{Actor, ActorID, ActorWrapper};

use super::messenger::Messenger;


/// The actors of one World (the former `World` actor container).
pub struct Actors
{
    actors: HashMap<ActorID, ActorWrapper>,
    main_actor_id: ActorID,
}


impl Actors
{
    pub fn new() -> Self
    {
        Actors {
            actors: HashMap::new(),
            main_actor_id: 0,
        }
    }

    /// Inserts an actor, assigning a random non-zero id if it has none.
    /// If the id is already taken, the actor that had it gets a new random id (as before).
    pub fn insert(&mut self, mut actor: ActorWrapper, messenger: &mut Messenger) -> ActorID
    {
        let id = match actor.get_id()
        {
            Some(id) => id,
            None =>
            {
                let new_id = self.get_new_random_uniq_id();
                actor.set_id(new_id);
                new_id
            },
        };

        if let Some(mut swapped_actor) = self.actors.insert(id, actor)
        {
            let new_id_for_swapped_actor = self.get_new_random_uniq_id();

            swapped_actor.change_id(new_id_for_swapped_actor, messenger);

            self.actors.insert(new_id_for_swapped_actor, swapped_actor);
        }

        id
    }

    pub fn remove(&mut self, id: ActorID) -> Option<ActorWrapper>
    {
        self.actors.remove(&id)
    }

    /// Changes an actor's id (keeps the main actor id in sync).
    pub fn change_id(&mut self, old_id: ActorID, new_id: ActorID, messenger: &mut Messenger)
    {
        if let Some(mut actor) = self.actors.remove(&old_id)
        {
            actor.change_id(new_id, messenger);

            if let Some(mut swapped_actor) = self.actors.insert(new_id, actor)
            {
                let new_id_for_swapped_actor = self.get_new_random_uniq_id();

                swapped_actor.change_id(new_id_for_swapped_actor, messenger);

                self.actors.insert(new_id_for_swapped_actor, swapped_actor);
            }

            if self.main_actor_id == old_id
            {
                self.main_actor_id = new_id;
            }
        }
    }

    #[inline]
    pub fn main_actor_id(&self) -> ActorID
    {
        self.main_actor_id
    }

    #[inline]
    pub fn set_main_actor_id(&mut self, id: ActorID)
    {
        self.main_actor_id = id;
    }

    #[inline]
    pub fn get(&self, id: &ActorID) -> Option<&ActorWrapper>
    {
        self.actors.get(id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: &ActorID) -> Option<&mut ActorWrapper>
    {
        self.actors.get_mut(id)
    }

    #[inline]
    pub fn iter(&self) -> hash_map::Iter<'_, ActorID, ActorWrapper>
    {
        self.actors.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> hash_map::IterMut<'_, ActorID, ActorWrapper>
    {
        self.actors.iter_mut()
    }

    pub fn retain(&mut self, f: impl FnMut(&ActorID, &mut ActorWrapper) -> bool)
    {
        self.actors.retain(f);
    }

    fn get_new_random_uniq_id(&self) -> ActorID
    {
        let mut new_id = get_random_non_zero_id();

        while self.actors.contains_key(&new_id)
        {
            new_id = get_random_non_zero_id();
        }

        new_id
    }
}


fn get_random_non_zero_id() -> ActorID
{
    let mut bytes: [u8; 16] = [0; 16];

    if getrandom::getrandom(&mut bytes).is_err()
    {
        panic!("can't make a random u128 in get_random_id");
    }

    let id: u128 = u128::from_be_bytes(bytes);

    // 0 is reserved: it's the sender id of messages from outside the world (engine / owner)
    if id == 0u128
    {
        return get_random_non_zero_id();
    }

    id
}
