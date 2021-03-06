use clap::{App, Arg};
use dns_lookup::lookup_host;
#[cfg(feature = "fuse_client")]
use fuse;
use rich::{quit_error, unpack_or_default, unwrap_or_err};
use shfs_api::responses::Response;
use shfs_client::{ServerConnection, VolumeConnection};
#[cfg(feature = "fuse_client")]
use shfs_fuse_fs;
use shfs_server::FileServer;
use std::ffi::OsStr;
use std::net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = App::new("ShareFS")
        .version(option_env!("CARGO_PKG_VERSION").unwrap())
        .author("JMARyA <jmarya0@icloud.com>")
        .about("Shared Network Filesystem")
        .subcommand(
            App::new("serve")
                .about("set up server")
                .arg(
                    Arg::with_name("config")
                        .short("C")
                        .long("config")
                        .required(true)
                        .value_name("FILE")
                        .help("Config file for the server"),
                )
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .default_value("30")
                        .help("used port"),
                ),
        )
        .subcommand(
            App::new("mount")
                .about("mount filesystem")
                .arg(
                    Arg::with_name("host")
                        .required(true)
                        .help("Fileserver Address"),
                )
                .arg(
                    Arg::with_name("mountpoint")
                        .help("filesystem mountpoint")
                        .required(true),
                )
                .arg(
                    Arg::with_name("options")
                        .short("o")
                        .multiple(true)
                        .help("filesystem options")
                        .takes_value(true)
                        .value_delimiter(","),
                ),
        )
        .subcommand(
            App::new("list")
                .about("list exported volumes on server")
                .arg(
                    Arg::with_name("host")
                        .required(true)
                        .help("Fileserver Address"),
                ),
        )
        .subcommand(
            App::new("info").about("list server info").arg(
                Arg::with_name("host")
                    .required(true)
                    .help("Fileserver Address"),
            ),
        )
        .get_matches();

    match args.subcommand() {
        ("list", Some(cmd)) => {
            list_volumes(cmd.value_of("host").unwrap());
        }
        ("serve", Some(cmd)) => {
            let config_file = cmd.value_of("config").unwrap();
            let port: u32 = cmd.value_of("port").unwrap().parse()?;
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(host_server(&config_file.to_string(), port));
        }
        ("mount", Some(cmd)) => {
            let host = cmd.value_of("host").unwrap();
            let mountpoint = cmd.value_of("mountpoint").unwrap();
            let options_values = cmd.values_of("options");
            let mut options = vec![];
            if options_values.is_some() {
                options = options_values.unwrap().collect();
            }
            #[cfg(feature = "fuse_client")]
                mount_fs(host, mountpoint, options);
            #[cfg(not(feature = "fuse_client"))]
            mountUnavailable();
        }
        ("info", Some(cmd)) => {
            let host = cmd.value_of("host").unwrap();
            server_info(host);
        }
        _ => {
            println!("{}", args.usage());
            return Ok(());
        }
    }

    return Ok(());
}

fn list_volumes(host: &str) {
    let (host, _) = resolve_host(host);
    let mut con = ServerConnection::new(&host.to_string());
    let vols = unwrap_or_err(con.list_volumes(), "");
    if vols.is_empty() {
        print!("Volumes : None\n");
    } else {
        println!("Volumes :");
        for vol in vols.iter() {
            print!("\t• {}\n", vol);
        }
    }
}

fn server_info(host: &str) {
    let (host, _) = resolve_host(host);
    let mut con = ServerConnection::new(&host.to_string());
    let info = unwrap_or_err(con.server_info(), "");
    match info {
        Response::ServerInfo { name, version } => {
            println!("ShareFS Server {}", version);
            println!("Name : {}", name);
        }
        _ => {}
    }
    let vols = unwrap_or_err(con.list_volumes(), "");
    if vols.is_empty() {
        print!("Volumes : None\n");
    } else {
        println!("Volumes :");
        for vol in vols.iter() {
            print!("\t• {}\n", vol);
        }
    }
}

async fn host_server(config: &String, port: u32) {
    let mut fs = unwrap_or_err(
        FileServer::new(&config, port).await,
        "Could not instantiate server",
    );
    unwrap_or_err(fs.run().await, "Server Error");
}

#[cfg(not(feature = "fuse_client"))]
fn mountUnavailable() {
    println!("The mounting feature was disabled during compilation");
}

#[cfg(feature = "fuse_client")]
fn mount_fs(host: &str, mountpoint: &str, options: Vec<&str>) {
    // Resolving host and determining the volume
    let (addr, volume) = resolve_host(host);
    if volume.is_empty() {
        quit_error("No Volume specified");
    }

    println!("Mounting {} on {:?}", volume, addr);

    // Handshake
    let mut srv = ServerConnection::new(&addr.to_string());
    let vol_id = unwrap_or_err(srv.lookup_volume(&volume), "");

    // Creating the Filesystem and Connection
    let fsapi = VolumeConnection::new(&addr.to_string(), vol_id);
    let fs = shfs_fuse_fs::Filesystem { api: fsapi };

    // Parsing Filesystem Options
    let mut fuse_options = vec![];
    for e in options.iter() {
        fuse_options.push("-o");
        fuse_options.push(e);
    }
    let fuse_options = fuse_options
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();

    println!("Filesystem mounted at {}", mountpoint);

    // FUSE Mount
    let x = fuse::mount(fs, &mountpoint, &fuse_options);
    if x.is_err() {
        panic!("{}", x.unwrap_err());
    }
    return;
}

/// Resolving the provided host returning the Address and the specified volume.
/// # Arguments
/// * `host` - Host String with following format: Host:Port/Volume
/// # Examples
/// `host` can provide either an IP Address or a Hostname: fileserver.com:30/Vol or 127.0.0.1:30/Vol
fn resolve_host(host: &str) -> (SocketAddr, String) {
    let host: Vec<&str> = host.split("/").collect();
    let volume = *unpack_or_default(host.get(1), &"");
    let mut addr: Result<SocketAddr, AddrParseError> = host.get(0).unwrap().parse();
    if addr.is_err() {
        let ip: Result<Ipv4Addr, AddrParseError> = host.get(0).unwrap().parse();
        if ip.is_err() {
            let host: Vec<&str> = host.get(0).unwrap().split(":").collect();
            let lookup = lookup_host(host.get(0).unwrap()).unwrap();
            let mut ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
            for e in lookup.iter() {
                match e {
                    IpAddr::V4(ip_addr) => {
                        ip = IpAddr::V4(*ip_addr);
                    }
                    _ => continue,
                }
            }
            if ip == IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)) {
                panic!("Cannot resolve hostname");
            }
            if host.len() > 1 {
                addr = Ok(SocketAddr::new(ip, host.get(1).unwrap().parse().unwrap()));
            } else {
                addr = Ok(SocketAddr::new(ip, 30));
            }
        } else {
            addr = Ok(SocketAddr::new(IpAddr::V4(ip.unwrap()), 30));
        }
    }
    return (addr.unwrap(), volume.to_string());
}
