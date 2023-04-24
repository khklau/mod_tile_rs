pub const fn size_of_return_type<T, U>(_: fn(T) -> U) -> usize
{
    std::mem::size_of::<U>()
}
