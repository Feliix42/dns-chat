use clap::Clap;

#[derive(Clap)]
#[clap(
    name = "sagishi",
    version = "1.0",
    author = "Felix Wittwer <hallo@felixwittwer.de>",
    about = "A stupid simple proof-of-concept chat application that uses the DNS protocol to exchange messages."
)]
pub struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    //#[clap(short, long, default_value = "default.conf")]
    #[clap(about = "Target IP address")]
    pub target: String,
    /// Some input. Because this isn't an Option<T> it's required to be used
    #[clap(short, long, default_value = "53")]
    pub target_port: u16,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, default_value = "53")]
    pub listening_port: u16,
}
