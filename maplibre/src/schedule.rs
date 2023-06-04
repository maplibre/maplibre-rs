use std::collections::HashMap;

use downcast_rs::{impl_downcast, Downcast};

use crate::{
    context::MapContext,
    define_label,
    tcs::system::{stage::SystemStage, IntoSystemContainer},
};

pub struct NopStage;

impl Stage for NopStage {
    fn run(&mut self, _context: &mut MapContext) {}
}

#[macro_export]
macro_rules! multi_stage {
    ($multi_stage:ident, $($stage:ident: $stage_ty:ty),*) => {
        pub struct $multi_stage {
            $($stage: $stage_ty),*
        }

        impl Stage for $multi_stage {
            fn run(&mut self, context: &mut $crate::context::MapContext) {
                 $(self.$stage.run(context);)*
            }
        }

        impl Default for $multi_stage {
            fn default() -> Self {
                $multi_stage {
                     $($stage: <$stage_ty>::default()),*
                }
            }
        }
    };
}

pub struct MultiStage<const I: usize, S>
where
    S: Stage,
{
    stages: [S; I],
}

impl<const I: usize, S> MultiStage<I, S>
where
    S: Stage,
{
    pub fn new(stages: [S; I]) -> Self {
        Self { stages }
    }
}

impl<const I: usize, S> Stage for MultiStage<I, S>
where
    S: Stage,
{
    fn run(&mut self, context: &mut MapContext) {
        for stage in self.stages.iter_mut() {
            stage.run(context)
        }
    }
}

define_label!(StageLabel);
pub(crate) type BoxedStageLabel = Box<dyn StageLabel>;

pub trait Stage: Downcast {
    /// Runs the stage; this happens once per update.
    /// Implementors must initialize all of their state before running the first time.
    fn run(&mut self, context: &mut MapContext);
}

impl_downcast!(Stage);

/// A container of [`Stage`]s set to be run in a linear order.
///
/// Since `Schedule` implements the [`Stage`] trait, it can be inserted into another schedule.
/// In this way, the properties of the child schedule can be set differently from the parent.
/// For example, it can be set to run only once during app execution, while the parent schedule
/// runs indefinitely.
#[derive(Default)]
pub struct Schedule {
    stages: HashMap<BoxedStageLabel, Box<dyn Stage>>,
    stage_order: Vec<BoxedStageLabel>,
}

impl Schedule {
    /// Adds the given `stage` at the last position of the schedule.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::schedule::{Schedule, NopStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// schedule.add_stage("my_stage", NopStage);
    /// ```
    pub fn add_stage<S: Stage>(&mut self, label: impl StageLabel, stage: S) -> &mut Self {
        let label: Box<dyn StageLabel> = Box::new(label);
        self.stage_order.push(label.clone());
        let prev = self.stages.insert(label.clone(), Box::new(stage));
        assert!(prev.is_none(), "Stage already exists: {label:?}.");
        self
    }

    pub fn remove_stage(&mut self, label: impl StageLabel) -> &mut Self {
        let remove: Box<dyn StageLabel> = Box::new(label);
        self.stages.remove(&remove).expect("stage not found");
        self.stage_order.retain(|label| label != &remove);
        self
    }

    /// Adds the given `stage` immediately after the `target` stage.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::schedule::{Schedule, NopStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("target_stage", NopStage);
    /// schedule.add_stage_after("target_stage", "my_stage", NopStage);
    /// ```
    pub fn add_stage_after<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        let label: Box<dyn StageLabel> = Box::new(label);
        let target = &target as &dyn StageLabel;
        let target_index = self
            .stage_order
            .iter()
            .enumerate()
            .find(|(_i, stage_label)| &***stage_label == target)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("Target stage does not exist: {target:?}."));

        self.stage_order.insert(target_index + 1, label.clone());
        let prev = self.stages.insert(label.clone(), Box::new(stage));
        assert!(prev.is_none(), "Stage already exists: {label:?}.");
        self
    }

    /// Adds the given `stage` immediately before the `target` stage.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::schedule::{Schedule, NopStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("target_stage", NopStage);
    /// #
    /// schedule.add_stage_before("target_stage", "my_stage", NopStage);
    /// ```
    pub fn add_stage_before<S: Stage>(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: S,
    ) -> &mut Self {
        let label: Box<dyn StageLabel> = Box::new(label);
        let target = &target as &dyn StageLabel;
        let target_index = self
            .stage_order
            .iter()
            .enumerate()
            .find(|(_i, stage_label)| &***stage_label == target)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("Target stage does not exist: {target:?}."));

        self.stage_order.insert(target_index, label.clone());
        let prev = self.stages.insert(label.clone(), Box::new(stage));
        assert!(prev.is_none(), "Stage already exists: {label:?}.");
        self
    }

    /// Fetches the [`Stage`] of type `T` marked with `label`, then executes the provided
    /// `func` passing the fetched stage to it as an argument.
    ///
    /// The `func` argument should be a function or a closure that accepts a mutable reference
    /// to a struct implementing `Stage` and returns the same type. That means that it should
    /// also assume that the stage has already been fetched successfully.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::schedule::{Schedule, NopStage};
    /// # let mut schedule = Schedule::default();
    ///
    /// # schedule.add_stage("my_stage", NopStage);
    /// #
    /// schedule.stage("my_stage", |stage: &mut NopStage| {
    ///     // modify stage
    ///     stage
    /// });
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `label` refers to a non-existing stage, or if it's not of type `T`.
    pub fn stage<T: Stage, F: FnOnce(&mut T) -> &mut T>(
        &mut self,
        label: impl StageLabel,
        func: F,
    ) -> &mut Self {
        let stage = self.get_stage_mut::<T>(&label).unwrap_or_else(move || {
            panic!("stage '{label:?}' does not exist or is the wrong type")
        });
        func(stage);
        self
    }

    /// Returns a shared reference to the stage identified by `label`, if it exists.
    ///
    /// If the requested stage does not exist, `None` is returned instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::schedule::{Schedule, NopStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("my_stage", NopStage);
    /// #
    /// let stage = schedule.get_stage::<NopStage>(&"my_stage").unwrap();
    /// ```
    pub fn get_stage<T: Stage>(&self, label: &dyn StageLabel) -> Option<&T> {
        self.stages
            .get(label)
            .and_then(|stage| stage.downcast_ref::<T>())
    }

    /// Returns a unique, mutable reference to the stage identified by `label`, if it exists.
    ///
    /// If the requested stage does not exist, `None` is returned instead.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::schedule::{Schedule, NopStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("my_stage", NopStage);
    /// #
    /// let stage = schedule.get_stage_mut::<NopStage>(&"my_stage").unwrap();
    /// ```
    pub fn get_stage_mut<T: Stage>(&mut self, label: &dyn StageLabel) -> Option<&mut T> {
        self.stages
            .get_mut(label)
            .and_then(|stage| stage.downcast_mut::<T>())
    }

    /// Executes each [`Stage`] contained in the schedule, one at a time.
    pub fn run_once(&mut self, context: &mut MapContext) {
        for label in &self.stage_order {
            #[cfg(feature = "trace")]
            let _stage_span = tracing::info_span!("stage", name = ?label).entered();
            let stage = self.stages.get_mut(label).unwrap(); // TODO: Remove unwrap
            stage.run(context);
        }
    }

    /// Iterates over all of schedule's stages and their labels, in execution order.
    pub fn iter_stages(&self) -> impl Iterator<Item = (&dyn StageLabel, &dyn Stage)> {
        self.stage_order
            .iter()
            .map(move |label| (&**label, &*self.stages[label]))
    }

    /// Adds a system to the [`Stage`] identified by `stage_label`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maplibre::context::MapContext;
    /// # use maplibre::tcs::system::stage::SystemStage;
    /// # use maplibre::schedule::{Schedule, NopStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("my_stage", SystemStage::default());
    /// # fn my_system(context: &mut MapContext) {}
    /// #
    /// schedule.add_system_to_stage("my_stage", my_system);
    /// ```
    pub fn add_system_to_stage(
        &mut self,
        stage_label: impl StageLabel,
        system: impl IntoSystemContainer,
    ) -> &mut Self {
        let stage = self
            .get_stage_mut::<SystemStage>(&stage_label)
            .unwrap_or_else(move || {
                panic!("Stage '{stage_label:?}' does not exist or is not a SystemStage")
            });
        stage.add_system(system);
        self
    }
}

impl Stage for Schedule {
    fn run(&mut self, context: &mut MapContext) {
        self.run_once(context);
    }
}
