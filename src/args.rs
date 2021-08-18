use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(name = "config_file_path", default_value = "config.ini")]
    pub config_file_path: String,
}
