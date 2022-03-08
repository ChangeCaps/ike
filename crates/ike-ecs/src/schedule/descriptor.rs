use crate::{
    ExclusiveSystem, ExclusiveSystemDescriptor, ExclusiveSystemFn, IntoSystem, ParallelSystem,
    SystemLabel, SystemParam, World,
};

pub enum SystemDescriptor {
    Parallel(ParallelSystem),
    Exclusive(ExclusiveSystemDescriptor),
}

pub trait IntoSystemDescriptor<Params> {
    fn into_descriptor(self) -> SystemDescriptor;
}

impl<Params, F> IntoSystemDescriptor<Params> for F
where
    F: IntoSystem<Params>,
{
    fn into_descriptor(self) -> SystemDescriptor {
        SystemDescriptor::Parallel(ParallelSystem::new(self.system()))
    }
}

impl<F> IntoSystemDescriptor<()> for ExclusiveSystemFn<F>
where
    F: FnMut(&mut World) + Send + Sync + 'static,
{
    fn into_descriptor(self) -> SystemDescriptor {
        SystemDescriptor::Exclusive(ExclusiveSystemDescriptor::new(self))
    }
}

impl IntoSystemDescriptor<()> for SystemDescriptor {
    fn into_descriptor(self) -> SystemDescriptor {
        self
    }
}

impl IntoSystemDescriptor<()> for ParallelSystem {
    fn into_descriptor(self) -> SystemDescriptor {
        SystemDescriptor::Parallel(self)
    }
}

pub trait ParallelSystemCoercion<Params> {
    fn label(self, label: impl SystemLabel) -> ParallelSystem;

    fn before(self, label: impl SystemLabel) -> ParallelSystem;

    fn after(self, label: impl SystemLabel) -> ParallelSystem;
}

impl ParallelSystemCoercion<()> for ParallelSystem {
    fn label(mut self, label: impl SystemLabel) -> ParallelSystem {
        self.labels.push(label.raw_label());
        self
    }

    fn before(mut self, label: impl SystemLabel) -> ParallelSystem {
        self.before.push(label.raw_label());
        self
    }

    fn after(mut self, label: impl SystemLabel) -> ParallelSystem {
        self.after.push(label.raw_label());
        self
    }
}

impl<Params, F> ParallelSystemCoercion<Params> for F
where
    F: IntoSystem<Params>,
    Params: SystemParam,
{
    fn label(self, label: impl SystemLabel) -> ParallelSystem {
        ParallelSystem::new(self.system()).label(label)
    }

    fn before(self, label: impl SystemLabel) -> ParallelSystem {
        ParallelSystem::new(self.system()).before(label)
    }

    fn after(self, label: impl SystemLabel) -> ParallelSystem {
        ParallelSystem::new(self.system()).after(label)
    }
}

pub trait ExclusiveSystemCoercion {
    fn label(self, label: impl SystemLabel) -> ExclusiveSystemDescriptor;

    fn before(self, label: impl SystemLabel) -> ExclusiveSystemDescriptor;

    fn after(self, label: impl SystemLabel) -> ExclusiveSystemDescriptor;
}

impl ExclusiveSystemCoercion for ExclusiveSystemDescriptor {
    fn label(mut self, label: impl SystemLabel) -> ExclusiveSystemDescriptor {
        self.labels.push(label.raw_label());
        self
    }

    fn before(mut self, label: impl SystemLabel) -> ExclusiveSystemDescriptor {
        self.before.push(label.raw_label());
        self
    }

    fn after(mut self, label: impl SystemLabel) -> ExclusiveSystemDescriptor {
        self.after.push(label.raw_label());
        self
    }
}

impl<S: ExclusiveSystem> ExclusiveSystemCoercion for S {
    fn label(self, label: impl SystemLabel) -> ExclusiveSystemDescriptor {
        ExclusiveSystemDescriptor::new(self).label(label)
    }

    fn before(self, label: impl SystemLabel) -> ExclusiveSystemDescriptor {
        ExclusiveSystemDescriptor::new(self).before(label)
    }

    fn after(self, label: impl SystemLabel) -> ExclusiveSystemDescriptor {
        ExclusiveSystemDescriptor::new(self).after(label)
    }
}
