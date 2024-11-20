use clap::Args;
use color_eyre::Result;
use rust_code_analysis::{get_function_spaces, guess_language, read_file_with_eol};
use std::fs::File;
use std::path::PathBuf;
use tracing::{error, info, warn};

use crate::errors::CliError;
use rust_code_analysis::FuncSpace;
use serde::{Deserialize, Serialize};

#[derive(Args)]
pub struct SourceCommand {
    #[arg(short, long)]
    path: PathBuf,
    #[arg(short, long, default_value = "csv", value_parser = clap::builder::PossibleValuesParser::new(["json", "csv"]))]
    fmt: String,
    #[arg(long, default_value = "false")]
    no_flatten: bool,
}

impl SourceCommand {
    pub fn execute(mut self) -> Result<(), CliError> {
        if self.no_flatten && self.fmt == "csv" {
            warn!("You have chosen the output format of CSV as well as not flattening. This is not supported \
            and the output format will be swap to JSON");
            self.fmt = "json".to_string();
        }

        info!("Executing source command on file: {}", self.path.display());

        let source = if let Some(source) = read_file_with_eol(&self.path)? {
            source
        } else {
            return Ok(());
        };

        let language = if let Some(language) = guess_language(&source, &self.path).0 {
            language
        } else {
            return Ok(());
        };

        info!("Source: {:?} bytes Language: {:?}", source.len(), language);

        if let Some(space) = get_function_spaces(&language, source.clone(), &self.path, None) {
            info!("Successfully extracted function metrics");

            // Fix the filepath ending
            let output_path = match self.fmt.as_str() {
                "csv" => self.path.with_extension("c.csv"),
                "json" => self.path.with_extension("c.json"),

                _ => {
                    unreachable!("Invalid format")
                }
            };

            // Remove any additional parent dirs etc
            let output_path = output_path.file_name().unwrap().to_str().unwrap();

            if self.no_flatten {
                match self.fmt.as_str() {
                    "csv" => {
                        error!("Not possible!")
                    }
                    "json" => {
                        serde_json::to_writer_pretty(File::create(output_path).unwrap(), &space)?;
                        info!("All saved to JSON")
                    }
                    _ => {}
                }
            } else {
                let mut flattened: Vec<FlattenedMetrics> = Vec::new();

                flatten_spaces(
                    &space.spaces,
                    &Some(self.path.to_string_lossy().to_string()),
                    &mut flattened,
                );

                match self.fmt.as_str() {
                    "csv" => {
                        let file = File::create(output_path)?;
                        let mut writer = csv::Writer::from_writer(file);
                        for entry in flattened {
                            writer.serialize(entry)?
                        }

                        writer.flush()?;
                        info!("All saved to CSV")
                    }
                    "json" => {
                        serde_json::to_writer_pretty(File::create(output_path).unwrap(), &space)?;
                        info!("All saved to JSON")
                    }
                    _ => {
                        unreachable!("Invalid format provided.")
                    }
                }
            }

            Ok(())
        } else {
            Err(CliError::FailedProcessing(
                "Failed to extract function metrics".to_string(),
            ))
        }
    }
}

// Flattened structure
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct FlattenedMetrics {
    pub name: Option<String>,
    pub source_file: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub kind: String,
    pub parent_name: Option<String>,

    // NArgs
    pub nargs_total_functions: f64,
    pub nargs_total_closures: f64,
    pub nargs_average_functions: f64,
    pub nargs_average_closures: f64,
    pub nargs_total: f64,
    pub nargs_average: f64,
    pub nargs_functions_min: f64,
    pub nargs_functions_max: f64,
    pub nargs_closures_min: f64,
    pub nargs_closures_max: f64,

    // Exits
    pub nexits_sum: f64,
    pub nexits_average: f64,
    pub nexits_min: f64,
    pub nexits_max: f64,

    // Cognitive
    pub cognitive_sum: f64,
    pub cognitive_average: f64,
    pub cognitive_min: f64,
    pub cognitive_max: f64,

    // Cyclomatic
    pub cyclomatic_sum: f64,
    pub cyclomatic_average: f64,
    pub cyclomatic_min: f64,
    pub cyclomatic_max: f64,

    // Halstead
    pub halstead_n1: f64,
    pub halstead_N1: f64,
    pub halstead_n2: f64,
    pub halstead_N2: f64,
    pub halstead_length: f64,
    pub halstead_estimated_program_length: Option<f64>,
    pub halstead_purity_ratio: Option<f64>,
    pub halstead_vocabulary: f64,
    pub halstead_volume: f64,
    pub halstead_difficulty: f64,
    pub halstead_level: Option<f64>,
    pub halstead_effort: f64,
    pub halstead_time: f64,
    pub halstead_bugs: f64,

    // Loc
    pub loc_sloc: f64,
    pub loc_ploc: f64,
    pub loc_lloc: f64,
    pub loc_cloc: f64,
    pub loc_blank: f64,

    // Nom
    pub nom_functions: f64,
    pub nom_closures: f64,
    pub nom_total: f64,
    pub nom_functions_min: f64,
    pub nom_functions_max: f64,
    pub nom_closures_min: f64,
    pub nom_closures_max: f64,

    // Mi
    pub mi_original: f64,
    pub mi_sei: f64,
    pub mi_visual_studio: f64,
}

impl FlattenedMetrics {
    pub fn from_space(
        space: &FuncSpace,
        parent_name: Option<String>,
        source_file: Option<String>,
    ) -> Self {
        Self {
            name: space.name.clone(),
            source_file,
            start_line: space.start_line,
            end_line: space.end_line,
            kind: space.kind.clone().to_string(),
            parent_name,

            // NArgs
            nargs_total_functions: space.metrics.nargs.fn_args_sum(),
            nargs_total_closures: space.metrics.nargs.closure_args_sum(),
            nargs_average_functions: space.metrics.nargs.fn_args_average(),
            nargs_average_closures: space.metrics.nargs.closure_args_average(),
            nargs_total: space.metrics.nargs.nargs_total(),
            nargs_average: space.metrics.nargs.nargs_average(),
            nargs_functions_min: space.metrics.nargs.fn_args_min(),
            nargs_functions_max: space.metrics.nargs.fn_args_max(),
            nargs_closures_min: space.metrics.nargs.closure_args_min(),
            nargs_closures_max: space.metrics.nargs.closure_args_max(),

            // Exits
            nexits_sum: space.metrics.nexits.exit_sum(),
            nexits_average: space.metrics.nexits.exit_average(),
            nexits_min: space.metrics.nexits.exit_min(),
            nexits_max: space.metrics.nexits.exit_max(),

            // Cognitive
            cognitive_sum: space.metrics.cognitive.cognitive_sum(),
            cognitive_average: space.metrics.cognitive.cognitive_average(),
            cognitive_min: space.metrics.cognitive.cognitive_min(),
            cognitive_max: space.metrics.cognitive.cognitive_max(),

            // Cyclomatic
            cyclomatic_sum: space.metrics.cyclomatic.cyclomatic_sum(),
            cyclomatic_average: space.metrics.cyclomatic.cyclomatic_average(),
            cyclomatic_min: space.metrics.cyclomatic.cyclomatic_min(),
            cyclomatic_max: space.metrics.cyclomatic.cyclomatic_max(),

            // Halstead
            halstead_n1: space.metrics.halstead.u_operators(),
            halstead_N1: space.metrics.halstead.operators(),
            halstead_n2: space.metrics.halstead.u_operands(),
            halstead_N2: space.metrics.halstead.operands(),
            halstead_length: space.metrics.halstead.length(),
            halstead_estimated_program_length: Some(
                space.metrics.halstead.estimated_program_length(),
            ),
            halstead_purity_ratio: Some(space.metrics.halstead.purity_ratio()),
            halstead_vocabulary: space.metrics.halstead.vocabulary(),
            halstead_volume: space.metrics.halstead.volume(),
            halstead_difficulty: space.metrics.halstead.difficulty(),
            halstead_level: Some(space.metrics.halstead.level()),
            halstead_effort: space.metrics.halstead.effort(),
            halstead_time: space.metrics.halstead.time(),
            halstead_bugs: space.metrics.halstead.bugs(),

            // Loc
            loc_sloc: space.metrics.loc.sloc(),
            loc_ploc: space.metrics.loc.ploc(),
            loc_lloc: space.metrics.loc.lloc(),
            loc_cloc: space.metrics.loc.cloc(),
            loc_blank: space.metrics.loc.blank(),

            // Nom
            nom_functions: space.metrics.nom.functions(),
            nom_closures: space.metrics.nom.closures(),
            nom_total: space.metrics.nom.total(),
            nom_functions_min: space.metrics.nom.functions_min(),
            nom_functions_max: space.metrics.nom.functions_max(),
            nom_closures_min: space.metrics.nom.closures_min(),
            nom_closures_max: space.metrics.nom.closures_max(),

            // Mi
            mi_original: space.metrics.mi.mi_original(),
            mi_sei: space.metrics.mi.mi_sei(),
            mi_visual_studio: space.metrics.mi.mi_visual_studio(),
        }
    }
}

fn flatten_spaces(
    spaces: &[FuncSpace],
    source_name: &Option<String>,
    flattened: &mut Vec<FlattenedMetrics>,
) {
    for space in spaces {
        flattened.push(FlattenedMetrics::from_space(
            space,
            Some(space.name.clone().unwrap()),
            Some(source_name.clone().unwrap()),
        ));

        // Recursively process nested spaces
        flatten_spaces(&space.spaces, source_name, flattened);
    }
}
