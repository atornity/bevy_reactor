use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity, world::World},
    hierarchy::BuildWorldChildren,
};

use crate::{scope::TrackingScope, Re, Reaction, ReactionHandle};

/// Trait that produces a bundle and inserts it into the target entity.
pub trait BundleProducer: Sync + Send {
    /// Initialize the bundle and add it to the target entity.
    ///
    /// Arguments:
    /// - `owner`: The entity that tracks ownership of this reaction, the reaction
    ///     will be deleted when the owner is deleted.
    /// - `target`: The entity that the bundle will be inserted into.
    /// - `world`: The Bevy world.
    fn start(&mut self, owner: Entity, target: Entity, world: &mut World);
}

/// Inserts a static, pre-constructed bundle into the target entity. No reactivity.
pub struct BundleStatic<B: Bundle> {
    pub(crate) bundle: Option<B>,
}

impl<B: Bundle> BundleProducer for BundleStatic<B> {
    // For a static bundle, we can just insert it once.
    fn start(&mut self, _owner: Entity, target: Entity, world: &mut World) {
        world.entity_mut(target).insert(self.bundle.take().unwrap());
    }
}

/// Produces a bundle reactively, returns the bundle as a result.
pub struct BundleComputed<B: Bundle, F: FnMut(&mut Re) -> B> {
    reaction: Arc<Mutex<BundleComputedReaction<B, F>>>,
}

impl<B: Bundle, F: FnMut(&mut Re) -> B> BundleComputed<B, F> {
    pub(crate) fn new(factory: F) -> Self {
        Self {
            reaction: Arc::new(Mutex::new(BundleComputedReaction {
                target: None,
                factory,
            })),
        }
    }
}

/// Produces a bundle reactively, returns the bundle as a result.
pub struct BundleComputedReaction<B: Bundle, F: FnMut(&mut Re) -> B> {
    pub(crate) target: Option<Entity>,
    pub(crate) factory: F,
}

impl<B: Bundle, F: Sync + Send + FnMut(&mut Re) -> B> Reaction for BundleComputedReaction<B, F> {
    fn react(&mut self, _owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let mut re = Re::new(world, tracking);
        let b = (self.factory)(&mut re);
        let mut entt = world.entity_mut(self.target.unwrap());
        entt.insert(b);
    }
}

impl<B: Bundle, F: Sync + Send + 'static + FnMut(&mut Re) -> B> BundleProducer
    for BundleComputed<B, F>
{
    // For a static bundle, we can just insert it once.
    fn start(&mut self, owner: Entity, target: Entity, world: &mut World) {
        let mut scope = TrackingScope::new(world.change_tick());
        let reaction = self.reaction.clone();
        let mut inner = reaction.lock().unwrap();
        inner.target = Some(target);
        let handle = ReactionHandle(self.reaction.clone());
        let reaction_id = world.spawn(handle).id();
        inner.react(reaction_id, world, &mut scope);
        world
            .entity_mut(reaction_id)
            .insert(scope)
            .set_parent(owner);
    }
}

// /// Allows reactively mutating a single component. Allows specifying the initial component value,
// /// as well as an update function the mutates the component in place.
// pub struct BundleComputedRef<C: Component, F1: FnMut() -> C, F2: FnMut(&mut Re, &mut C)> {
//     pub(crate) target: Option<Entity>,
//     pub(crate) init: F1,
//     pub(crate) update: F2,
//     pub(crate) marker: PhantomData<C>,
//     pub(crate) tracker: Option<Entity>,
// }

// impl<C: Component, F1: Send + Sync + FnMut() -> C, F2: Send + Sync + FnMut(&mut Re, &mut C)>
//     BundleProducer for BundleComputedRef<C, F1, F2>
// {
//     // Insert the bundle on build, then update.
//     fn init(&mut self, target: Entity, world: &mut World) {
//         assert!(self.tracker.is_none());
//         let mut scope = TrackingScope::new(world.change_tick());
//         let b = (self.init)();
//         let mut entt = world.entity_mut(target);
//         entt.insert(b);
//         self.target = Some(target);
//         self.react(target, world, &mut scope);
//         let tracker = world.spawn(scope);
//         self.tracker = Some(tracker.id());
//     }
// }

// impl<C: Component, F1: Send + Sync + FnMut() -> C, F2: Send + Sync + FnMut(&mut Re, &mut C)>
//     Reaction for BundleComputedRef<C, F1, F2>
// {
//     fn react(&mut self, target: Entity, world: &mut World, tracking: &mut TrackingScope) {
//         let mut entt = world.entity_mut(target);
//         let mut cmp = entt.get_mut::<C>().unwrap();
//         let mut re = Re::new(world, tracking);
//         todo!();
//         // (self.update)(&mut re, &mut cmp);
//     }

//     fn cleanup(&mut self, owner: Entity, world: &mut World) {
//         assert!(self.tracker.is_some());
//         world.entity_mut(self.tracker.unwrap()).despawn();
//         self.tracker = None;
//     }
// }
