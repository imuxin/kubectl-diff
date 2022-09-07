use delta_lib::cli;
use delta_lib::config;
use delta_lib::env;
use delta_lib::git_config;
use delta_lib::subcommands;
use delta_lib::utils;
use kube::api::DynamicObject;
use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;

pub fn diff(v: &Vec<DynamicObject>) -> std::io::Result<i32> {
    if v.len() < 2 {
        return Ok(0);
    }

    let (minus_file, plus_file) = store_to_file(v);
    // init delta args
    let exit_code = diff_files(minus_file, plus_file)?;
    return Ok(exit_code);
}

fn store_to_file(v: &Vec<DynamicObject>) -> (PathBuf, PathBuf) {
    let obj_last = v.last().unwrap();
    let obj_penultimate = &v[v.len() - 2];

    let plus_yaml = serde_yaml::to_string(obj_last).unwrap();
    let minus_yaml = serde_yaml::to_string(obj_penultimate).unwrap();

    let mut path = temp_dir();
    path.push("kubectl-watch");
    fs::create_dir_all(&path).unwrap();
    let mut minus_file = path.clone();
    minus_file.push("minus");
    let mut plus_file = path.clone();
    plus_file.push("plus");

    std::fs::write(&minus_file, minus_yaml).unwrap();
    std::fs::write(&plus_file, plus_yaml).unwrap();

    let minus_file = PathBuf::from(&minus_file);
    let plus_file = PathBuf::from(&plus_file);
    return (minus_file, plus_file);
}

pub fn diff_files(minus_file: PathBuf, plus_file: PathBuf) -> std::io::Result<i32> {
    utils::process::start_determining_calling_process_in_thread();
    // init delta args
    let assets = utils::bat::assets::load_highlighting_assets();
    let env = env::DeltaEnv::init();
    let mut opt =
        cli::Opt::from_git_config(env.clone(), git_config::GitConfig::try_create(&env), assets);
    opt.computed.paging_mode = utils::bat::output::PagingMode::Never;
    opt.side_by_side = true;
    let config = config::Config::from(opt);

    let mut output_type = utils::bat::output::OutputType::from_mode(
        &env,
        config.paging_mode,
        config.pager.clone(),
        &config,
    )
    .unwrap();
    let writer = output_type.handle().unwrap();
    let exit_code = subcommands::diff::diff(&minus_file, &plus_file, &config, writer);
    return Ok(exit_code);

    // Note(ql): show delta config
    // let stdout = std::io::stdout();
    // let mut stdout = stdout.lock();
    // subcommands::show_config::show_config(&config, &mut stdout)?;
}
