use core::{cell::UnsafeCell, ptr::NonNull, marker::PhantomData, ops::{Deref, DerefMut}};


/// [`NonNull`] with the dereference traits implemented.
#[repr(transparent)]
pub struct NonNullPtr<T> {
    ptr: NonNull<T>,
    // This marker does not affect the variance but is required for
    // dropck to undestand that we logically own a `T`.
    //
    // TODO: Use `Unique<T>` when stabalized!
    _phantom: PhantomData<T>,
}

impl<T> NonNullPtr<T> {
    pub fn as_ptr(&self) -> *mut T { self.ptr.as_ptr() }
}

impl<T> Deref for NonNullPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: We have shared reference to the data.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for NonNullPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: We have exclusive reference to the data.
        unsafe { self.ptr.as_mut() }
    }
}

#[repr(transparent)]
pub struct LiminePtr<T> {
    ptr: Option<NonNull<T>>,
    // This marker does not affect the variance but is required for
    // dropck to undestand that we logically own a `T`.
    //
    // TODO: Use `Unique<T>` when stabalized!
    _phantom: PhantomData<T>,
}

impl<T> LiminePtr<T> {
    const DEFAULT: LiminePtr<T> = Self { ptr: None, _phantom: PhantomData };

    #[inline]
    pub fn as_ptr(&self) -> Option<*mut T> { Some(self.ptr?.as_ptr()) }

    #[inline]
    pub fn get<'a>(&self) -> Option<&'a T> {
        // SAFETY: According to the specication the bootloader provides
        // a aligned pointer and there is no public API to construct a [`LiminePtr`]
        // so, its safe to assume that the [`NonNull::as_ref`] are applied. If not,
        // its the bootloader's fault that they have violated the
        // specification!.
        //
        // Also, we have a shared reference to the data and there is no
        // legal way to mutate it, unless through [`LiminePtr::as_ptr`]
        // (requires pointer dereferencing which is unsafe) or [`LiminePtr::get_mut`]
        // (requires exclusive access to the [`LiminePtr`]).
        self.ptr.map(|e| unsafe { e.as_ref() })
    }

    #[inline]
    pub fn get_mut<'a>(&mut self) -> Option<&'a mut T> {
        // SAFETY: Check the safety for [`LiminePtr::get`] and we have
        // exclusive access to the data.
        self.ptr.as_mut().map(|e| unsafe { e.as_mut() })
    }
}



unsafe impl<T: Sync> Sync for LiminePtr<T> {}

type ArrayPtr<T> = NonNullPtr<NonNullPtr<T>>;

fn into_slice<T>(array: *const T, len: usize) -> &'static [T] {
    unsafe { core::slice::from_raw_parts(array, len) }
}

fn into_slice_mut<T>(array: *mut T, len: usize) -> &'static [T] {
    unsafe { core::slice::from_raw_parts_mut(array, len) }
}


#[repr(C)]
#[derive(Debug)]
pub struct LimineFramebuffer {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub reserved: [u8; 7],
    pub edid_size: u64,
    pub edid: *mut u8,
}

impl LimineFramebuffer {
    /// Returns the size of the framebuffer.
    pub fn size(&self) -> usize {
        self.pitch as usize * self.height as usize * (self.bpp as usize / 8)
    }
}


#[repr(C)]
#[derive(Debug)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    /// How many framebuffers are present.
    pub framebuffer_count: u64,
    pub framebuffers: &'static *const LimineFramebuffer,
}

impl LimineFramebufferResponse {
    pub fn framebuffers(&self) -> &'static [LimineFramebuffer] {
        into_slice(*self.framebuffers, self.framebuffer_count as usize)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct LimineFramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: UnsafeCell<LiminePtr<

    LimineFramebufferResponse>>,
}

impl LimineFramebufferRequest {
    pub const ID: [u64; 4] =
        [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, 0x9d5827dcd881dd75,
                0xa3148604f6fab11b];
    pub const fn new(revision: u64) -> Self {
        Self {
            id: Self::ID,
            revision,
            response: UnsafeCell::new(LiminePtr::DEFAULT),
        }
    }
    pub fn get_response(&self) -> LiminePtr<LimineFramebufferResponse> {
        unsafe { core::ptr::read_volatile(self.response.get()) }
    }
}
unsafe impl Sync for LimineFramebufferRequest {}
