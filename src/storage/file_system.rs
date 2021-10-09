use crate::apache2::virtual_host::VirtualHostContext;

use std::io::Write;
use std::result::Result;

pub fn initialise(
    context : &mut VirtualHostContext,
) -> Result<(), std::io::Error> {
    context.trace_file.borrow_mut().write_all(b"storage::file_system::initialise - start\n")?;
    context.trace_file.borrow_mut().write_all(b"storage::file_system::initialise - finish\n")?;
    Ok(())
}
