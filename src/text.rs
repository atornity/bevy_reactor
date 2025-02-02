use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::{
    node_span::NodeSpan, scope::TrackingScope, view::View, DespawnScopes, IntoView, Rcx, ViewRef,
};

/// A UI element that displays text
pub struct TextStatic {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// The text to display
    text: String,
}

impl TextStatic {
    /// Construct a new static text view.
    pub fn new(text: String) -> Self {
        Self { node: None, text }
    }
}

impl View for TextStatic {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Node(self.node.unwrap())
    }

    fn build(&mut self, _view_entity: Entity, world: &mut World) {
        assert!(self.node.is_none());
        self.node = Some(
            world
                .spawn((TextBundle {
                    text: Text::from_section(self.text.clone(), TextStyle { ..default() }),
                    ..default()
                },))
                .id(),
        );
    }

    fn raze(&mut self, _view_entity: Entity, world: &mut World) {
        // Delete the display node.
        let display = self.node.expect("Razing unbuilt DynTextNode");
        world.entity_mut(display).remove_parent();
        world.entity_mut(display).despawn();
    }
}

/// Creates a static text view.
pub fn text(text: &str) -> TextStatic {
    TextStatic::new(text.to_string())
}

/// A UI element that displays text that is dynamically computed.
pub struct TextComputed<F: FnMut(&Rcx) -> String> {
    /// The visible UI node for this element.
    node: Option<Entity>,

    /// The text to display
    text: F,
}

impl<F: FnMut(&Rcx) -> String> TextComputed<F> {
    /// Construct a new computed text view.
    pub fn new(text: F) -> Self {
        Self { node: None, text }
    }
}

impl IntoView for TextStatic {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}

impl<F: FnMut(&Rcx) -> String> View for TextComputed<F> {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Node(self.node.unwrap())
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.node.is_none());
        let mut tracking = TrackingScope::new(world.change_tick());
        let re = Rcx::new(world, &mut tracking);
        let text = (self.text)(&re);
        let node = Some(
            world
                .spawn((TextBundle {
                    text: Text::from_section(text, TextStyle { ..default() }),
                    ..default()
                },))
                .id(),
        );
        self.node = node;
        world.entity_mut(view_entity).insert(tracking);
    }

    fn react(&mut self, _view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let re = Rcx::new(world, tracking);
        let text = (self.text)(&re);
        world
            .entity_mut(self.node.unwrap())
            .get_mut::<Text>()
            .unwrap()
            .sections[0]
            .value = text;
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        let display = self.node.expect("Razing unbuilt DynTextNode");
        world.entity_mut(display).remove_parent();
        world.entity_mut(display).despawn();
        world.despawn_owned_recursive(view_entity);
    }
}

/// Creates a computed text view.
pub fn text_computed<F: FnMut(&Rcx) -> String>(text: F) -> TextComputed<F> {
    TextComputed::new(text)
}

impl<F: Send + Sync + 'static + FnMut(&Rcx) -> String> IntoView for TextComputed<F> {
    fn into_view(self) -> ViewRef {
        Arc::new(Mutex::new(self))
    }
}
