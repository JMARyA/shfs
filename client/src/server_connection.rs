use super::udp_connection::UDPConnection;
use shfs_api::calls::Call;
use shfs_api::responses::Response;

/// General Connection to Server
pub struct ServerConnection {
    con: UDPConnection,
}

impl ServerConnection {
    pub fn new(addr: &String) -> ServerConnection {
        return ServerConnection {
            con: UDPConnection::new(addr),
        };
    }

    /// Lookup the ID of Volume
    pub fn lookup_volume(&mut self, name: &str) -> Result<u64, shfs_api::ApiError> {
        let req = Call::VolumeLookup {
            name: name.to_string(),
        };

        let obj = self.con.send_call(req);

        let ret = match obj {
            Response::VolumeLookup { id } => Ok(id),
            Response::Error { error } => Err(shfs_api::ApiError::new(&error)),
            _ => Err(shfs_api::ApiError::new(&String::new())),
        };

        return ret;
    }

    /// Get Server Info
    pub fn server_info(&mut self) -> Result<Response, shfs_api::ApiError> {
        let req = Call::ServerInfo {};
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::ServerInfo { .. } => Ok(obj),
            Response::Error { error } => Err(shfs_api::ApiError::new(&error)),
            _ => Err(shfs_api::ApiError::new(&String::new())),
        };

        return ret;
    }

    /// Get the List of Volumes
    pub fn list_volumes(&mut self) -> Result<Vec<String>, shfs_api::ApiError> {
        let req = Call::ListVolumes {};
        let obj = self.con.send_call(req);
        let ret = match obj {
            Response::ListVolumes { data } => Ok(data),
            Response::Error { error } => Err(shfs_api::ApiError::new(&error)),
            _ => Err(shfs_api::ApiError::new("")),
        };

        return ret;
    }
}
