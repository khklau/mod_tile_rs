use crate::interface::communication::{
    CommunicationError, BidirectionalChannel,
};
use crate::interface::context::RequestContext;
use crate::schema::apache2::error::InvalidConfigError;
use crate::schema::apache2::config::ModuleConfig;

use std::io::Read;
use std::io::Write;
use std::io::ErrorKind::TimedOut;
use std::option::Option;
use std::os::unix::net::UnixStream;
use std::result::Result;

pub struct RenderdSocket {
    socket: UnixStream,
}

#[cfg(not(test))]
use std::path::Path;
#[cfg(not(test))]
impl RenderdSocket {
    pub fn new(config: &ModuleConfig) -> Result<RenderdSocket, InvalidConfigError> {
        let path = Path::new(&config.renderd.ipc_uri);
        match UnixStream::connect(path) {
            Err(ioerr) => Err(
                InvalidConfigError {
                    entry: String::from("ipc_uri"),
                    reason: ioerr.to_string(),
                }
            ),
            Ok(socket) => {
                let availability_timeout = config.renderd.availability_timeout.clone();
                if !availability_timeout.is_zero() {
                    if let Err(ioerr) = socket.set_write_timeout(Some(availability_timeout)) {
                        return Err(
                            InvalidConfigError {
                                entry: String::from("availability_timeout"),
                                reason: ioerr.to_string(),
                            }
                        );
                    }
                }
                let render_timeout = config.renderd.render_timeout.clone();
                if !render_timeout.is_zero() {
                    if let Err(ioerr) = socket.set_read_timeout(Some(render_timeout)) {
                        return Err(
                            InvalidConfigError {
                                entry: String::from("render_timeout"),
                                reason: ioerr.to_string(),
                            }
                        );
                    }
                }
                Ok(
                    RenderdSocket {
                        socket,
                    }
                )
            }
        }
    }
}

#[cfg(test)]
impl RenderdSocket {
    pub fn new(_config: &ModuleConfig) -> Result<RenderdSocket, InvalidConfigError> {
        match UnixStream::pair() {
            Err(ioerr) => Err(
                InvalidConfigError {
                    entry: String::from("ipc_uri"),
                    reason: ioerr.to_string(),
                }
            ),
            Ok((client_socket, _)) => Ok(
                RenderdSocket {
                    socket: client_socket
                }
            )
        }
    }
}

impl BidirectionalChannel for RenderdSocket {
    fn send_blocking_request(
        &mut self,
        _context: &RequestContext,
        request: &[u8],
        response_buffer: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, CommunicationError> {
        let mut output = match response_buffer {
            Some(buffer) => buffer,
            None => Vec::new()
        };
        if let Err(ioerr) = self.socket.write_all(request) {
            match ioerr.kind() {
                TimedOut => return Err(
                    CommunicationError::TimeoutError
                ),
                _ => return Err(
                    CommunicationError::Io(ioerr)
                )
            }
        }
        self.socket.flush()?;
        let _bytes_read = match self.socket.read_to_end(&mut output) {
            Ok(bytes_read) => bytes_read,
            Err(ioerr) => match ioerr.kind() {
                TimedOut => return Err(
                    CommunicationError::TimeoutError
                ),
                _ => return Err(
                    CommunicationError::Io(ioerr)
                )
            }
        };
        return Ok(output);
    }
}
