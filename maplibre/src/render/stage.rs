use crate::render::RenderState;
use crate::{define_label, Renderer};
use downcast_rs::{impl_downcast, Downcast};
use std::collections::HashMap;
use std::fmt::Debug;

pub struct NopStage;

impl Stage for NopStage {
    fn run(&mut self, _renderer: &Renderer, _state: &mut RenderState) {}
}

define_label!(StageLabel);
pub(crate) type BoxedStageLabel = Box<dyn StageLabel>;

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

    // FIXME: TODO: This could probably be moved in favor of a system ordering abstraction in Render or Queue
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
    /// Implementors must initialize all of their state before running the first time.
    fn run(&mut self, renderer: &Renderer, state: &mut RenderState);
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
    stages: HashMap<BoxedStageLabel, Box<dyn Stage>>, // FIXME Is this the archetype pattern?
    stage_order: Vec<BoxedStageLabel>,
    run_criteria: RunCriteria,
}

impl Schedule {
    /// Adds the given `stage` at the last position of the schedule.
    ///
    /// # Example
    ///
    /// ```
    /// # use maplibre::render::stage::{Schedule, NopStage};
    /// #
    /// # let mut schedule = Schedule::default();
    /// schedule.add_stage("my_stage", NopStage);
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
    /// # use maplibre::render::stage::{Schedule, NopStage};
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
    /// # use maplibre::render::stage::{Schedule, NopStage};
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
            .unwrap_or_else(|| panic!("Target stage does not exist: {:?}.", target));

        self.stage_order.insert(target_index, label.clone());
        let prev = self.stages.insert(label.clone(), Box::new(stage));
        assert!(prev.is_none(), "Stage already exists: {:?}.", label);
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
    /// # use maplibre::render::stage::{Schedule, NopStage};
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
    /// # use maplibre::render::stage::{Schedule, NopStage};
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
    /// # use maplibre::render::stage::{Schedule, NopStage};
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
    pub fn run_once(&mut self, renderer: &Renderer, state: &mut RenderState) {
        for label in &self.stage_order {
            #[cfg(feature = "trace")]
            let _stage_span = tracing::info_span!("stage", name = ?label).entered();
            let stage = self.stages.get_mut(label).unwrap();
            stage.run(renderer, state);
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
/// - FIXME: Any systems with run criteria that returns [`Yes`] will be ran exactly one more time during
///   the stage's execution that tick.
/// - FIXME: Any systems with run criteria that returns [`No`] are not ran for the rest of the stage's
///   execution that tick.
/// - FIXME: Any systems with run criteria that returns [`YesAndCheckAgain`] will be ran during this
///   iteration of the loop. After all the systems that need to run are ran, that criteria will be
///   checked again.
/// - FIXME: Any systems with run criteria that returns [`NoAndCheckAgain`] will not be ran during this
///   iteration of the loop. After all the systems that need to run are ran, that criteria will be
///   checked again.
///
/// [`Yes`]: ShouldRun::Yes
/// [`No`]: ShouldRun::No
/// [`YesAndCheckAgain`]: ShouldRun::YesAndCheckAgain
/// [`NoAndCheckAgain`]: ShouldRun::NoAndCheckAgain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShouldRun {
    /// FIXME: Yes, the system should run one more time this tick.
    Yes,
    /// FIXME: No, the system should not run for the rest of this tick.
    No,
    /// FIXME: Yes, the system should run, and after all systems in this stage have run, the criteria
    /// should be checked again. This will cause the stage to loop over the remaining systems and
    /// criteria this tick until they no longer need to be checked.
    YesAndCheckAgain,
    /// FIXME: No, the system should not run right now, but after all systems in this stage have run, the
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
    fn run(&mut self, renderer: &Renderer, state: &mut RenderState) {
        loop {
            match self.run_criteria.should_run() {
                ShouldRun::No => return,
                ShouldRun::Yes => {
                    self.run_once(renderer, state);
                    return;
                }
                ShouldRun::YesAndCheckAgain => {
                    self.run_once(renderer, state);
                }
                ShouldRun::NoAndCheckAgain => {
                    panic!("`NoAndCheckAgain` would loop infinitely in this situation.")
                }
            }
        }
    }
}
