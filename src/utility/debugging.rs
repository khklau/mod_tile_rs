use std::any::type_name;

pub fn function_name<F>(_: F) -> &'static str {
    type_name::<F>()
}
