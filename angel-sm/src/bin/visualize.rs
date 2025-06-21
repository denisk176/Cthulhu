use std::io::Write;
use clap::{Parser, Subcommand, ValueEnum};
use cthulhu_angel_sm::builder::StateMachineBuilder;
use graphviz_rust::cmd::Format;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::exec_dot;
use graphviz_rust::printer::{DotPrinter, PrinterContext};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: CliCmd,
}

#[derive(Debug, Subcommand)]
enum CliCmd {
    List,
    Graph {
        #[clap(long, short, value_enum)]
        format: CliFormat,
        #[clap(long, short)]
        output: Option<PathBuf>,
        state: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum CliFormat {
    Dot,
    Svg,
    Png,
}

fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        CliCmd::List => {
            let mut builder = StateMachineBuilder::new();
            builder.load_builtin_state_files()?;
            for id in builder.loaded_state_file_ids() {
                println!("{id}");
            }
        }
        CliCmd::Graph { format, output, state } => {
            let mut builder = StateMachineBuilder::new();
            builder.load_builtin_state_files()?;
            builder.activate_state_file(&state)?;
            let sm = builder.build()?;
            let mut g: Graph = graph!(strict di id!(&state));
            for n in sm.states() {
                g.add_stmt(node!(n).into());
            }
            for n in sm.states() {
                let s = sm.state(&n)?;
                for t in s.transitions {
                    g.add_stmt(edge!(node_id!(&n) => node_id!(&t.target)).into());
                }
            }
            let dot = g.print(&mut PrinterContext::default());
            let data = match format {
                CliFormat::Dot => dot.into_bytes(),
                CliFormat::Svg => {
                    exec_dot(dot.clone(), vec![Format::Svg.into()])?
                }
                CliFormat::Png => {
                    exec_dot(dot.clone(), vec![Format::Png.into()])?
                }
            };

            match output {
                None => {
                    std::io::stdout().write_all(&data)?;
                }
                Some(f) => {
                    std::fs::write(f, data)?;
                }
            }
        }
    }
    Ok(())
}
