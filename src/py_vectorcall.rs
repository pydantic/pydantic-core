use pyo3::{prelude::*, AsPyPointer};

pub fn py_vectorcall<'py>(obj: &'py PyAny, args: &[&PyAny]) -> PyResult<&'py PyAny> {
    match args.len() {
        0 => obj.call0(),
        // match length i with total args slice of i+1 (+1 for self arg to fill in by python if needed)
        1 => py_vectorcall_fixed_size_with_offset::<2>(obj, args),
        2 => py_vectorcall_fixed_size_with_offset::<3>(obj, args),
        3 => py_vectorcall_fixed_size_with_offset::<4>(obj, args),
        4 => py_vectorcall_fixed_size_with_offset::<5>(obj, args),
        5 => py_vectorcall_fixed_size_with_offset::<6>(obj, args),
        _ => py_vectorcall_variable_size(obj, args),
    }
}

fn py_vectorcall_fixed_size_with_offset<'py, const N: usize>(obj: &'py PyAny, args: &[&PyAny]) -> PyResult<&'py PyAny> {
    let nargs = args.len();
    // N should be the number of args, plus one, so that we can allocate an empty pointer
    // for vectorcall to use as the self argument
    debug_assert!(N >= 1);
    debug_assert_eq!(nargs + 1, N);
    let mut args_ptrs = [std::ptr::null_mut(); N];
    let args_slice = &mut args_ptrs[1..];
    for (arg_ptr, arg) in args_slice.iter_mut().zip(args) {
        *arg_ptr = arg.as_ptr();
    }
    // TODO add offset to denote vectorcall allowed to use args_ptrs[0]
    let nargsf = nargs;
    unsafe {
        obj.py().from_owned_ptr_or_err(pyo3::ffi::PyObject_Vectorcall(
            obj.as_ptr(),
            args_slice.as_ptr(),
            nargsf,
            std::ptr::null_mut(),
        ))
    }
}

fn py_vectorcall_variable_size<'py>(obj: &'py PyAny, args: &[&PyAny]) -> PyResult<&'py PyAny> {
    let nargs = args.len();
    let mut args_ptrs: Vec<*mut pyo3::ffi::PyObject> = std::iter::once(std::ptr::null_mut())
        .chain(args.iter().map(|arg| arg.as_ptr()))
        .collect();
    // TODO add offset to denote vectorcall allowed to use args_ptrs[0]
    let args = unsafe { args_ptrs.as_mut_ptr().offset(1) };
    let nargsf = nargs;
    unsafe {
        obj.py().from_owned_ptr_or_err(pyo3::ffi::PyObject_Vectorcall(
            obj.as_ptr(),
            args,
            nargsf,
            std::ptr::null_mut(),
        ))
    }
}
