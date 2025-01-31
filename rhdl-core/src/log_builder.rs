use crate::{clock_details::ClockDetails, digital::Digital, tag_id::TagID};

pub trait LogBuilder {
    type SubBuilder: LogBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder;
    fn tag<T: Digital>(&mut self, name: &str) -> TagID<T>;
    fn allocate<T: Digital>(&self, tag: TagID<T>, width: usize);
    fn namespace(&self, name: &str) -> Self::SubBuilder;
    fn add_clock(&mut self, clock: ClockDetails);
    fn add_simple_clock(&mut self, period_in_fs: u64) {
        self.add_clock(ClockDetails {
            name: "clock".to_string(),
            period_in_fs,
            offset_in_fs: 0,
            initial_state: false,
        });
    }
}

impl<T: LogBuilder> LogBuilder for &mut T {
    type SubBuilder = T::SubBuilder;
    fn scope(&self, name: &str) -> Self::SubBuilder {
        (**self).scope(name)
    }
    fn tag<S: Digital>(&mut self, name: &str) -> TagID<S> {
        (**self).tag(name)
    }
    fn allocate<S: Digital>(&self, tag: TagID<S>, width: usize) {
        (**self).allocate(tag, width)
    }
    fn namespace(&self, name: &str) -> Self::SubBuilder {
        (**self).namespace(name)
    }
    fn add_clock(&mut self, clock: ClockDetails) {
        (**self).add_clock(clock)
    }
}
