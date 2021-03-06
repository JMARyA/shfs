use rich::{unpack_or_default, unwrap_or_err};
use shfs_api::calls::{Call};
use shfs_api::config::ServerConfig;
use shfs_api::responses::Response;
use shfs_api::volume::Volume;
use std::io::{Read};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// File Server Object
pub struct FileServer {
    listener: TcpListener,
    config: ServerConfig,
    volumes: Vec<Volume>,
}

impl FileServer {
    /// Returns a new [FileServer]
    /// # Arguments
    /// * `config` - The Path to the config file
    /// * `port` - The port to use
    pub async fn new(config: &String, port: u32) -> Result<FileServer, std::io::Error> {
        println!("Starting Server on port {}", port);
        let listener = TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
        let mut conf_file = unwrap_or_err(std::fs::File::open(config), "Config file could not be opened");
        println!("Reading config file {}", config);
        let mut buf = vec![];
        unwrap_or_err(conf_file.read_to_end(&mut buf), "Error receiving call");
        let config: ServerConfig = serde_json::from_slice(&buf).expect("");
        let mut volumes = vec![];
        for vol in &config.volumes {
            volumes.push(Volume::new(vol));
        }
        return Ok(FileServer {
            listener,
            config,
            volumes,
        });
    }

    async fn handle_cmd(&mut self, mut stream: TcpStream) {
        let mut buf = [0; 1024];
        let n = match stream.read(&mut buf).await {
            Ok(n) if n == 0 => {
                return;
            }
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to read from socket; err = {:?}", e);
                return;
            }
        };
        let data = &buf[0..n];
        // TODO : Implement better reading
        let obj: Call = serde_json::from_slice(&data).expect("What happened?");
        //println!("{:?}", obj);

        let resp = match obj {
            Call::ReadDir { info, path } => {
                let data = self.volumes[info.volume_id as usize].api.readdir(&path);
                Response::ReadDir { data: data }
            }
            Call::GetEntry { info, path } => {
                let data = self.volumes[info.volume_id as usize].api.get_entry(&path);
                if data.is_err() {
                    Response::IOError {
                        error: data.unwrap_err().raw_os_error().unwrap(),
                    }
                } else {
                    Response::GetEntry {
                        data: data.unwrap(),
                    }
                }
            }
            Call::GetEntryFromInode { info, ino } => {
                let data = self.volumes[info.volume_id as usize]
                    .api
                    .get_entry_from_inode(ino);
                if data.is_err() {
                    Response::IOError {
                        error: data.unwrap_err().raw_os_error().unwrap(),
                    }
                } else {
                    Response::GetEntry {
                        data: data.unwrap(),
                    }
                }
            }
            Call::GetPathFromInode { info, ino } => {
                let data = self.volumes[info.volume_id as usize]
                    .api
                    .get_path_from_inode(ino);
                if data.is_err() {
                    Response::IOError {
                        error: data.unwrap_err().raw_os_error().unwrap(),
                    }
                } else {
                    Response::GetPath {
                        data: data.unwrap(),
                    }
                }
            }
            Call::Read {
                info,
                ino,
                offset,
                size,
            } => {
                let data = self.volumes[info.volume_id as usize]
                    .api
                    .read(ino, offset, size);
                if data.is_err() {
                    Response::IOError {
                        error: data.unwrap_err().raw_os_error().unwrap(),
                    }
                } else {
                    Response::Read {
                        data: data.unwrap(),
                    }
                }
            }
            Call::Rename {
                info,
                parent,
                name,
                nparent,
                nname,
            } => {
                let ro = self.check_read_only(info.volume_id as usize);
                if ro.is_err() {
                    let ret = ro.unwrap_err();
                    ret
                } else {
                    let data = self.volumes[info.volume_id as usize]
                        .api
                        .rename(parent, &name, nparent, &nname);
                    if data.is_err() {
                        Response::IOError {
                            error: data.unwrap_err().raw_os_error().unwrap(),
                        }
                    } else {
                        Response::Ok {}
                    }
                }
            }
            Call::Mkdir { info, parent, name } => {
                let ro = self.check_read_only(info.volume_id as usize);
                if ro.is_err() {
                    let ret = ro.unwrap_err();
                    ret
                } else {
                    let data = self.volumes[info.volume_id as usize]
                        .api
                        .mkdir(parent, &name);
                    if data.is_err() {
                        Response::IOError {
                            error: data.unwrap_err().raw_os_error().unwrap(),
                        }
                    } else {
                        Response::GetEntry {
                            data: data.unwrap(),
                        }
                    }
                }
            }
            Call::Rmdir { info, parent, name } => {
                let ro = self.check_read_only(info.volume_id as usize);
                if ro.is_err() {
                    let ret = ro.unwrap_err();
                    ret
                } else {
                    let data = self.volumes[info.volume_id as usize]
                        .api
                        .rmdir(parent, &name);
                    if data.is_err() {
                        Response::IOError {
                            error: data.unwrap_err().raw_os_error().unwrap(),
                        }
                    } else {
                        Response::Ok {}
                    }
                }
            }
            Call::Create { info, parent, name } => {
                let ro = self.check_read_only(info.volume_id as usize);
                if ro.is_err() {
                    let ret = ro.unwrap_err();
                    ret
                } else {
                    let data = self.volumes[info.volume_id as usize]
                        .api
                        .create(parent, &name);
                    if data.is_err() {
                        Response::IOError {
                            error: data.unwrap_err().raw_os_error().unwrap(),
                        }
                    } else {
                        Response::GetEntry {
                            data: data.unwrap(),
                        }
                    }
                }
            }
            Call::Unlink { info, parent, name } => {
                let ro = self.check_read_only(info.volume_id as usize);
                if ro.is_err() {
                    let ret = ro.unwrap_err();
                    ret
                } else {
                    let data = self.volumes[info.volume_id as usize]
                        .api
                        .unlink(parent, &name);
                    if data.is_err() {
                        Response::IOError {
                            error: data.unwrap_err().raw_os_error().unwrap(),
                        }
                    } else {
                        Response::Ok {}
                    }
                }
            }
            Call::Write {
                info,
                ino,
                offset,
                data,
            } => {
                let ro = self.check_read_only(info.volume_id as usize);
                if ro.is_err() {
                    let ret = ro.unwrap_err();
                    ret
                } else {
                    let data = self.volumes[info.volume_id as usize]
                        .api
                        .write(ino, offset, &data);
                    if data.is_err() {
                        Response::IOError {
                            error: data.unwrap_err().raw_os_error().unwrap(),
                        }
                    } else {
                        Response::Write {
                            data: data.unwrap(),
                        }
                    }
                }
            }
            Call::ListVolumes {} => {
                let mut ret = vec![];
                for volume in &self.volumes {
                    if volume.config.discoverable.is_some() {
                        if !volume.config.discoverable.unwrap() {
                            continue;
                        }
                    }
                    ret.push(volume.name.to_string());
                }
                Response::ListVolumes { data: ret }
            }
            Call::VolumeLookup { name } => {
                let mut resp = Response::Error {
                    error: String::from("Volume not found"),
                };
                for (id, volume) in self.volumes.iter().enumerate() {
                    if volume.name == name {
                        resp = Response::VolumeLookup { id: id as u64 };
                    }
                }
                resp
            }
            Call::ServerInfo {} => {
                let mut name = "";
                if self.config.name.is_some() {
                    name = &self.config.name.as_ref().unwrap();
                }
                Response::ServerInfo {
                    name: name.to_string(),
                    version: option_env!("CARGO_PKG_VERSION").unwrap().to_string(),
                }
            }
            //_ => Response::invalid,
        };

        let mut resp = serde_json::to_vec(&resp).unwrap();

        let resp_comp = zstd::stream::encode_all(&resp[0..resp.len()], 5).unwrap();

        let size_resp = resp.len() * 8;
        let size_comp = resp_comp.len() * 8;
        //let perc: f64 = (size_comp as f64) / (size_resp as f64) * 100.0;
        /*println!(
            "SENDING SIZE {} bytes COMPRESSED {} bytes [{}%]",
            size_resp, size_comp, perc
        );*/

        if size_comp < size_resp {
            let obj = Response::Compressed { data: resp_comp };
            resp = serde_json::to_vec(&obj).unwrap();
        }

        unwrap_or_err(stream.write_all(&resp).await, "Error sending response");
    }

    /// Checks if the volume is read only.
    /// If the [Volume] is read only this will return a Read Only Error.
    /// # Arguments
    /// * `vol_id` - The ID of the [Volume]
    pub fn check_read_only(&self, vol_id: usize) -> Result<(), Response> {
        if unpack_or_default(self.volumes[vol_id].config.readonly, false) {
            return Err(Response::IOError { error: 30 });
        }
        return Ok(());
    }

    /// Infinite loop to run the server
    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        loop {
            let (socket, _) = self.listener.accept().await?;

            self.handle_cmd(socket).await;
        }
    }
}
