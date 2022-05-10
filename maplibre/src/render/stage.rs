use crate::define_label;
use crate::render::RenderState;
use downcast_rs::{impl_downcast, Downcast};
use std::collections::HashMap;
use std::fmt::Debug;

define_label!(StageLabel);
pub(crate) type BoxedStageLabel = Box<dyn StageLabel>;

pub struct System {}

pub struct SystemStage {
    systems: Vec<System>,
}

impl SystemStage {
    pub fn add_system(&mut self, system: impl Into<System>) {
        self.systems.push(system.into())
    }
}

impl Stage for SystemStage {
    fn run(&mut self, state: &mut RenderState) {
        for x in self.systems {}
    }
}

#[derive(Default)]
pub struct RunCriteria {
    should_run: ShouldRun,
}

impl RunCriteria {
    pub fn should_run(&mut self) -> ShouldRun {
        self.should_run
    }

    pub fn set_should_run(&mut self, should_run: ShouldRun) {
        self.should_run = should_run;
    }
}

/// The labels of the default App rendering stages.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RenderStage {
    /// FIXME Extract data from the "app world" and insert it into the "render world".
    /// This step should be kept as short as possible to increase the "pipelining potential" for
    /// running the next frame while rendering the current frame.
    Extract,

    /// Prepare render resources from the extracted data for the GPU.
    Prepare,

    /// Create [`BindGroups`](crate::render_resource::BindGroup) that depend on
    /// [`Prepare`](RenderStage::Prepare) data and queue up draw calls to run during the
    /// [`Render`](RenderStage::Render) stage.
    Queue,

    // TODO: This could probably be moved in favor of a system ordering abstraction in Render or Queue
    /// Sort the [`RenderPhases`](crate::render_phase::RenderPhase) here.
    PhaseSort,

    /// Actual rendering happens here.
    /// In most cases, only the render backend should insert resources here.
    Render,

    /// Cleanup render resources here.
    Cleanup,
}

impl StageLabel for RenderStage {
    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        Box::new(self.clone())
    }
}

pub trait Stage: Downcast + Send + Sync {
    /// Runs the stage; this happens once per update.
    /// Implementors must initialize all of their state and systems before running the first time.
    fn run(&mut self, state: &mut RenderState);
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
    run_criteria: RunCriteria,
}

impl Schedule {
    /// Adds the given `stage` at the last position of the schedule.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::render::stage::{Schedule, SystemStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// schedule.add_stage("my_stage", SystemStage::parallel());
    /// ```
    pub fn add_stage<S: Stage>(&mut self, label: impl StageLabel, stage: S) -> &mut Self {
        let label: Box<dyn StageLabel> = Box::new(label);
        self.stage_order.push(label.clone());
        let prev = self.stages.insert(label.clone(), Box::new(stage));
        assert!(prev.is_none(), "Stage already exists: {:?}.", label);
        self
    }

    /// Adds the given `stage` immediately after the `target` stage.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::render::stage::{Schedule, SystemStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("target_stage", SystemStage::parallel());
    /// schedule.add_stage_after("target_stage", "my_stage", SystemStage::parallel());
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
            .unwrap_or_else(|| panic!("Target stage does not exist: {:?}.", target));

        self.stage_order.insert(target_index + 1, label.clone());
        let prev = self.stages.insert(label.clone(), Box::new(stage));
        assert!(prev.is_none(), "Stage already exists: {:?}.", label);
        self
    }

    /// Adds the given `stage` immediately before the `target` stage.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::render::stage::{Schedule, SystemStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("target_stage", SystemStage::parallel());
    /// #
    /// schedule.add_stage_before("target_stage", "my_stage", SystemStage::parallel());
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
            .unwrap_or_else(|| panic!("Target stage does not exist: {:?}.", target));

        self.stage_order.insert(target_index, label.clone());
        let prev = self.stages.insert(label.clone(), Box::new(stage));
        assert!(prev.is_none(), "Stage already exists: {:?}.", label);
        self
    }

    /// Adds the given `system` to the stage identified by `stage_label`.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::render::stage::{Schedule, SystemStage};
    /// #
    /// # fn my_system() {}
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("my_stage", SystemStage::parallel());
    /// #
    /// schedule.add_system_to_stage("my_stage", my_system);
    /// ```
    pub fn add_system_to_stage(
        &mut self,
        stage_label: impl StageLabel,
        system: impl Into<System>,
    ) -> &mut Self {
        // Use a function instead of a closure to ensure that it is codegend instead
        // of the game. Closures inherit generic parameters from their enclosing function.
        #[cold]
        fn stage_not_found(stage_label: &dyn Debug) -> ! {
            panic!(
                "Stage '{:?}' does not exist or is not a SystemStage",
                stage_label
            )
        }

        let stage = self
            .get_stage_mut::<SystemStage>(&stage_label)
            .unwrap_or_else(move || stage_not_found(&stage_label));
        stage.add_system(system);
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
    /// # use maplibre::render::stage::{Schedule, SystemStage};
    /// # let mut schedule = Schedule::default();
    ///
    /// # schedule.add_stage("my_stage", SystemStage::parallel());
    /// #
    /// schedule.stage("my_stage", |stage: &mut SystemStage| {
    ///     stage.add_system(my_system)
    /// });
    /// #
    /// # fn my_system() {}
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
            panic!("stage '{:?}' does not exist or is the wrong type", label)
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
    /// # use maplibre::render::stage::{Schedule, SystemStage};
    /// #
    /// # fn my_system() {}
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("my_stage", SystemStage::parallel());
    /// #
    /// let stage = schedule.get_stage::<SystemStage>(&"my_stage").unwrap();
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
    /// # use maplibre::render::stage::{Schedule, SystemStage};
    /// #
    /// # fn my_system() {}
    /// # let mut schedule = Schedule::default();
    /// # schedule.add_stage("my_stage", SystemStage::parallel());
    /// #
    /// let stage = schedule.get_stage_mut::<SystemStage>(&"my_stage").unwrap();
    /// ```
    pub fn get_stage_mut<T: Stage>(&mut self, label: &dyn StageLabel) -> Option<&mut T> {
        self.stages
            .get_mut(label)
            .and_then(|stage| stage.downcast_mut::<T>())
    }

    /// Executes each [`Stage`] contained in the schedule, one at a time.
    pub fn run_once(&mut self, state: &mut RenderState) {
        for label in &self.stage_order {
            #[cfg(feature = "trace")]
            let _stage_span = tracing::info_span!("stage", name = ?label).entered();
            let stage = self.stages.get_mut(label).unwrap();
            stage.run(state);
        }
    }

    /// Iterates over all of schedule's stages and their labels, in execution order.
    pub fn iter_stages(&self) -> impl Iterator<Item = (&dyn StageLabel, &dyn Stage)> {
        self.stage_order
            .iter()
            .map(move |label| (&**label, &*self.stages[label]))
    }
    pub fn run_criteria_mut(&mut self) -> &mut RunCriteria {
        &mut self.run_criteria
    }
}

/// Determines whether a system should be executed or not, and how many times it should be ran each
/// time the stage is executed.
///
/// A stage will loop over its run criteria and systems until no more systems need to be executed
/// and no more run criteria need to be checked.
/// - Any systems with run criteria that returns [`Yes`] will be ran exactly one more time during
///   the stage's execution that tick.
/// - Any systems with run criteria that returns [`No`] are not ran for the rest of the stage's
///   execution that tick.
/// - Any systems with run criteria that returns [`YesAndCheckAgain`] will be ran during this
///   iteration of the loop. After all the systems that need to run are ran, that criteria will be
///   checked again.
/// - Any systems with run criteria that returns [`NoAndCheckAgain`] will not be ran during this
///   iteration of the loop. After all the systems that need to run are ran, that criteria will be
///   checked again.
///
/// [`Yes`]: ShouldRun::Yes
/// [`No`]: ShouldRun::No
/// [`YesAndCheckAgain`]: ShouldRun::YesAndCheckAgain
/// [`NoAndCheckAgain`]: ShouldRun::NoAndCheckAgain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShouldRun {
    /// Yes, the system should run one more time this tick.
    Yes,
    /// No, the system should not run for the rest of this tick.
    No,
    /// Yes, the system should run, and after all systems in this stage have run, the criteria
    /// should be checked again. This will cause the stage to loop over the remaining systems and
    /// criteria this tick until they no longer need to be checked.
    YesAndCheckAgain,
    /// No, the system should not run right now, but after all systems in this stage have run, the
    /// criteria should be checked again. This will cause the stage to loop over the remaining
    /// systems and criteria this tick until they no longer need to be checked.
    NoAndCheckAgain,
}

impl Default for ShouldRun {
    fn default() -> Self {
        ShouldRun::No
    }
}

impl Stage for Schedule {
    fn run(&mut self, state: &mut RenderState) {
        loop {
            match self.run_criteria.should_run() {
                ShouldRun::No => return,
                ShouldRun::Yes => {
                    self.run_once(state);
                    return;
                }
                ShouldRun::YesAndCheckAgain => {
                    self.run_once(state);
                }
                ShouldRun::NoAndCheckAgain => {
                    panic!("`NoAndCheckAgain` would loop infinitely in this situation.")
                }
            }
        }
    }
}
