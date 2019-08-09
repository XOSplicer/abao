use std::cell::Cell;

pub(crate) fn cell_from_mut<T: ?Sized>(t: &mut T) -> &Cell<T> {
    unsafe {
        &*(t as *mut T as *const Cell<T>)
    }
}

pub(crate) fn cell_as_slice_of_cells<T: >(cell: &Cell<[T]>) -> &[Cell<T>] {
    unsafe {
        &*(cell as *const Cell<[T]> as *const [Cell<T>])
    }
}