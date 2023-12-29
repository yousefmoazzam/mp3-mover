use std::env;
use std::process;

use log::{error, info};

use mp3_mover::config::Config;
use mp3_mover::run;

fn main(){
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        error!("Problem parsing args: {}", err);
        process::exit(1);
    });

    if let Err(e) = run(config) {
        error!("Encountered error: {}", e);
        process::exit(1);
    }

    info!("Program completed sucecssfully");
}
