use crate::binding::apache2::conn_rec;


pub struct Connection<'c> {
    pub record: &'c conn_rec,
}
