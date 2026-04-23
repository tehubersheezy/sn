pub mod aggregate;
pub mod app;
pub mod atf;
pub mod attachment;
pub mod auth;
pub mod catalog;
pub mod change;
pub mod cmdb;
pub mod identify;
pub mod import;
pub mod init;
pub mod introspect;
pub mod profile;
pub mod progress;
pub mod schema;
pub mod scores;
pub mod table;
pub mod update_set;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "sn",
    version,
    about = "ServiceNow CLI (Table API + schema + CICD)"
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalFlags,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct GlobalFlags {
    /// Profile name (overrides SN_PROFILE and default_profile).
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Override the profile's instance URL for this invocation.
    #[arg(long, global = true, value_name = "URL")]
    pub instance_override: Option<String>,

    /// Output mode. `default` (unwrapped result) or `raw` (full SN envelope).
    #[arg(long, global = true, value_enum, default_value_t = OutputMode::Default)]
    pub output: OutputMode,

    /// Force pretty-printed JSON regardless of TTY detection.
    #[arg(long, global = true, conflicts_with = "compact")]
    pub pretty: bool,

    /// Force compact JSON regardless of TTY detection.
    #[arg(long, global = true, conflicts_with = "pretty")]
    pub compact: bool,

    /// Request timeout in seconds (overrides SN_TIMEOUT).
    #[arg(long, global = true, value_name = "SECS")]
    pub timeout: Option<u64>,

    /// Verbosity: -v, -vv, -vvv (see spec §9).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Proxy URL (http://, https://, socks5://). Overrides SN_PROXY and profile config.
    #[arg(long, global = true, value_name = "URL")]
    pub proxy: Option<String>,

    /// Bypass any configured proxy for this invocation.
    #[arg(long, global = true)]
    pub no_proxy: bool,

    /// Custom CA certificate for the proxy connection.
    #[arg(long, global = true, value_name = "PATH")]
    pub proxy_ca_cert: Option<String>,

    /// Disable TLS certificate verification (DANGEROUS).
    #[arg(long, global = true)]
    pub insecure: bool,

    /// Custom CA certificate bundle for the ServiceNow endpoint.
    #[arg(long, global = true, value_name = "PATH")]
    pub ca_cert: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq, Default)]
#[value(rename_all = "lowercase")]
pub enum OutputMode {
    #[default]
    Default,
    Raw,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create or update a profile interactively.
    Init(InitArgs),
    /// Auth operations.
    Auth {
        #[command(subcommand)]
        sub: AuthSub,
    },
    /// Manage profiles.
    Profile {
        #[command(subcommand)]
        sub: ProfileSub,
    },
    /// Table API CRUD.
    Table {
        #[command(subcommand)]
        sub: TableSub,
    },
    /// Schema discovery.
    Schema {
        #[command(subcommand)]
        sub: SchemaSub,
    },
    /// Dump the full command tree as JSON for agent/MCP generation.
    Introspect,
    /// Get pipeline/deployment progress by ID.
    Progress(ProgressArgs),
    /// App repository operations (install, publish, rollback).
    App {
        #[command(subcommand)]
        sub: AppSub,
    },
    /// Update Set lifecycle operations.
    #[command(name = "updateset")]
    UpdateSet {
        #[command(subcommand)]
        sub: UpdateSetSub,
    },
    /// Aggregate statistics for a table (GET /api/now/stats/{table}).
    Aggregate(AggregateArgs),
    /// Performance Analytics scorecard operations.
    Scores {
        #[command(subcommand)]
        sub: ScoresSub,
    },
    /// Automated Test Framework operations.
    Atf {
        #[command(subcommand)]
        sub: AtfSub,
    },
    /// Change Management operations (normal, emergency, standard).
    Change {
        #[command(subcommand)]
        sub: ChangeSub,
    },
    /// Attachment operations (upload, download, list, delete).
    Attachment {
        #[command(subcommand)]
        sub: AttachmentSub,
    },
    /// CMDB Instance and Meta operations.
    Cmdb {
        #[command(subcommand)]
        sub: CmdbSub,
    },
    /// Import Set operations (staging table imports).
    Import {
        #[command(subcommand)]
        sub: ImportSub,
    },
    /// Service Catalog operations (catalogs, items, cart, ordering).
    Catalog {
        #[command(subcommand)]
        sub: CatalogSub,
    },
    /// Identification and Reconciliation (CI create/update/query).
    Identify {
        #[command(subcommand)]
        sub: IdentifySub,
    },
}

#[derive(clap::Args, Debug)]
pub struct InitArgs {
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub instance: Option<String>,
    #[arg(long)]
    pub username: Option<String>,
    #[arg(long)]
    pub password: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum AuthSub {
    /// Verify credentials by calling /api/now/table/sys_user?sysparm_limit=1.
    Test,
}

#[derive(Subcommand, Debug)]
pub enum ProfileSub {
    List,
    Show { name: Option<String> },
    Remove { name: String },
    Use { name: String },
}

#[derive(Subcommand, Debug)]
pub enum TableSub {
    #[command(about = "List records")]
    List(TableListArgs),
    #[command(about = "Get a single record by sys_id")]
    Get(TableGetArgs),
    #[command(about = "Create a record")]
    Create(TableCreateArgs),
    #[command(about = "Patch a record (partial update)")]
    Update(TableUpdateArgs),
    #[command(about = "Replace a record (PUT, full overwrite)")]
    Replace(TableReplaceArgs),
    #[command(about = "Delete a record")]
    Delete(TableDeleteArgs),
}

#[derive(clap::Args, Debug)]
pub struct TableListArgs {
    /// Table name (e.g. `incident`).
    pub table: String,
    /// Encoded query, e.g. `active=true^priority=1`.
    #[arg(long, alias = "sysparm-query")]
    pub query: Option<String>,
    /// Comma-separated fields to return.
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    /// Maximum records returned (default 1000). Maps to sysparm_limit.
    #[arg(
        long,
        alias = "limit",
        alias = "sysparm-limit",
        alias = "page-size",
        default_value_t = 1000
    )]
    pub setlimit: u32,
    /// Starting offset for manual pagination (ignored with --all).
    #[arg(long, alias = "sysparm-offset")]
    pub offset: Option<u32>,
    /// Resolve reference/choice fields: false (default), true, or all.
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    /// Strip reference-link URLs from reference fields.
    #[arg(long, alias = "sysparm-exclude-reference-link")]
    pub exclude_reference_link: bool,
    /// Skip X-Total-Count calculation.
    #[arg(long, alias = "sysparm-suppress-pagination-header")]
    pub suppress_pagination_header: bool,
    /// Apply a named form/list view.
    #[arg(long, alias = "sysparm-view")]
    pub view: Option<String>,
    /// Query category for index selection.
    #[arg(long, alias = "sysparm-query-category")]
    pub query_category: Option<String>,
    /// Cross-domain access if authorized.
    #[arg(long, alias = "sysparm-query-no-domain")]
    pub query_no_domain: bool,
    /// Skip the count query.
    #[arg(long, alias = "sysparm-no-count")]
    pub no_count: bool,
    /// Auto-paginate: stream every matching record (JSONL unless --array).
    #[arg(long)]
    pub all: bool,
    /// With --all, buffer into a single JSON array instead of JSONL.
    #[arg(long, requires = "all")]
    pub array: bool,
    /// Cap total records returned (default 100000; 0 = unlimited).
    #[arg(long, default_value_t = 100_000)]
    pub max_records: u32,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum DisplayValueArg {
    True,
    False,
    All,
}

impl From<DisplayValueArg> for crate::query::DisplayValue {
    fn from(v: DisplayValueArg) -> Self {
        match v {
            DisplayValueArg::True => crate::query::DisplayValue::True,
            DisplayValueArg::False => crate::query::DisplayValue::False,
            DisplayValueArg::All => crate::query::DisplayValue::All,
        }
    }
}
#[derive(clap::Args, Debug)]
pub struct TableGetArgs {
    pub table: String,
    pub sys_id: String,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    #[arg(long, alias = "sysparm-exclude-reference-link")]
    pub exclude_reference_link: bool,
    #[arg(long, alias = "sysparm-view")]
    pub view: Option<String>,
    #[arg(long, alias = "sysparm-query-no-domain")]
    pub query_no_domain: bool,
}
#[derive(clap::Args, Debug)]
pub struct TableCreateArgs {
    pub table: String,
    /// Inline JSON, @file, or @- for stdin.
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    /// Repeatable name=value. Mutually exclusive with --data.
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    #[arg(long, alias = "sysparm-exclude-reference-link")]
    pub exclude_reference_link: bool,
    #[arg(long, alias = "sysparm-input-display-value")]
    pub input_display_value: bool,
    #[arg(long, alias = "sysparm-suppress-auto-sys-field")]
    pub suppress_auto_sys_field: bool,
    #[arg(long, alias = "sysparm-view")]
    pub view: Option<String>,
}
#[derive(clap::Args, Debug)]
pub struct TableUpdateArgs {
    pub table: String,
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    #[arg(long, alias = "sysparm-exclude-reference-link")]
    pub exclude_reference_link: bool,
    #[arg(long, alias = "sysparm-input-display-value")]
    pub input_display_value: bool,
    #[arg(long, alias = "sysparm-suppress-auto-sys-field")]
    pub suppress_auto_sys_field: bool,
    #[arg(long, alias = "sysparm-view")]
    pub view: Option<String>,
    #[arg(long, alias = "sysparm-query-no-domain")]
    pub query_no_domain: bool,
}
#[derive(clap::Args, Debug)]
pub struct TableReplaceArgs {
    pub table: String,
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    #[arg(long, alias = "sysparm-exclude-reference-link")]
    pub exclude_reference_link: bool,
    #[arg(long, alias = "sysparm-input-display-value")]
    pub input_display_value: bool,
    #[arg(long, alias = "sysparm-suppress-auto-sys-field")]
    pub suppress_auto_sys_field: bool,
    #[arg(long, alias = "sysparm-view")]
    pub view: Option<String>,
    #[arg(long, alias = "sysparm-query-no-domain")]
    pub query_no_domain: bool,
}
#[derive(clap::Args, Debug)]
pub struct TableDeleteArgs {
    pub table: String,
    pub sys_id: String,
    /// Skip confirmation prompt (required for non-interactive use).
    #[arg(long, short = 'y')]
    pub yes: bool,
    #[arg(long, alias = "sysparm-query-no-domain")]
    pub query_no_domain: bool,
}

#[derive(Subcommand, Debug)]
pub enum SchemaSub {
    Tables(SchemaTablesArgs),
    Columns(SchemaColumnsArgs),
    Choices(SchemaChoicesArgs),
}

#[derive(clap::Args, Debug, Default)]
pub struct SchemaTablesArgs {
    #[arg(long)]
    pub filter: Option<String>,
    #[arg(long)]
    pub reference_only: bool,
}
#[derive(clap::Args, Debug)]
pub struct SchemaColumnsArgs {
    pub table: String,
    #[arg(long)]
    pub filter: Option<String>,
    #[arg(long, value_name = "TYPE")]
    pub r#type: Option<String>,
    #[arg(long)]
    pub mandatory: bool,
    #[arg(long)]
    pub writable: bool,
    #[arg(long)]
    pub choices_only: bool,
    #[arg(long)]
    pub references_only: bool,
}
#[derive(clap::Args, Debug, Default)]
pub struct SchemaChoicesArgs {
    pub table: String,
    pub field: String,
}

// ── Progress ─────────────────────────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct ProgressArgs {
    /// Progress ID returned by app/updateset/atf operations.
    pub progress_id: String,
}

// ── App repo ─────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum AppSub {
    /// Install an application from the app repository.
    Install(AppInstallArgs),
    /// Publish an application to the app repository.
    Publish(AppPublishArgs),
    /// Roll back an application to a previous version.
    Rollback(AppRollbackArgs),
}

#[derive(clap::Args, Debug)]
pub struct AppInstallArgs {
    /// sys_id of the application.
    #[arg(long)]
    pub sys_id: Option<String>,
    /// Application scope (e.g. `x_acme_myapp`).
    #[arg(long)]
    pub scope: Option<String>,
    /// Version to install.
    #[arg(long)]
    pub version: Option<String>,
    /// Automatically upgrade the base application if needed.
    #[arg(long)]
    pub auto_upgrade_base_app: bool,
    /// Version of the base application to use.
    #[arg(long)]
    pub base_app_version: Option<String>,
    /// Block until the operation completes (polls progress API).
    #[arg(long)]
    pub wait: bool,
}

#[derive(clap::Args, Debug)]
pub struct AppPublishArgs {
    /// sys_id of the application.
    #[arg(long)]
    pub sys_id: Option<String>,
    /// Application scope (e.g. `x_acme_myapp`).
    #[arg(long)]
    pub scope: Option<String>,
    /// Version to publish.
    #[arg(long)]
    pub version: Option<String>,
    /// Developer notes for this publish.
    #[arg(long)]
    pub dev_notes: Option<String>,
    /// Block until the operation completes (polls progress API).
    #[arg(long)]
    pub wait: bool,
}

#[derive(clap::Args, Debug)]
pub struct AppRollbackArgs {
    /// sys_id of the application.
    #[arg(long)]
    pub sys_id: Option<String>,
    /// Application scope (e.g. `x_acme_myapp`).
    #[arg(long)]
    pub scope: Option<String>,
    /// Version to roll back to (required).
    #[arg(long, required = true)]
    pub version: String,
    /// Block until the operation completes (polls progress API).
    #[arg(long)]
    pub wait: bool,
}

// ── Update Set ───────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum UpdateSetSub {
    /// Create a new Update Set.
    Create(UpdateSetCreateArgs),
    /// Retrieve a remote Update Set into this instance.
    Retrieve(UpdateSetRetrieveArgs),
    /// Preview a retrieved remote Update Set.
    Preview(UpdateSetIdArg),
    /// Commit a previewed remote Update Set.
    Commit(UpdateSetIdArg),
    /// Commit multiple remote Update Sets at once.
    CommitMultiple(UpdateSetCommitMultipleArgs),
    /// Back out (undo) an applied Update Set.
    BackOut(UpdateSetBackOutArgs),
}

#[derive(clap::Args, Debug)]
pub struct UpdateSetCreateArgs {
    /// Name for the new Update Set (required).
    #[arg(long, required = true)]
    pub name: String,
    /// Optional description.
    #[arg(long)]
    pub description: Option<String>,
    /// sys_id to assign to the new Update Set.
    #[arg(long)]
    pub sys_id: Option<String>,
    /// Application scope.
    #[arg(long)]
    pub scope: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct UpdateSetRetrieveArgs {
    /// sys_id of the Update Set to retrieve (required).
    #[arg(long, required = true)]
    pub update_set_id: String,
    /// sys_id of the source record.
    #[arg(long)]
    pub source_id: Option<String>,
    /// Instance ID of the source ServiceNow instance.
    #[arg(long)]
    pub source_instance_id: Option<String>,
    /// Automatically preview after retrieval.
    #[arg(long)]
    pub auto_preview: bool,
    /// Clean up retrieved set after preview/commit.
    #[arg(long)]
    pub cleanup_retrieved: bool,
    /// Block until the operation completes (polls progress API).
    #[arg(long)]
    pub wait: bool,
}

/// Shared arg struct for preview and commit (single path param).
#[derive(clap::Args, Debug)]
pub struct UpdateSetIdArg {
    /// Remote Update Set sys_id.
    pub remote_update_set_id: String,
    /// Block until the operation completes (polls progress API).
    #[arg(long)]
    pub wait: bool,
}

#[derive(clap::Args, Debug)]
pub struct UpdateSetCommitMultipleArgs {
    /// Comma-separated list of remote Update Set sys_ids.
    #[arg(long, required = true)]
    pub ids: String,
    /// Block until the operation completes (polls progress API).
    #[arg(long)]
    pub wait: bool,
}

#[derive(clap::Args, Debug)]
pub struct UpdateSetBackOutArgs {
    /// sys_id of the Update Set to back out (required).
    #[arg(long, required = true)]
    pub update_set_id: String,
    /// Also roll back any application installs included in the set.
    #[arg(long)]
    pub rollback_installs: bool,
    /// Block until the operation completes (polls progress API).
    #[arg(long)]
    pub wait: bool,
}

// ── ATF ──────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum AtfSub {
    /// Run an ATF test suite.
    Run(AtfRunArgs),
    /// Get results for an ATF test suite run.
    Results(AtfResultsArgs),
}

#[derive(clap::Args, Debug)]
pub struct AtfRunArgs {
    /// sys_id of the test suite.
    #[arg(long)]
    pub suite_id: Option<String>,
    /// Name of the test suite.
    #[arg(long)]
    pub suite_name: Option<String>,
    /// Browser name (e.g. `chrome`).
    #[arg(long)]
    pub browser_name: Option<String>,
    /// Browser version.
    #[arg(long)]
    pub browser_version: Option<String>,
    /// OS name.
    #[arg(long)]
    pub os_name: Option<String>,
    /// OS version.
    #[arg(long)]
    pub os_version: Option<String>,
    /// Run tests in cloud browser.
    #[arg(long)]
    pub run_in_cloud: bool,
    /// Record performance metrics during the run.
    #[arg(long)]
    pub performance_run: bool,
    /// Block until the operation completes (polls progress API).
    #[arg(long)]
    pub wait: bool,
}

#[derive(clap::Args, Debug)]
pub struct AtfResultsArgs {
    /// Test suite result sys_id.
    pub result_id: String,
}

// ── Aggregate ─────────────────────────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct AggregateArgs {
    /// Table name (e.g. `incident`).
    pub table: String,
    /// Encoded query filter.
    #[arg(long, alias = "sysparm-query")]
    pub query: Option<String>,
    /// Comma-separated fields to average.
    #[arg(long, alias = "sysparm-avg-fields")]
    pub avg_fields: Option<String>,
    /// Count the number of records in the query.
    #[arg(long, alias = "sysparm-count")]
    pub count: bool,
    /// Comma-separated fields for minimum value.
    #[arg(long, alias = "sysparm-min-fields")]
    pub min_fields: Option<String>,
    /// Comma-separated fields for maximum value.
    #[arg(long, alias = "sysparm-max-fields")]
    pub max_fields: Option<String>,
    /// Comma-separated fields to sum.
    #[arg(long, alias = "sysparm-sum-fields")]
    pub sum_fields: Option<String>,
    /// Comma-separated fields to group by.
    #[arg(long, alias = "sysparm-group-by")]
    pub group_by: Option<String>,
    /// Comma-separated fields to order by.
    #[arg(long, alias = "sysparm-order-by")]
    pub order_by: Option<String>,
    /// Aggregate filter (HAVING clause).
    #[arg(long, alias = "sysparm-having")]
    pub having: Option<String>,
    /// Resolve reference/choice fields: false (default), true, or all.
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    /// Query category for index selection.
    #[arg(long, alias = "sysparm-query-category")]
    pub query_category: Option<String>,
}

// ── Scores ────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ScoresSub {
    /// List scorecards (GET /api/now/pa/scorecards).
    List(Box<ScoresListArgs>),
    /// Mark a scorecard as a favorite (POST /api/now/pa/scorecards).
    Favorite(ScoresFavoriteArgs),
    /// Remove a scorecard from favorites (DELETE /api/now/pa/scorecards).
    Unfavorite(ScoresFavoriteArgs),
}

#[derive(clap::Args, Debug)]
pub struct ScoresListArgs {
    /// Comma-separated scorecard UUIDs.
    #[arg(long, alias = "sysparm-uuid")]
    pub uuid: Option<String>,
    /// Breakdown sys_id.
    #[arg(long, alias = "sysparm-breakdown")]
    pub breakdown: Option<String>,
    /// Breakdown relation sys_id.
    #[arg(long, alias = "sysparm-breakdown-relation")]
    pub breakdown_relation: Option<String>,
    /// Elements filter sys_id.
    #[arg(long, alias = "sysparm-elements-filter")]
    pub elements_filter: Option<String>,
    /// Display value: true, false, or all.
    #[arg(long, alias = "sysparm-display", value_enum)]
    pub display: Option<DisplayValueArg>,
    /// Return only favorites.
    #[arg(long, alias = "sysparm-favorites")]
    pub favorites: bool,
    /// Return only key indicators.
    #[arg(long, alias = "sysparm-key")]
    pub key: bool,
    /// Return only indicators with a target.
    #[arg(long, alias = "sysparm-target")]
    pub target: bool,
    /// Comma-separated substrings to search for.
    #[arg(long, alias = "sysparm-contains")]
    pub contains: Option<String>,
    /// Comma-separated tag sys_ids to filter by.
    #[arg(long, alias = "sysparm-tags")]
    pub tags: Option<String>,
    /// Number of results per page (default 10, max 100).
    #[arg(long, alias = "sysparm-per-page", default_value_t = 10)]
    pub per_page: u32,
    /// Page number (default 1).
    #[arg(long, alias = "sysparm-page", default_value_t = 1)]
    pub page: u32,
    /// Field to sort by.
    #[arg(long, alias = "sysparm-sortby", value_enum)]
    pub sort_by: Option<SortBy>,
    /// Sort direction.
    #[arg(long, alias = "sysparm-sortdir", value_enum)]
    pub sort_dir: Option<SortDir>,
    /// Resolve reference/choice display values: true, false, or all.
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    /// Exclude reference link URLs from the response.
    #[arg(long, alias = "sysparm-exclude-reference-link")]
    pub exclude_reference_link: bool,
    /// Include historical score data.
    #[arg(long, alias = "sysparm-include-scores")]
    pub include_scores: bool,
    /// Start of score date range (ISO-8601).
    #[arg(long, alias = "sysparm-from")]
    pub from: Option<String>,
    /// End of score date range (ISO-8601).
    #[arg(long, alias = "sysparm-to")]
    pub to: Option<String>,
    /// Step between scores.
    #[arg(long, alias = "sysparm-step")]
    pub step: Option<u32>,
    /// Maximum number of scores to return (-1 = all).
    #[arg(long, alias = "sysparm-limit")]
    pub limit: Option<i64>,
    /// Include available breakdowns in the response.
    #[arg(long, alias = "sysparm-include-available-breakdowns")]
    pub include_available_breakdowns: bool,
    /// Include available aggregates in the response.
    #[arg(long, alias = "sysparm-include-available-aggregates")]
    pub include_available_aggregates: bool,
    /// Include real-time score data.
    #[arg(long, alias = "sysparm-include-realtime")]
    pub include_realtime: bool,
    /// Include target color scheme.
    #[arg(long, alias = "sysparm-include-target-color-scheme")]
    pub include_target_color_scheme: bool,
    /// Include forecast scores.
    #[arg(long, alias = "sysparm-include-forecast-scores")]
    pub include_forecast_scores: bool,
    /// Include trendline scores.
    #[arg(long, alias = "sysparm-include-trendline-scores")]
    pub include_trendline_scores: bool,
    /// Include prediction interval data.
    #[arg(long, alias = "sysparm-include-prediction-interval")]
    pub include_prediction_interval: bool,
}

#[derive(clap::Args, Debug)]
pub struct ScoresFavoriteArgs {
    /// Scorecard UUID.
    pub uuid: String,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SortBy {
    #[value(name = "VALUE")]
    Value,
    #[value(name = "CHANGE")]
    Change,
    #[value(name = "CHANGEPERC")]
    ChangePerc,
    #[value(name = "GAP")]
    Gap,
    #[value(name = "GAPPERC")]
    GapPerc,
    #[value(name = "NAME")]
    Name,
    #[value(name = "ORDER")]
    Order,
    #[value(name = "DEFAULT")]
    Default,
    #[value(name = "INDICATOR_GROUP")]
    IndicatorGroup,
    #[value(name = "FREQUENCY")]
    Frequency,
    #[value(name = "TARGET")]
    Target,
    #[value(name = "DATE")]
    Date,
    #[value(name = "DIRECTION")]
    Direction,
}

impl SortBy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Value => "VALUE",
            Self::Change => "CHANGE",
            Self::ChangePerc => "CHANGEPERC",
            Self::Gap => "GAP",
            Self::GapPerc => "GAPPERC",
            Self::Name => "NAME",
            Self::Order => "ORDER",
            Self::Default => "DEFAULT",
            Self::IndicatorGroup => "INDICATOR_GROUP",
            Self::Frequency => "FREQUENCY",
            Self::Target => "TARGET",
            Self::Date => "DATE",
            Self::Direction => "DIRECTION",
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SortDir {
    #[value(name = "ASC")]
    Asc,
    #[value(name = "DESC")]
    Desc,
}

impl SortDir {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

// ── Change Management ────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ChangeSub {
    /// List change requests.
    List(ChangeListArgs),
    /// Get a change request by sys_id.
    Get(ChangeGetArgs),
    /// Create a change request.
    Create(ChangeCreateArgs),
    /// Update (PATCH) a change request.
    Update(ChangeUpdateArgs),
    /// Delete a change request.
    Delete(ChangeDeleteArgs),
    /// Get valid next states for a change.
    Nextstates(ChangeSysIdArg),
    /// Update approval state on a change.
    Approvals(ChangeApprovalsArgs),
    /// Update the risk assessment of a change.
    Risk(ChangeRiskArgs),
    /// Get the schedule for a change.
    Schedule(ChangeSysIdArg),
    /// Change task operations.
    Task {
        #[command(subcommand)]
        sub: ChangeTaskSub,
    },
    /// CI relationship operations on a change.
    Ci {
        #[command(subcommand)]
        sub: ChangeCiSub,
    },
    /// Conflict operations on a change.
    Conflict {
        #[command(subcommand)]
        sub: ChangeConflictSub,
    },
    /// List change models.
    Models(ChangeOptionalIdArg),
    /// List standard change templates.
    Templates(ChangeOptionalIdArg),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum ChangeType {
    Normal,
    Emergency,
    Standard,
}

#[derive(clap::Args, Debug)]
pub struct ChangeListArgs {
    /// Filter by change type.
    #[arg(long, value_enum)]
    pub r#type: Option<ChangeType>,
    #[arg(long, alias = "sysparm-query")]
    pub query: Option<String>,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-limit", alias = "limit", default_value_t = 1000)]
    pub setlimit: u32,
    #[arg(long, alias = "sysparm-offset")]
    pub offset: Option<u32>,
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    #[arg(long, alias = "sysparm-exclude-reference-link")]
    pub exclude_reference_link: bool,
    #[arg(long, alias = "sysparm-view")]
    pub view: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeGetArgs {
    pub sys_id: String,
    /// Get a specific change type (uses type-specific endpoint).
    #[arg(long, value_enum)]
    pub r#type: Option<ChangeType>,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
    #[arg(long, alias = "sysparm-exclude-reference-link")]
    pub exclude_reference_link: bool,
    #[arg(long, alias = "sysparm-view")]
    pub view: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeCreateArgs {
    /// Change type: normal, emergency, or standard.
    #[arg(long, value_enum, default_value_t = ChangeType::Normal)]
    pub r#type: ChangeType,
    /// Standard change template sys_id (required for --type standard).
    #[arg(long)]
    pub template: Option<String>,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeUpdateArgs {
    pub sys_id: String,
    #[arg(long, value_enum)]
    pub r#type: Option<ChangeType>,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-display-value", value_enum)]
    pub display_value: Option<DisplayValueArg>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeDeleteArgs {
    pub sys_id: String,
    #[arg(long, value_enum)]
    pub r#type: Option<ChangeType>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeSysIdArg {
    pub sys_id: String,
}

#[derive(clap::Args, Debug)]
pub struct ChangeOptionalIdArg {
    pub sys_id: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeApprovalsArgs {
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeRiskArgs {
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum ChangeTaskSub {
    /// List tasks for a change.
    List(ChangeTaskListArgs),
    /// Get a specific change task.
    Get(ChangeTaskGetArgs),
    /// Create a task on a change.
    Create(ChangeTaskCreateArgs),
    /// Update a change task (PATCH).
    Update(ChangeTaskUpdateArgs),
    /// Delete a change task.
    Delete(ChangeTaskDeleteArgs),
}

#[derive(clap::Args, Debug)]
pub struct ChangeTaskListArgs {
    pub change_sys_id: String,
    #[arg(long, alias = "sysparm-fields")]
    pub fields: Option<String>,
    #[arg(long, alias = "sysparm-limit", alias = "limit", default_value_t = 100)]
    pub setlimit: u32,
}

#[derive(clap::Args, Debug)]
pub struct ChangeTaskGetArgs {
    pub change_sys_id: String,
    pub task_sys_id: String,
}

#[derive(clap::Args, Debug)]
pub struct ChangeTaskCreateArgs {
    pub change_sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeTaskUpdateArgs {
    pub change_sys_id: String,
    pub task_sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct ChangeTaskDeleteArgs {
    pub change_sys_id: String,
    pub task_sys_id: String,
}

#[derive(Subcommand, Debug)]
pub enum ChangeCiSub {
    /// List CIs associated with a change.
    List(ChangeSysIdArg),
    /// Add a CI to a change.
    Add(ChangeCiAddArgs),
}

#[derive(clap::Args, Debug)]
pub struct ChangeCiAddArgs {
    pub change_sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum ChangeConflictSub {
    /// Get conflicts for a change.
    Get(ChangeSysIdArg),
    /// Add a conflict to a change.
    Add(ChangeConflictAddArgs),
    /// Remove conflicts from a change.
    Remove(ChangeSysIdArg),
}

#[derive(clap::Args, Debug)]
pub struct ChangeConflictAddArgs {
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

// ── Attachment ───────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum AttachmentSub {
    /// List attachments with optional query filter.
    List(AttachmentListArgs),
    /// Get attachment metadata by sys_id.
    Get(AttachmentGetArgs),
    /// Upload a file as an attachment.
    Upload(AttachmentUploadArgs),
    /// Download attachment content to a file or stdout.
    Download(AttachmentDownloadArgs),
    /// Delete an attachment.
    Delete(AttachmentDeleteArgs),
}

#[derive(clap::Args, Debug)]
pub struct AttachmentListArgs {
    #[arg(long, alias = "sysparm-query")]
    pub query: Option<String>,
    #[arg(long, alias = "sysparm-limit", alias = "limit", default_value_t = 100)]
    pub setlimit: u32,
    #[arg(long, alias = "sysparm-offset")]
    pub offset: Option<u32>,
}

#[derive(clap::Args, Debug)]
pub struct AttachmentGetArgs {
    pub sys_id: String,
}

#[derive(clap::Args, Debug)]
pub struct AttachmentUploadArgs {
    /// Table to attach to (e.g. `incident`).
    #[arg(long, required = true)]
    pub table: String,
    /// sys_id of the record to attach to.
    #[arg(long, required = true)]
    pub record: String,
    /// Path to the file to upload.
    #[arg(long, required = true)]
    pub file: String,
    /// File name override (defaults to the file's basename).
    #[arg(long)]
    pub file_name: Option<String>,
    /// Content type (auto-detected if not specified).
    #[arg(long)]
    pub content_type: Option<String>,
    /// Encryption context sys_id.
    #[arg(long)]
    pub encryption_context: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct AttachmentDownloadArgs {
    pub sys_id: String,
    /// Output file path (writes to stdout if not specified).
    #[arg(long, short)]
    pub output: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct AttachmentDeleteArgs {
    pub sys_id: String,
}

// ── CMDB ─────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum CmdbSub {
    /// List CI records for a CMDB class.
    List(CmdbListArgs),
    /// Get a CI record with relations.
    Get(CmdbGetArgs),
    /// Create a CI record.
    Create(CmdbCreateArgs),
    /// Update a CI record (PATCH).
    Update(CmdbUpdateArgs),
    /// Replace a CI record (PUT).
    Replace(CmdbReplaceArgs),
    /// Get metadata for a CMDB class.
    Meta(CmdbMetaArgs),
    /// Relation operations on a CI.
    Relation {
        #[command(subcommand)]
        sub: CmdbRelationSub,
    },
}

#[derive(clap::Args, Debug)]
pub struct CmdbListArgs {
    /// CMDB class name (e.g. `cmdb_ci_server`).
    pub class: String,
    #[arg(long, alias = "sysparm-query")]
    pub query: Option<String>,
    #[arg(long, alias = "sysparm-limit", alias = "limit", default_value_t = 1000)]
    pub setlimit: u32,
    #[arg(long, alias = "sysparm-offset")]
    pub offset: Option<u32>,
}

#[derive(clap::Args, Debug)]
pub struct CmdbGetArgs {
    pub class: String,
    pub sys_id: String,
}

#[derive(clap::Args, Debug)]
pub struct CmdbCreateArgs {
    pub class: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct CmdbUpdateArgs {
    pub class: String,
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct CmdbReplaceArgs {
    pub class: String,
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct CmdbMetaArgs {
    pub class: String,
}

#[derive(Subcommand, Debug)]
pub enum CmdbRelationSub {
    /// Create a relation on a CI.
    Add(CmdbRelationAddArgs),
    /// Delete a relation from a CI.
    Delete(CmdbRelationDeleteArgs),
}

#[derive(clap::Args, Debug)]
pub struct CmdbRelationAddArgs {
    pub class: String,
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct CmdbRelationDeleteArgs {
    pub class: String,
    pub sys_id: String,
    /// sys_id of the relation to delete.
    pub rel_sys_id: String,
}

// ── Import Set ───────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ImportSub {
    /// Insert a single record into a staging table.
    Create(ImportCreateArgs),
    /// Insert multiple records into a staging table.
    Bulk(ImportBulkArgs),
    /// Retrieve an import set record.
    Get(ImportGetArgs),
}

#[derive(clap::Args, Debug)]
pub struct ImportCreateArgs {
    /// Staging table name.
    pub staging_table: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct ImportBulkArgs {
    /// Staging table name.
    pub staging_table: String,
    /// JSON array of records, @file, or @- for stdin.
    #[arg(long, required = true)]
    pub data: String,
}

#[derive(clap::Args, Debug)]
pub struct ImportGetArgs {
    /// Staging table name.
    pub staging_table: String,
    /// sys_id of the import set record.
    pub sys_id: String,
}

// ── Service Catalog ──────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum CatalogSub {
    /// List service catalogs.
    List(CatalogListArgs),
    /// Get a specific catalog.
    Get(CatalogGetArgs),
    /// List categories for a catalog.
    Categories(CatalogCategoriesArgs),
    /// Get a specific category.
    Category(CatalogCategoryArgs),
    /// List catalog items.
    Items(CatalogItemsArgs),
    /// Get a specific catalog item.
    Item(CatalogItemArgs),
    /// Get variables for a catalog item.
    ItemVariables(CatalogItemArgs),
    /// Order a catalog item immediately.
    Order(CatalogOrderArgs),
    /// Add a catalog item to cart.
    AddToCart(CatalogOrderArgs),
    /// Get the current cart.
    Cart,
    /// Update a cart item.
    CartUpdate(CatalogCartUpdateArgs),
    /// Remove an item from cart.
    CartRemove(CatalogCartItemArgs),
    /// Empty the cart.
    CartEmpty(CatalogCartEmptyArgs),
    /// Check out the cart.
    Checkout,
    /// Submit the cart order.
    SubmitOrder,
    /// Get the wishlist.
    Wishlist,
}

#[derive(clap::Args, Debug)]
pub struct CatalogListArgs {
    /// Search text for catalogs.
    #[arg(long, alias = "sysparm-text")]
    pub text: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct CatalogGetArgs {
    pub sys_id: String,
}

#[derive(clap::Args, Debug)]
pub struct CatalogCategoriesArgs {
    /// Catalog sys_id.
    pub catalog_sys_id: String,
    #[arg(long, alias = "sysparm-limit", alias = "limit", default_value_t = 100)]
    pub setlimit: u32,
    #[arg(long, alias = "sysparm-offset")]
    pub offset: Option<u32>,
    /// Show only top-level categories.
    #[arg(long, alias = "sysparm-top-level-only")]
    pub top_level_only: bool,
}

#[derive(clap::Args, Debug)]
pub struct CatalogCategoryArgs {
    pub sys_id: String,
}

#[derive(clap::Args, Debug)]
pub struct CatalogItemsArgs {
    /// Search text for items.
    #[arg(long, alias = "sysparm-text")]
    pub text: Option<String>,
    /// Filter by category sys_id.
    #[arg(long, alias = "sysparm-category")]
    pub category: Option<String>,
    /// Filter by catalog sys_id.
    #[arg(long, alias = "sysparm-catalog")]
    pub catalog: Option<String>,
    /// Filter by type (e.g. `record_producer`).
    #[arg(long, alias = "sysparm-type")]
    pub item_type: Option<String>,
    #[arg(long, alias = "sysparm-limit", alias = "limit", default_value_t = 100)]
    pub setlimit: u32,
    #[arg(long, alias = "sysparm-offset")]
    pub offset: Option<u32>,
}

#[derive(clap::Args, Debug)]
pub struct CatalogItemArgs {
    pub sys_id: String,
}

#[derive(clap::Args, Debug)]
pub struct CatalogOrderArgs {
    pub sys_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct CatalogCartUpdateArgs {
    pub cart_item_id: String,
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct CatalogCartItemArgs {
    pub cart_item_id: String,
}

#[derive(clap::Args, Debug)]
pub struct CatalogCartEmptyArgs {
    /// Cart sys_id.
    pub sys_id: String,
}

// ── Identification & Reconciliation ──────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum IdentifySub {
    /// Create or update a CI (POST /api/now/identifyreconcile).
    CreateUpdate(IdentifyArgs),
    /// Identify a CI without modifying (POST /api/now/identifyreconcile/query).
    Query(IdentifyArgs),
    /// Create or update with enhanced options.
    CreateUpdateEnhanced(IdentifyEnhancedArgs),
    /// Identify with enhanced options.
    QueryEnhanced(IdentifyEnhancedArgs),
}

#[derive(clap::Args, Debug)]
pub struct IdentifyArgs {
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
    /// Data source identifier.
    #[arg(long)]
    pub data_source: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct IdentifyEnhancedArgs {
    #[arg(long, conflicts_with = "field")]
    pub data: Option<String>,
    #[arg(long = "field", conflicts_with = "data")]
    pub field: Vec<String>,
    /// Data source identifier.
    #[arg(long)]
    pub data_source: Option<String>,
    /// Comma-separated key:value options (e.g. `partial_payload:true,partial_commits:true`).
    #[arg(long)]
    pub options: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_compiles_and_debugs() {
        Cli::command().debug_assert();
    }

    #[test]
    fn pretty_and_compact_conflict() {
        let err = Cli::try_parse_from(["sn", "--pretty", "--compact", "introspect"]).unwrap_err();
        // clap emits conflict error; kind may differ by version, just assert it's an error
        let _ = err.kind();
    }
}
