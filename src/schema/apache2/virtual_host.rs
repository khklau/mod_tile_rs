use crate::binding::apache2::server_rec;


pub struct VirtualHost<'h> {
    pub record: &'h server_rec,
}
