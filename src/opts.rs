use clap::Clap;

#[derive(Clap)]
#[clap(
    name = "kakure",
    version = "1.0",
    author = "Felix Wittwer <hallo@felixwittwer.de>",
    about = "A stupid simple proof-of-concept chat application that uses the DNS protocol to exchange messages."
)]
pub struct Opts {
    /// Target IP address
    pub target: String,
    /// Target port on the other side.
    #[clap(short, long, default_value = "53")]
    pub target_port: u16,
    /// Port the client listens on.
    #[clap(short, long, default_value = "53")]
    pub listening_port: u16,
}
