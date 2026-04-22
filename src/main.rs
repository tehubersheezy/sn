use clap::Parser;
use sn::cli::{AppSub, AtfSub, AuthSub, Cli, Command, SchemaSub, TableSub, UpdateSetSub};
use sn::error::Result;
use sn::output::emit_error;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    sn::observability::set_level(cli.global.verbose);
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let _ = emit_error(io::stderr().lock(), &err);
            ExitCode::from(err.exit_code() as u8)
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    let Cli { global, command } = cli;
    match command {
        Command::Init(args) => sn::cli::init::run(args),
        Command::Auth { sub } => match sub {
            AuthSub::Test => sn::cli::auth::test(&global),
        },
        Command::Profile { sub } => sn::cli::profile::run(sub),
        Command::Introspect => sn::cli::introspect::run(),
        Command::Table { sub } => match sub {
            TableSub::List(args) => sn::cli::table::list(&global, args),
            TableSub::Get(args) => sn::cli::table::get(&global, args),
            TableSub::Create(args) => sn::cli::table::create(&global, args),
            TableSub::Update(args) => sn::cli::table::update(&global, args),
            TableSub::Replace(args) => sn::cli::table::replace(&global, args),
            TableSub::Delete(args) => sn::cli::table::delete(&global, args),
        },
        Command::Schema { sub } => match sub {
            SchemaSub::Tables(args) => sn::cli::schema::tables(&global, args),
            SchemaSub::Columns(args) => sn::cli::schema::columns(&global, args),
            SchemaSub::Choices(args) => sn::cli::schema::choices(&global, args),
        },
        Command::Progress(args) => sn::cli::progress::run(&global, args),
        Command::App { sub } => match sub {
            AppSub::Install(args) => sn::cli::app::install(&global, args),
            AppSub::Publish(args) => sn::cli::app::publish(&global, args),
            AppSub::Rollback(args) => sn::cli::app::rollback(&global, args),
        },
        Command::UpdateSet { sub } => match sub {
            UpdateSetSub::Create(args) => sn::cli::update_set::create(&global, args),
            UpdateSetSub::Retrieve(args) => sn::cli::update_set::retrieve(&global, args),
            UpdateSetSub::Preview(args) => sn::cli::update_set::preview(&global, args),
            UpdateSetSub::Commit(args) => sn::cli::update_set::commit(&global, args),
            UpdateSetSub::CommitMultiple(args) => {
                sn::cli::update_set::commit_multiple(&global, args)
            }
            UpdateSetSub::BackOut(args) => sn::cli::update_set::back_out(&global, args),
        },
        Command::Atf { sub } => match sub {
            AtfSub::Run(args) => sn::cli::atf::run(&global, args),
            AtfSub::Results(args) => sn::cli::atf::results(&global, args),
        },
    }
}
