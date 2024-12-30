use clap::Args;
use color_eyre::Result;
use rayon::prelude::*;
use rust_code_analysis::{get_function_spaces, guess_language, read_file};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::exit;

use crate::errors::CliError;
use archaeo_macros::ReplaceInfNan;
use rust_code_analysis::FuncSpace;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

#[derive(Args)]
pub struct SourceCommand {
    #[arg(short, long)]
    path: PathBuf,
    #[arg(short, long)]
    output_path: PathBuf,
    #[arg(short, long, default_value = "csv", value_parser = clap::builder::PossibleValuesParser::new(["json", "csv"]))]
    fmt: String,
    #[arg(long, default_value = "false")]
    no_flatten: bool,
    #[arg(long, default_value = "false")]
    extended: bool,
}

impl SourceCommand {
    pub fn execute(mut self) -> Result<(), CliError> {
        let extensions: Vec<String> = vec![
            "cpp".to_string(),
            "cc".to_string(),
            "hpp".to_string(),
            "c".to_string(),
            "h".to_string(),
        ];

        if self.no_flatten && self.fmt == "csv" {
            warn!("You have chosen the output format of CSV as well as not flattening. This is not supported \
            and the output format will be swap to JSON");
            self.fmt = "json".to_string();
        }

        let mut filepaths = Vec::new();

        if self.path.is_file() {
            info!("Single file found...");
            filepaths.push(self.path.clone());
        } else if self.path.is_dir() {
            info!("Multiple files found...");
            for entry in WalkDir::new(&self.path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let path = entry.path();
                    if Self::has_valid_extension(path, &extensions) {
                        filepaths.push(path.to_path_buf());
                    }
                }
            }
        } else {
            error!("The provided path is not a file or a dir. Exiting...");
            exit(1)
        }

        if !self.output_path.exists() {
            info!("The output path does not exist. Creating...");
            fs::create_dir_all(&self.output_path)?;
        }

        filepaths
            .par_iter()
            .try_for_each(|fp| self.extract_metrics(fp))?;

        Ok(())
    }

    fn extract_metrics(&self, path: &PathBuf) -> Result<(), CliError> {
        info!("Executing source command on file: {}", path.display());

        let source = read_file(path)
            .map_err(|_| CliError::FailedProcessing(path.to_string_lossy().to_string()))?;

        let language = if let Some(language) = guess_language(&source, path).0 {
            language
        } else {
            return Err(CliError::FailedGuessLang(
                path.to_string_lossy().to_string(),
            ));
        };

        debug!("Source: {:?} bytes Language: {:?}", source.len(), language);

        if let Some(space) = get_function_spaces(&language, source.clone(), path, None) {
            debug!("Successfully extracted function metrics");

            // Fix the filepath ending
            let output_path = match self.fmt.as_str() {
                "csv" if self.extended => path.with_file_name(format!(
                    "{}-extended.csv",
                    path.file_stem().unwrap().to_string_lossy()
                )),
                "json" if self.extended => path.with_file_name(format!(
                    "{}-extended.json",
                    path.file_stem().unwrap().to_string_lossy()
                )),
                "csv" => path.with_extension("csv"),
                "json" => path.with_extension("json"),
                _ => {
                    unreachable!("Invalid format")
                }
            };

            // Remove any additional parent dirs etc
            let output_path = output_path.file_name().unwrap().to_str().unwrap();
            let output_path = self.output_path.clone().join(output_path);

            if self.no_flatten {
                match self.fmt.as_str() {
                    "csv" => {
                        error!("Not possible!")
                    }
                    "json" => {
                        serde_json::to_writer_pretty(File::create(output_path).unwrap(), &space)?;
                        debug!("All saved to JSON")
                    }
                    _ => {}
                }
            } else {
                let flattened = if self.extended {
                    let mut flattened: Vec<FlattenedMetricsExtended> = Vec::new();

                    flatten_spaces_extended(
                        &space.spaces,
                        &Some(path.to_string_lossy().to_string()),
                        &mut flattened,
                    );

                    if flattened.is_empty() {
                        debug!("No function metrics extracted for {}", path.display());
                        return Ok(());
                    }
                    MetricsType::Extended(flattened)
                } else {
                    let mut flattened: Vec<FlattenedMetrics> = Vec::new();

                    flatten_spaces(
                        &space.spaces,
                        &Some(path.to_string_lossy().to_string()),
                        &mut flattened,
                    );

                    if flattened.is_empty() {
                        debug!("No function metrics extracted for {}", path.display());
                        return Ok(());
                    }
                    MetricsType::Regular(flattened)
                };

                match self.fmt.as_str() {
                    "csv" => {
                        let file = File::create(output_path)?;
                        let mut writer = csv::Writer::from_writer(file);
                        match &flattened {
                            MetricsType::Extended(metrics) => {
                                for entry in metrics {
                                    writer.serialize(entry)?
                                }
                            }
                            MetricsType::Regular(metrics) => {
                                for entry in metrics {
                                    writer.serialize(entry)?
                                }
                            }
                        }
                        writer.flush()?;
                        debug!("All saved to CSV")
                    }
                    "json" => {
                        serde_json::to_writer_pretty(File::create(output_path).unwrap(), &space)?;
                        debug!("All saved to JSON")
                    }
                    _ => {
                        unreachable!("Invalid format provided.")
                    }
                }
            }

            Ok(())
        } else {
            error!("Failed to process: {}", path.display());
            Ok(())
        }
    }

    // Helper function to check file extensions
    fn has_valid_extension(path: &Path, extensions: &[String]) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                return extensions
                    .iter()
                    .any(|valid_ext| ext_str.eq_ignore_ascii_case(valid_ext));
            }
        }
        false
    }
}

enum MetricsType {
    Extended(Vec<FlattenedMetricsExtended>),
    Regular(Vec<FlattenedMetrics>),
}

pub trait ReplaceInfNan {
    fn replace_inf_nan(&mut self);
}

impl ReplaceInfNan for f64 {
    fn replace_inf_nan(&mut self) {
        if self.is_infinite() || self.is_nan() {
            *self = 0.0;
        }
    }
}

// Flattended Structure
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, ReplaceInfNan)]
pub struct FlattenedMetrics {
    pub name: Option<String>,
    pub source_file: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub kind: String,
    pub parent_name: Option<String>,

    // NArgs
    pub fn_args: f64,
    pub closure_args: f64,

    // Exits
    pub nexits: f64,

    // Cognitive
    pub cognitive: f64,

    // Cyclomatic
    pub cyclomatic: f64,

    // Halstead
    pub halstead_n1: f64,
    pub halstead_N1: f64,
    pub halstead_n2: f64,
    pub halstead_N2: f64,
    pub halstead_length: f64,
    pub halstead_estimated_program_length: f64,
    pub halstead_purity_ratio: f64,
    pub halstead_vocabulary: f64,
    pub halstead_volume: f64,
    pub halstead_difficulty: f64,
    pub halstead_level: f64,
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

    // Mi
    pub mi_original: f64,
    pub mi_sei: f64,
    pub mi_visual_studio: f64,
}

// Flattened Extended structure
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize, ReplaceInfNan)]
pub struct FlattenedMetricsExtended {
    pub name: Option<String>,
    pub source_file: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub kind: String,
    pub parent_name: Option<String>,

    // NArgs
    pub fn_args: f64,
    pub closure_args: f64,
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
    pub nexits: f64,
    pub nexits_sum: f64,
    pub nexits_average: f64,
    pub nexits_min: f64,
    pub nexits_max: f64,

    // Cognitive
    pub cognitive: f64,
    pub cognitive_sum: f64,
    pub cognitive_average: f64,
    pub cognitive_min: f64,
    pub cognitive_max: f64,

    // Cyclomatic
    pub cyclomatic: f64,
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
    pub halstead_estimated_program_length: f64,
    pub halstead_purity_ratio: f64,
    pub halstead_vocabulary: f64,
    pub halstead_volume: f64,
    pub halstead_difficulty: f64,
    pub halstead_level: f64,
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

impl FlattenedMetricsExtended {
    pub fn from_space(
        space: &FuncSpace,
        parent_name: Option<String>,
        source_file: Option<String>,
    ) -> Self {
        let mut obj = Self {
            name: space.name.clone(),
            source_file,
            start_line: space.start_line,
            end_line: space.end_line,
            kind: space.kind.clone().to_string(),
            parent_name,

            // NArgs
            fn_args: space.metrics.nargs.fn_args(),
            closure_args: space.metrics.nargs.closure_args(),
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
            nexits: space.metrics.nexits.exit(),
            nexits_sum: space.metrics.nexits.exit_sum(),
            nexits_average: space.metrics.nexits.exit_average(),
            nexits_min: space.metrics.nexits.exit_min(),
            nexits_max: space.metrics.nexits.exit_max(),

            // Cognitive
            cognitive: space.metrics.cognitive.cognitive(),
            cognitive_sum: space.metrics.cognitive.cognitive_sum(),
            cognitive_average: space.metrics.cognitive.cognitive_average(),
            cognitive_min: space.metrics.cognitive.cognitive_min(),
            cognitive_max: space.metrics.cognitive.cognitive_max(),

            // Cyclomatic
            cyclomatic: space.metrics.cyclomatic.cyclomatic(),
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
            halstead_estimated_program_length: space.metrics.halstead.estimated_program_length(),
            halstead_purity_ratio: space.metrics.halstead.purity_ratio(),
            halstead_vocabulary: space.metrics.halstead.vocabulary(),
            halstead_volume: space.metrics.halstead.volume(),
            halstead_difficulty: space.metrics.halstead.difficulty(),
            halstead_level: space.metrics.halstead.level(),
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
        };

        // Scan through struct members and replace nan/inf's with 0.0
        obj.replace_inf_nan();

        obj
    }
}

fn flatten_spaces_extended(
    spaces: &[FuncSpace],
    source_name: &Option<String>,
    flattened: &mut Vec<FlattenedMetricsExtended>,
) {
    for space in spaces {
        flattened.push(FlattenedMetricsExtended::from_space(
            space,
            Some(space.name.clone().unwrap_or("no_name_found".to_string())),
            Some(source_name.clone().unwrap()),
        ));

        // Recursively process nested spaces
        flatten_spaces_extended(&space.spaces, source_name, flattened);
    }
}

impl FlattenedMetrics {
    pub fn from_space(
        space: &FuncSpace,
        parent_name: Option<String>,
        source_file: Option<String>,
    ) -> Self {
        let mut obj = Self {
            name: space.name.clone(),
            source_file,
            start_line: space.start_line,
            end_line: space.end_line,
            kind: space.kind.clone().to_string(),
            parent_name,

            // NArgs
            fn_args: space.metrics.nargs.fn_args(),
            closure_args: space.metrics.nargs.closure_args(),

            // Exits
            nexits: space.metrics.nexits.exit(),

            // Cognitive
            cognitive: space.metrics.cognitive.cognitive(),

            // Cyclomatic
            cyclomatic: space.metrics.cyclomatic.cyclomatic(),

            // Halstead
            halstead_n1: space.metrics.halstead.u_operators(),
            halstead_N1: space.metrics.halstead.operators(),
            halstead_n2: space.metrics.halstead.u_operands(),
            halstead_N2: space.metrics.halstead.operands(),
            halstead_length: space.metrics.halstead.length(),
            halstead_estimated_program_length: space.metrics.halstead.estimated_program_length(),
            halstead_purity_ratio: space.metrics.halstead.purity_ratio(),
            halstead_vocabulary: space.metrics.halstead.vocabulary(),
            halstead_volume: space.metrics.halstead.volume(),
            halstead_difficulty: space.metrics.halstead.difficulty(),
            halstead_level: space.metrics.halstead.level(),
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

            // Mi
            mi_original: space.metrics.mi.mi_original(),
            mi_sei: space.metrics.mi.mi_sei(),
            mi_visual_studio: space.metrics.mi.mi_visual_studio(),
        };

        // Scan through struct members and replace nan/inf's with 0.0
        obj.replace_inf_nan();

        obj
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
            Some(space.name.clone().unwrap_or("no_name_found".to_string())),
            Some(source_name.clone().unwrap()),
        ));

        // Recursively process nested spaces
        flatten_spaces(&space.spaces, source_name, flattened);
    }
}
