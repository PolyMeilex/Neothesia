pub struct SuperUnsafeDowncaster {}

impl<'a> SuperUnsafeDowncaster {
  pub unsafe fn downcast_ref<T>(self, ptr: *mut std::ffi::c_void) -> Option<&'a mut T> {
    Some(&mut *(ptr as *mut T))
  }
}

