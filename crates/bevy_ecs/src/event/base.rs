use crate::{component::Component, traversal::Traversal};
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;
#[cfg(feature = "track_location")]
use core::panic::Location;
use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

/// Something that "happens" and might be read / observed by app logic.
///
/// Events can be stored in an [`Events<E>`] resource
/// You can conveniently access events using the [`EventReader`] and [`EventWriter`] system parameter.
///
/// Events can also be "triggered" on a [`World`], which will then cause any [`Observer`] of that trigger to run.
///
/// This trait can be derived.
///
/// Events implement the [`Component`] type (and they automatically do when they are derived). Events are (generally)
/// not directly inserted as components. More often, the [`ComponentId`] is used to identify the event type within the
/// context of the ECS.
///
/// Events must be thread-safe.
///
/// [`World`]: crate::world::World
/// [`ComponentId`]: crate::component::ComponentId
/// [`Observer`]: crate::observer::Observer
/// [`Events<E>`]: super::Events
/// [`EventReader`]: super::EventReader
/// [`EventWriter`]: super::EventWriter
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an `Event`",
    label = "invalid `Event`",
    note = "consider annotating `{Self}` with `#[derive(Event)]`"
)]
pub trait Event: Component {
    /// The component that describes which Entity to propagate this event to next, when [propagation] is enabled.
    ///
    /// [propagation]: crate::observer::Trigger::propagate
    type Traversal: Traversal<Self>;

    /// When true, this event will always attempt to propagate when [triggered], without requiring a call
    /// to [`Trigger::propagate`].
    ///
    /// [triggered]: crate::system::Commands::trigger_targets
    /// [`Trigger::propagate`]: crate::observer::Trigger::propagate
    const AUTO_PROPAGATE: bool = false;
}

/// An `EventId` uniquely identifies an event stored in a specific [`World`].
///
/// An `EventId` can among other things be used to trace the flow of an event from the point it was
/// sent to the point it was processed. `EventId`s increase monotonically by send order.
///
/// [`World`]: crate::world::World
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct EventId<E: Event> {
    /// Uniquely identifies the event associated with this ID.
    // This value corresponds to the order in which each event was added to the world.
    pub id: usize,
    /// The source code location that triggered this event.
    #[cfg(feature = "track_location")]
    pub caller: &'static Location<'static>,
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub(super) _marker: PhantomData<E>,
}

impl<E: Event> Copy for EventId<E> {}

impl<E: Event> Clone for EventId<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: Event> fmt::Display for EventId<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl<E: Event> fmt::Debug for EventId<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "event<{}>#{}",
            core::any::type_name::<E>().split("::").last().unwrap(),
            self.id,
        )
    }
}

impl<E: Event> PartialEq for EventId<E> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<E: Event> Eq for EventId<E> {}

impl<E: Event> PartialOrd for EventId<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E: Event> Ord for EventId<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<E: Event> Hash for EventId<E> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id, state);
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub(crate) struct EventInstance<E: Event> {
    pub event_id: EventId<E>,
    pub event: E,
}
