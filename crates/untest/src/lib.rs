pub use ::inventory;
pub use ::untest_macro::untest;
#[cfg(feature = "libtest-mimic")]
pub use ::libtest_mimic;

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct CaseError {
    pub message: String,
}

pub type CaseResult = Result<(), CaseError>;

pub trait IntoCaseResult {
    fn into_case_result(self) -> CaseResult;
}

impl IntoCaseResult for () {
    fn into_case_result(self) -> CaseResult {
        Ok(())
    }
}

impl<E> IntoCaseResult for Result<(), E>
where
    E: std::fmt::Display,
{
    fn into_case_result(self) -> CaseResult {
        self.map_err(|err| CaseError {
            message: err.to_string(),
        })
    }
}

pub trait Case: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn run(&self) -> CaseResult;
}

// ---- Static ----

pub struct StaticTrial {
    pub name: &'static str,
    pub runner: fn() -> CaseResult,
}

impl StaticTrial {
    pub const fn new(name: &'static str, runner: fn() -> CaseResult) -> Self {
        Self { name, runner }
    }
}

impl Case for StaticTrial {
    fn name(&self) -> &str {
        self.name
    }

    fn run(&self) -> CaseResult {
        (self.runner)()
    }
}

// ---- Dynamic ----

pub struct DynamicTrial {
    pub name: String,
    #[allow(clippy::type_complexity)]
    pub runner: Box<dyn Fn() -> CaseResult + Send + Sync>,
}

impl Case for DynamicTrial {
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&self) -> CaseResult {
        (self.runner)()
    }
}

// ---- Unified ----

pub enum Trial {
    Static(&'static StaticTrial),
    Dynamic(DynamicTrial),
}

impl Case for Trial {
    fn name(&self) -> &str {
        match self {
            Trial::Static(t) => t.name(),
            Trial::Dynamic(t) => t.name(),
        }
    }

    fn run(&self) -> CaseResult {
        match self {
            Trial::Static(t) => t.run(),
            Trial::Dynamic(t) => t.run(),
        }
    }
}

pub struct TrialFactory(pub fn() -> Trial);

inventory::collect!(TrialFactory);

pub fn iter_trials() -> impl Iterator<Item = Trial> {
    inventory::iter::<TrialFactory>
        .into_iter()
        .map(|factory| (factory.0)())
}

// ---- Conversions ----

#[cfg(feature = "libtest-mimic")]
impl From<Trial> for libtest_mimic::Trial {
    fn from(trial: Trial) -> Self {
        let name = trial.name().to_string();
        libtest_mimic::Trial::test(name, move || {
            trial.run().map_err(|err| err.to_string().into())
        })
    }
}

#[cfg(feature = "libtest-mimic")]
impl From<DynamicTrial> for libtest_mimic::Trial {
    fn from(trial: DynamicTrial) -> Self {
        libtest_mimic::Trial::from(Trial::Dynamic(trial))
    }
}

#[cfg(feature = "libtest-mimic")]
impl From<&'static StaticTrial> for libtest_mimic::Trial {
    fn from(trial: &'static StaticTrial) -> Self {
        libtest_mimic::Trial::from(Trial::Static(trial))
    }
}

#[cfg(feature = "libtest-mimic")]
pub fn run_with_libtest_mimic<I>(
    args: &libtest_mimic::Arguments,
    trials: I,
) -> libtest_mimic::Conclusion
where
    I: IntoIterator,
    I::Item: Into<libtest_mimic::Trial>,
{
    let collected: Vec<_> = trials.into_iter().map(Into::into).collect();
    libtest_mimic::run(args, collected)
}

#[cfg(feature = "libtest-mimic")]
pub fn run_inventory_with_libtest_mimic(
    args: &libtest_mimic::Arguments,
) -> libtest_mimic::Conclusion {
    run_with_libtest_mimic(args, iter_trials())
}
