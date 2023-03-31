pub const fn size_of_return_type<F, T, U>(_f: F) -> usize
where
    F: FnOnce(T) -> U
{
    std::mem::size_of::<U>()
}
