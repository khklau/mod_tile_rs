use crate::binding::renderd_protocol::{
    protoCmd_cmdDone,
    protoCmd_cmdIgnore,
    protoCmd_cmdNotDone,
};


pub enum RenderResponseVersion {
    Two = 2,
    Three = 3,
}

pub enum RenderResponseCommand {
    Done = protoCmd_cmdDone as isize,
    InvalidRequestIgnored = protoCmd_cmdIgnore as isize,
    NotDone = protoCmd_cmdNotDone as isize,
}
