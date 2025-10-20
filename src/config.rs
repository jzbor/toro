#[derive(serde::Serialize, serde::Deserialize, clap::Args, Debug)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Config {
    #[clap(flatten)]
    #[serde(default)]
    pub columns: ColumnSelector,

    #[clap(flatten)]
    #[serde(default)]
    pub git: GitConfig,

    #[clap(flatten)]
    #[serde(default)]
    pub view: ViewConfig,
}

#[derive(clap::Args, serde::Serialize, serde::Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "kebab-case", deny_unknown_fields, default)]
pub struct ColumnSelector {
    // only ever read/use the non-negated versions

    /// Show completion mark column
    #[clap(long = "col-completed")]
    pub completed: bool,

    /// Hide completion mark column
    #[clap(long = "no-col-completed", overrides_with = "completed")]
    #[serde(skip)]
    no_completed: bool,

    /// Show priority column
    #[clap(long = "col-priority")]
    pub priority: bool,

    /// Hide priority column
    #[clap(long = "no-col-priority", overrides_with = "priority")]
    #[serde(skip)]
    no_priority: bool,

    /// Show completion date column
    #[clap(long = "col-completion-date")]
    pub completion_date: bool,

    /// Hide completion date column
    #[clap(long = "no-col-completion-date", overrides_with = "completion_date")]
    #[serde(skip)]
    no_completion_date: bool,

    /// Show creation date column
    #[clap(long = "col-creation-date")]
    pub creation_date: bool,

    /// Hide creation date column
    #[clap(long = "no-col-creation-date", overrides_with = "creation_date")]
    #[serde(skip)]
    no_creation_date: bool,
}

#[derive(serde::Deserialize, serde::Serialize, clap::Args, Debug)]
#[serde(rename_all = "kebab-case", deny_unknown_fields, default)]
pub struct GitConfig {
    /// Automatically create git commit after changes
    #[clap(long)]
    pub auto_commit: bool,

    /// Do not automatically create git commit after changes
    #[clap(long, overrides_with = "auto_commit")]
    #[serde(skip)]
    no_auto_commit: bool,

    /// Automatically pull, rebase and push git repository after changes
    #[clap(long)]
    pub auto_sync: bool,

    /// Do not automatically pull, rebase and push git repository after changes
    #[clap(long, overrides_with = "auto_sync")]
    #[serde(skip)]
    no_auto_sync: bool,
}

#[derive(serde::Deserialize, serde::Serialize, clap::Args, Debug, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields, default)]
pub struct ViewConfig {
    /// Automatically create git commit after changes
    #[clap(long)]
    pub cal_command: Option<String>,
}


pub trait UpdatableConfig {
    fn update_with_cmdline(&mut self, other: &Self);
}


impl UpdatableConfig for Config {
    fn update_with_cmdline(&mut self, other: &Self) {
        self.columns.update_with_cmdline(&other.columns);
        self.git.update_with_cmdline(&other.git);
        self.view.update_with_cmdline(&other.view);
    }
}

impl UpdatableConfig for ColumnSelector {
    fn update_with_cmdline(&mut self, other: &Self) {
        if other.completed || other.no_completed {
            self.completed = other.completed
        }

        if other.priority || other.no_priority {
            self.priority = other.priority
        }

        if other.completion_date || other.no_completion_date {
            self.completion_date = other.completion_date
        }

        if other.creation_date || other.no_creation_date {
            self.creation_date = other.creation_date
        }
    }
}

impl GitConfig {
    pub fn update_with_cmdline(&mut self, other: &Self) {
        if other.auto_commit || other.no_auto_commit {
            self.auto_commit = other.auto_commit
        }

        if other.auto_sync || other.no_auto_sync {
            self.auto_sync = other.auto_sync
        }
    }
}

impl ViewConfig {
    pub fn update_with_cmdline(&mut self, other: &Self) {
        if let Some(other_cmd) = &other.cal_command {
            self.cal_command = Some(other_cmd.to_owned());
        }
    }
}


impl Default for Config {
    fn default() -> Self {
        Config {
            columns: ColumnSelector::default(),
            git: GitConfig::default(),
            view: ViewConfig::default(),
        }
    }
}

impl Default for ColumnSelector {
    fn default() -> Self {
        ColumnSelector {
            completed: true,
            no_completed: false,
            priority: true,
            no_priority: false,
            completion_date: true,
            no_completion_date: false,
            creation_date: true,
            no_creation_date: false
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        GitConfig {
            auto_commit: false,
            no_auto_commit: true,
            auto_sync: false,
            no_auto_sync: true,
        }
    }
}
