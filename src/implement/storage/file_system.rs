use crate::apache2::virtual_host::VirtualHost;

use std::result::Result;

pub fn initialise(
    _context : &mut VirtualHost,
) -> Result<(), std::io::Error> {
    Ok(())
}
