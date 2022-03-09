use std::{any::type_name, marker::PhantomData};

use crate::{
    Access, ChangeTick, Local, Res, ResMut, Resource, SystemAccess, SystemParam, SystemParamFetch,
    World,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId<T> {
    pub id: usize,
    marker: PhantomData<T>,
}

impl<T> Copy for EventId<T> {}

impl<T> Clone for EventId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> std::fmt::Debug for EventId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "event<{}>({})", type_name::<T>(), self.id)
    }
}

#[derive(Debug)]
pub struct EventInstance<T> {
    pub event_id: EventId<T>,
    pub event: T,
}

#[derive(Debug)]
pub enum State {
    A,
    B,
}

#[derive(Debug)]
pub struct Events<T> {
    events_a: Vec<EventInstance<T>>,
    events_b: Vec<EventInstance<T>>,
    a_start_events_count: usize,
    b_start_events_count: usize,
    event_count: usize,
    state: State,
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self {
            events_a: Vec::new(),
            events_b: Vec::new(),
            a_start_events_count: 0,
            b_start_events_count: 0,
            event_count: 0,
            state: State::A,
        }
    }
}

impl<T: Resource> Events<T> {
    pub fn send(&mut self, event: T) {
        let event_id = EventId {
            id: self.event_count,
            marker: PhantomData,
        };

        let event_instance = EventInstance { event_id, event };

        match self.state {
            State::A => self.events_a.push(event_instance),
            State::B => self.events_b.push(event_instance),
        }

        self.event_count += 1;
    }

    pub fn update(&mut self) {
        match self.state {
            State::A => {
                self.events_b.clear();
                self.state = State::B;
                self.b_start_events_count = self.event_count;
            }
            State::B => {
                self.events_a.clear();
                self.state = State::A;
                self.a_start_events_count = self.event_count;
            }
        }
    }

    pub fn update_system(mut events: ResMut<Self>) {
        events.update();
    }
}

pub struct EventWriter<'w, T: Resource> {
    events: ResMut<'w, Events<T>>,
}

impl<'w, T: Resource> EventWriter<'w, T> {
    pub fn send(&mut self, event: T) {
        self.events.send(event);
    }
}

impl<'w, T: Resource> SystemParam for EventWriter<'w, T> {
    type Fetch = EventWriterFetch<'w, T>;
}

pub struct EventWriterFetch<'w, T: Resource> {
    events: <ResMut<'w, Events<T>> as SystemParam>::Fetch,
}

impl<'w0, 'w, 's, T: Resource> SystemParamFetch<'w, 's> for EventWriterFetch<'w0, T> {
    type Item = EventWriter<'w, T>;

    fn init(world: &mut World) -> Self {
        Self {
            events: SystemParamFetch::init(world),
        }
    }

    fn access(access: &mut SystemAccess) {
        access.borrow_resource::<Events<T>>(Access::Write);
    }

    fn get(&'s mut self, world: &'w World, last_change_tick: ChangeTick) -> Self::Item {
        EventWriter {
            events: SystemParamFetch::get(&mut self.events, world, last_change_tick),
        }
    }
}

pub struct EventReader<'w, 's, T: Resource> {
    last_event_count: Local<'s, usize>,
    event: Res<'w, Events<T>>,
}

fn internal_event_reader<'a, T>(
    last_event_count: &'a mut usize,
    events: &'a Events<T>,
) -> impl DoubleEndedIterator<Item = (&'a T, EventId<T>)> {
    let a_index = last_event_count.saturating_sub(events.a_start_events_count);
    let b_index = last_event_count.saturating_sub(events.b_start_events_count);

    let a = events.events_a.get(a_index..).unwrap_or_default();
    let b = events.events_b.get(b_index..).unwrap_or_default();

    let unread_count = a.len() + b.len();
    *last_event_count = events.event_count - unread_count;

    let iterator = match events.state {
        State::A => b.iter().chain(a),
        State::B => a.iter().chain(b),
    };

    iterator
        .map(|event| (&event.event, event.event_id))
        .inspect(move |(_, id)| *last_event_count = usize::max(id.id + 1, *last_event_count))
}

impl<'w, 's, T: Resource> EventReader<'w, 's, T> {
    pub fn iter(&mut self) -> impl DoubleEndedIterator<Item = &T> {
        self.iter_with_ids().map(|(event, _)| event)
    }

    pub fn iter_with_ids(&mut self) -> impl DoubleEndedIterator<Item = (&T, EventId<T>)> {
        internal_event_reader(&mut self.last_event_count, &self.event)
    }
}

impl<'w, 's, T: Resource> SystemParam for EventReader<'w, 's, T> {
    type Fetch = EventReaderFetch<'w, 's, T>;
}

pub struct EventReaderFetch<'w, 's, T: Resource> {
    last_event_count: <Local<'s, usize> as SystemParam>::Fetch,
    event: <Res<'w, Events<T>> as SystemParam>::Fetch,
}

impl<'w0, 's0, 'w, 's, T: Resource> SystemParamFetch<'w, 's> for EventReaderFetch<'w0, 's0, T> {
    type Item = EventReader<'w, 's, T>;

    fn init(world: &mut World) -> Self {
        Self {
            last_event_count: SystemParamFetch::init(world),
            event: SystemParamFetch::init(world),
        }
    }

    fn access(access: &mut SystemAccess) {
        access.borrow_resource::<Events<T>>(Access::Read);
    }

    fn get(&'s mut self, world: &'w World, last_change_tick: ChangeTick) -> Self::Item {
        EventReader {
            last_event_count: SystemParamFetch::get(
                &mut self.last_event_count,
                world,
                last_change_tick,
            ),
            event: SystemParamFetch::get(&mut self.event, world, last_change_tick),
        }
    }
}
