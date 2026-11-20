// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::collections::{hash_map, HashMap};

use crate::actor::{Actor, ActorID, ActorWrapper};

use super::messenger::{Messenger, OUTER_WORLD_SENDER_ID};


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

    /// Inserts an actor and returns its id.
    ///
    /// An actor without an id, or with the reserved id `OUTER_WORLD_SENDER_ID` (0), gets a fresh
    /// random non-zero id. If the id is already taken, the actor that had it is displaced to a fresh
    /// random id (it broadcasts `IWasChangedMyId`); if the displaced actor was the main actor,
    /// `main_actor_id` follows it.
    pub fn insert(&mut self, mut actor: ActorWrapper, messenger: &mut Messenger) -> ActorID
    {
        let id = match actor.get_id()
        {
            Some(id) if id != OUTER_WORLD_SENDER_ID => id,
            requested_id =>
            {
                if requested_id.is_some()
                {
                    log::warn!("Actors::insert: the actor has the reserved id 0, a random id is assigned instead");
                }

                let new_id = self.get_new_random_uniq_id();
                actor.set_id(new_id);
                new_id
            },
        };

        self.vacate(id, messenger);
        self.actors.insert(id, actor);

        id
    }

    pub fn remove(&mut self, id: ActorID) -> Option<ActorWrapper>
    {
        self.actors.remove(&id)
    }

    /// Changes an actor's id (`CommandType::ChangeMyId`); the actor broadcasts `IWasChangedMyId`.
    ///
    /// If `new_id` is taken, the actor that had it is displaced to a fresh random id, exactly as in
    /// `insert`. `main_actor_id` keeps identifying the same actor in every case: it follows the
    /// changed actor if that was the main one, or the displaced actor if that was the main one.
    /// The reserved id 0, an unknown `old_id` and `old_id == new_id` leave everything unchanged.
    ///
    /// Messages already queued are delivered by id at delivery time: direct messages to a vacated id
    /// are dropped, and direct messages to a reused id reach its new holder.
    pub fn change_id(&mut self, old_id: ActorID, new_id: ActorID, messenger: &mut Messenger)
    {
        if new_id == OUTER_WORLD_SENDER_ID
        {
            log::error!("Actors::change_id: id 0 is reserved for messages from outside the world, the id is not changed");
            return;
        }

        if old_id == new_id
        {
            return;
        }

        if !self.actors.contains_key(&old_id)
        {
            log::warn!("Actors::change_id: no actor with the old id, the id is not changed");
            return;
        }

        let changed_actor_was_main = self.main_actor_id == old_id;

        // The displaced actor's notification (new -> random) must be queued before the changed
        // actor's one (old -> new): see `vacate`. Vacating while `old_id` is still taken also keeps
        // the displaced actor's random id different from `old_id`.
        self.vacate(new_id, messenger);

        let mut actor = self.actors.remove(&old_id)
            .expect("the actor disappeared while its new id was vacated");

        actor.change_id(new_id, messenger);

        self.actors.insert(new_id, actor);

        if changed_actor_was_main
        {
            self.main_actor_id = new_id;
        }
    }

    /// Frees `id` before another actor is stored under it: its current holder (if any) is moved
    /// to a fresh random id, broadcasts `IWasChangedMyId` (id -> random), and keeps `main_actor_id`
    /// if it was the main actor.
    ///
    /// Notification order: the displaced actor's notification is queued before the notification of
    /// the actor that takes `id` (old -> id). With FIFO delivery a receiver that tracks ids
    /// (e.g. `MoveWBonusSpot`) first moves a reference to `id` onto the random id, and only then a
    /// reference to `old` onto `id`, so no reference is renamed twice.
    /// See issues/programming/message-ordering/.
    fn vacate(&mut self, id: ActorID, messenger: &mut Messenger)
    {
        if let Some(mut displaced_actor) = self.actors.remove(&id)
        {
            let new_id_for_displaced_actor = self.get_new_random_uniq_id();

            displaced_actor.change_id(new_id_for_displaced_actor, messenger);

            self.actors.insert(new_id_for_displaced_actor, displaced_actor);

            if self.main_actor_id == id
            {
                self.main_actor_id = new_id_for_displaced_actor;
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


#[cfg(test)]
mod tests
{
    use super::*;
    use crate::{
        actor::{move_w_bonus::MoveWBonusSpot, CommonActorsMessage, MessageType},
        engine::transform::Transform,
    };

    fn actor_with_id(id: Option<ActorID>) -> ActorWrapper
    {
        let mut actor = ActorWrapper::MoveWBonusSpot(MoveWBonusSpot::new(Transform::new(), 0));

        if let Some(id) = id
        {
            actor.set_id(id);
        }

        actor
    }

    /// `(from, new id)` of every queued `IWasChangedMyId`, oldest first.
    fn id_change_notifications(messenger: &Messenger) -> Vec<(ActorID, ActorID)>
    {
        messenger.queued_broadcast_messages()
            .filter_map(|message| match message.message
            {
                MessageType::CommonActorsMessages(CommonActorsMessage::IWasChangedMyId(new_id)) =>
                    Some((message.from, new_id)),
                _ => None,
            })
            .collect()
    }

    /// Every actor is stored under its own id, and none has the reserved id.
    fn assert_ids_valid(actors: &Actors, expected_count: usize)
    {
        assert_eq!(actors.iter().count(), expected_count);

        for (id, actor) in actors.iter()
        {
            assert_ne!(*id, OUTER_WORLD_SENDER_ID);
            assert_eq!(actor.get_id(), Some(*id));
        }
    }

    /// A world with a main actor `main_id` and another actor `other_id`.
    fn world_with(main_id: ActorID, other_id: ActorID) -> (Actors, Messenger)
    {
        let mut actors = Actors::new();
        let mut messenger = Messenger::new();

        let id = actors.insert(actor_with_id(Some(main_id)), &mut messenger);
        actors.set_main_actor_id(id);
        actors.insert(actor_with_id(Some(other_id)), &mut messenger);

        (actors, messenger)
    }

    #[test]
    fn insert_displacing_the_main_actor_keeps_main_actor_id_on_it()
    {
        let (mut actors, mut messenger) = world_with(10, 20);

        let id = actors.insert(actor_with_id(Some(10)), &mut messenger);

        assert_eq!(id, 10);
        let main_id = actors.main_actor_id();
        assert_ne!(main_id, 10);
        assert!(actors.get(&main_id).is_some());
        assert_eq!(id_change_notifications(&messenger), vec![(10, main_id)]);
        assert_ids_valid(&actors, 3);
    }

    #[test]
    fn insert_displacing_another_actor_keeps_main_actor_id()
    {
        let (mut actors, mut messenger) = world_with(10, 20);

        actors.insert(actor_with_id(Some(20)), &mut messenger);

        assert_eq!(actors.main_actor_id(), 10);
        let notifications = id_change_notifications(&messenger);
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].0, 20);
        assert_ids_valid(&actors, 3);
    }

    #[test]
    fn insert_replaces_missing_or_reserved_id_with_a_random_one()
    {
        let mut actors = Actors::new();
        let mut messenger = Messenger::new();

        let random_id = actors.insert(actor_with_id(None), &mut messenger);
        let replaced_id = actors.insert(actor_with_id(Some(OUTER_WORLD_SENDER_ID)), &mut messenger);

        assert_ne!(random_id, OUTER_WORLD_SENDER_ID);
        assert_ne!(replaced_id, OUTER_WORLD_SENDER_ID);
        assert!(id_change_notifications(&messenger).is_empty());
        assert_ids_valid(&actors, 2);
    }

    #[test]
    fn main_actor_changing_to_a_free_id_moves_main_actor_id()
    {
        let (mut actors, mut messenger) = world_with(10, 20);

        actors.change_id(10, 30, &mut messenger);

        assert_eq!(actors.main_actor_id(), 30);
        assert_eq!(id_change_notifications(&messenger), vec![(10, 30)]);
        assert_ids_valid(&actors, 2);
    }

    #[test]
    fn main_actor_changing_to_a_taken_id_displaces_the_other_actor()
    {
        let (mut actors, mut messenger) = world_with(10, 20);

        actors.change_id(10, 20, &mut messenger);

        assert_eq!(actors.main_actor_id(), 20);
        let notifications = id_change_notifications(&messenger);
        assert_eq!(notifications.len(), 2);
        assert_eq!(notifications[0].0, 20);
        assert!(actors.get(&notifications[0].1).is_some());
        assert_eq!(notifications[1], (10, 20));
        assert_ids_valid(&actors, 2);
    }

    #[test]
    fn other_actor_changing_to_the_main_id_displaces_the_main_actor()
    {
        let (mut actors, mut messenger) = world_with(10, 20);

        actors.change_id(20, 10, &mut messenger);

        let main_id = actors.main_actor_id();
        assert_ne!(main_id, 10);
        assert_ne!(main_id, 20);
        assert_eq!(id_change_notifications(&messenger), vec![(10, main_id), (20, 10)]);
        assert_ids_valid(&actors, 2);
    }

    #[test]
    fn reserved_unknown_or_same_id_changes_nothing()
    {
        let (mut actors, mut messenger) = world_with(10, 20);

        actors.change_id(10, OUTER_WORLD_SENDER_ID, &mut messenger);
        actors.change_id(10, 10, &mut messenger);
        actors.change_id(99, 20, &mut messenger);

        assert_eq!(actors.main_actor_id(), 10);
        assert!(actors.get(&10).is_some());
        assert!(actors.get(&20).is_some());
        assert!(id_change_notifications(&messenger).is_empty());
        assert_ids_valid(&actors, 2);
    }
}
