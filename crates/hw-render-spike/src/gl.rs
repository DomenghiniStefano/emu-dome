//! The OpenGL side: a desktop GL context for a core that asks `SET_HW_RENDER` for OpenGL, the
//! framebuffer the core draws into, and the two ways out of GL measured by the spike:
//! `glReadPixels` (CPU copy) or a texture whose memory is a Vulkan image (zero copy,
//! `GL_EXT_memory_object` + `GL_EXT_semaphore`, Win32 handles).
//!
//! The context is wgpu-hal's own GL context (`Backends::GL`): on Windows a WGL core-profile
//! context on a window wgpu-hal creates and never shows (no `WS_VISIBLE`, so no focus taken);
//! on Linux/Android an EGL context. The spike borrows it and creates nothing else with it.

use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

use anyhow::{anyhow, bail, Context, Result};
use glow::HasContext;
use libloading::Library;

// GL_EXT_memory_object / GL_EXT_semaphore / *_win32 enums (Khronos gl.xml).
const GL_TEXTURE_TILING_EXT: u32 = 0x9580;
const GL_DEDICATED_MEMORY_OBJECT_EXT: u32 = 0x9581;
const GL_OPTIMAL_TILING_EXT: u32 = 0x9584;
const GL_HANDLE_TYPE_OPAQUE_WIN32_EXT: u32 = 0x9587;
pub const GL_LAYOUT_SHADER_READ_ONLY_EXT: u32 = 0x9591;

/// The framebuffer `get_current_framebuffer` hands to the core.
static FBO: AtomicU32 = AtomicU32::new(0);

/// How GL function addresses are found: `wglGetProcAddress` then `opengl32.dll`'s exports
/// (GL 1.1 entry points) with WGL, `eglGetProcAddress` with EGL. Chosen at run time by which
/// library loads, so the code has no per-OS branch.
struct Loader {
    get_proc: unsafe extern "system" fn(*const c_char) -> *const c_void,
    lib: Library,
}

// SAFETY: a library handle and a function pointer, both valid for the whole process.
unsafe impl Send for Loader {}
unsafe impl Sync for Loader {}

static LOADER: OnceLock<Loader> = OnceLock::new();

fn loader() -> Result<&'static Loader> {
    if let Some(l) = LOADER.get() {
        return Ok(l);
    }
    // SAFETY: system GL libraries, loaded once and never unloaded.
    let l = unsafe {
        [
            ("opengl32.dll", "wglGetProcAddress"),
            ("libEGL.so.1", "eglGetProcAddress"),
            ("libEGL.so", "eglGetProcAddress"),
        ]
        .iter()
        .find_map(|(lib, sym)| {
            let lib = Library::new(*lib).ok()?;
            let get_proc = *lib
                .get::<unsafe extern "system" fn(*const c_char) -> *const c_void>(
                    format!("{sym}\0").as_bytes(),
                )
                .ok()?;
            Some(Loader { get_proc, lib })
        })
        .ok_or_else(|| anyhow!("no GL loader library (opengl32.dll, libEGL.so)"))?
    };
    Ok(LOADER.get_or_init(|| l))
}

fn proc_address(name: &CStr) -> *const c_void {
    let Ok(l) = loader() else {
        return std::ptr::null();
    };
    // SAFETY: a C string; the GL context is current on this thread (the caller holds it).
    let p = unsafe { (l.get_proc)(name.as_ptr()) };
    // wglGetProcAddress returns small sentinels instead of null for unknown names.
    if !matches!(p as isize, -1..=3) {
        return p;
    }
    // SAFETY: looking up an export by name.
    unsafe {
        l.lib
            .get::<*const c_void>(name.to_bytes_with_nul())
            .map(|s| *s)
            .unwrap_or(std::ptr::null())
    }
}

/// `retro_hw_render_callback::get_proc_address`.
pub unsafe extern "C" fn get_proc_address(
    sym: *const c_char,
) -> Option<libretro_sys::retro_proc_address_t> {
    if sym.is_null() {
        return None;
    }
    let p = proc_address(CStr::from_ptr(sym));
    if p.is_null() {
        None
    } else {
        Some(std::mem::transmute::<
            *const c_void,
            libretro_sys::retro_proc_address_t,
        >(p))
    }
}

/// `retro_hw_render_callback::get_current_framebuffer`.
pub unsafe extern "C" fn get_current_framebuffer() -> usize {
    FBO.load(Ordering::Relaxed) as usize
}

/// The external-object entry points glow does not wrap.
struct ExtFns {
    create_memory_objects: unsafe extern "system" fn(c_int, *mut c_uint),
    delete_memory_objects: unsafe extern "system" fn(c_int, *const c_uint),
    memory_object_parameteriv: unsafe extern "system" fn(c_uint, u32, *const c_int),
    import_memory_win32_handle: unsafe extern "system" fn(c_uint, u64, u32, *mut c_void),
    tex_storage_mem_2d: unsafe extern "system" fn(u32, c_int, u32, c_int, c_int, c_uint, u64),
    gen_semaphores: unsafe extern "system" fn(c_int, *mut c_uint),
    delete_semaphores: unsafe extern "system" fn(c_int, *const c_uint),
    import_semaphore_win32_handle: unsafe extern "system" fn(c_uint, u32, *mut c_void),
    signal_semaphore:
        unsafe extern "system" fn(c_uint, c_uint, *const c_uint, c_uint, *const c_uint, *const u32),
    wait_semaphore:
        unsafe extern "system" fn(c_uint, c_uint, *const c_uint, c_uint, *const c_uint, *const u32),
}

impl ExtFns {
    fn load() -> Result<Self> {
        fn f<T: Copy>(name: &str) -> Result<T> {
            let c = CString::new(name)?;
            let p = proc_address(&c);
            if p.is_null() {
                bail!("GL function {name} not found");
            }
            // SAFETY: T is the function pointer type of `name` (declared above).
            Ok(unsafe { std::mem::transmute_copy::<*const c_void, T>(&p) })
        }
        Ok(Self {
            create_memory_objects: f("glCreateMemoryObjectsEXT")?,
            delete_memory_objects: f("glDeleteMemoryObjectsEXT")?,
            memory_object_parameteriv: f("glMemoryObjectParameterivEXT")?,
            import_memory_win32_handle: f("glImportMemoryWin32HandleEXT")?,
            tex_storage_mem_2d: f("glTexStorageMem2DEXT")?,
            gen_semaphores: f("glGenSemaphoresEXT")?,
            delete_semaphores: f("glDeleteSemaphoresEXT")?,
            import_semaphore_win32_handle: f("glImportSemaphoreWin32HandleEXT")?,
            signal_semaphore: f("glSignalSemaphoreEXT")?,
            wait_semaphore: f("glWaitSemaphoreEXT")?,
        })
    }
}

/// Win32 handles of a Vulkan image's memory and of two semaphores, exported for GL. GL does
/// not take ownership of Win32 handles; the spike leaves them to process exit (a product would
/// `CloseHandle` them after the import).
pub struct ExportedHandles {
    pub memory: isize,
    pub memory_size: u64,
    pub gl_done: isize,
    pub vk_done: isize,
}

/// The colour attachment of our framebuffer.
enum Colour {
    /// A plain GL texture (readback path).
    Texture,
    /// A texture on imported Vulkan memory (zero-copy path), with its semaphores.
    Imported {
        ext: ExtFns,
        memory: c_uint,
        gl_done: c_uint,
        vk_done: c_uint,
        /// Whether the Vulkan side has signalled `vk_done` since GL last waited on it.
        vk_pending: bool,
    },
}

/// The framebuffer the core draws into: colour + depth/stencil, `width` x `height` (the
/// core's maximum frame size; a frame uses its bottom-left corner, GL convention).
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    fbo: glow::NativeFramebuffer,
    texture: glow::NativeTexture,
    depth: glow::NativeRenderbuffer,
    colour: Colour,
}

/// What the context is, for the report.
pub fn describe(gl: &glow::Context) -> String {
    // SAFETY: plain queries on the current context.
    unsafe {
        format!(
            "{} / {} / GLSL {}",
            gl.get_parameter_string(glow::VERSION),
            gl.get_parameter_string(glow::RENDERER),
            gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION)
        )
    }
}

/// Whether the context can import Vulkan memory and semaphores through Win32 handles.
pub fn has_win32_interop(gl: &glow::Context) -> bool {
    let ext = gl.supported_extensions();
    ["GL_EXT_memory_object_win32", "GL_EXT_semaphore_win32"]
        .iter()
        .all(|e| ext.contains(*e))
}

impl Framebuffer {
    /// # Safety
    /// The context is current on this thread.
    pub unsafe fn new(
        gl: &glow::Context,
        width: u32,
        height: u32,
        import: Option<&ExportedHandles>,
    ) -> Result<Self> {
        let texture = gl.create_texture().map_err(|e| anyhow!("texture: {e}"))?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        let colour = match import {
            None => {
                gl.tex_storage_2d(
                    glow::TEXTURE_2D,
                    1,
                    glow::RGBA8,
                    width as i32,
                    height as i32,
                );
                Colour::Texture
            }
            Some(h) => {
                let ext = ExtFns::load()?;
                let mut memory = 0;
                (ext.create_memory_objects)(1, &mut memory);
                let dedicated: c_int = 1;
                (ext.memory_object_parameteriv)(memory, GL_DEDICATED_MEMORY_OBJECT_EXT, &dedicated);
                (ext.import_memory_win32_handle)(
                    memory,
                    h.memory_size,
                    GL_HANDLE_TYPE_OPAQUE_WIN32_EXT,
                    h.memory as *mut c_void,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    GL_TEXTURE_TILING_EXT,
                    GL_OPTIMAL_TILING_EXT as i32,
                );
                (ext.tex_storage_mem_2d)(
                    glow::TEXTURE_2D,
                    1,
                    glow::RGBA8,
                    width as c_int,
                    height as c_int,
                    memory,
                    0,
                );
                let mut sems = [0; 2];
                (ext.gen_semaphores)(2, sems.as_mut_ptr());
                (ext.import_semaphore_win32_handle)(
                    sems[0],
                    GL_HANDLE_TYPE_OPAQUE_WIN32_EXT,
                    h.gl_done as *mut c_void,
                );
                (ext.import_semaphore_win32_handle)(
                    sems[1],
                    GL_HANDLE_TYPE_OPAQUE_WIN32_EXT,
                    h.vk_done as *mut c_void,
                );
                Colour::Imported {
                    ext,
                    memory,
                    gl_done: sems[0],
                    vk_done: sems[1],
                    vk_pending: false,
                }
            }
        };
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        gl.bind_texture(glow::TEXTURE_2D, None);

        let depth = gl
            .create_renderbuffer()
            .map_err(|e| anyhow!("renderbuffer: {e}"))?;
        gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth));
        gl.renderbuffer_storage(
            glow::RENDERBUFFER,
            glow::DEPTH24_STENCIL8,
            width as i32,
            height as i32,
        );
        gl.bind_renderbuffer(glow::RENDERBUFFER, None);

        let fbo = gl
            .create_framebuffer()
            .map_err(|e| anyhow!("framebuffer: {e}"))?;
        gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
        gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(texture),
            0,
        );
        gl.framebuffer_renderbuffer(
            glow::FRAMEBUFFER,
            glow::DEPTH_STENCIL_ATTACHMENT,
            glow::RENDERBUFFER,
            Some(depth),
        );
        let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
        // Clear once, so a frame that leaves pixels untouched reads as black, not garbage.
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        if status != glow::FRAMEBUFFER_COMPLETE {
            bail!("framebuffer incomplete: 0x{status:x}");
        }
        let err = gl.get_error();
        if err != glow::NO_ERROR {
            bail!("GL error 0x{err:x} while creating the framebuffer");
        }
        FBO.store(fbo.0.get(), Ordering::Relaxed);
        Ok(Self {
            width,
            height,
            fbo,
            texture,
            depth,
            colour,
        })
    }

    /// Before `retro_run` on the zero-copy path: GL waits until Vulkan has finished reading
    /// the image of the previous frame (and takes it back from `SHADER_READ_ONLY`).
    ///
    /// # Safety
    /// The context is current.
    pub unsafe fn acquire(&mut self) {
        if let Colour::Imported {
            ext,
            vk_done,
            vk_pending,
            ..
        } = &mut self.colour
        {
            if *vk_pending {
                let tex = self.texture.0.get();
                (ext.wait_semaphore)(
                    *vk_done,
                    0,
                    std::ptr::null(),
                    1,
                    &tex,
                    &GL_LAYOUT_SHADER_READ_ONLY_EXT,
                );
                *vk_pending = false;
            }
        }
    }

    /// After `retro_run` on the zero-copy path: GL signals that the frame is written and
    /// moves the image to `SHADER_READ_ONLY` for Vulkan. The caller then submits the Vulkan
    /// work that waits on `gl_done` and signals `vk_done`, and calls [`Self::vk_signalled`].
    ///
    /// # Safety
    /// The context is current.
    pub unsafe fn release(&self, gl: &glow::Context) {
        if let Colour::Imported { ext, gl_done, .. } = &self.colour {
            let tex = self.texture.0.get();
            (ext.signal_semaphore)(
                *gl_done,
                0,
                std::ptr::null(),
                1,
                &tex,
                &GL_LAYOUT_SHADER_READ_ONLY_EXT,
            );
            gl.flush();
        }
    }

    pub fn vk_signalled(&mut self) {
        if let Colour::Imported { vk_pending, .. } = &mut self.colour {
            *vk_pending = true;
        }
    }

    /// Reads the bottom-left `width` x `height` of the framebuffer (rows bottom-up, RGBA).
    ///
    /// # Safety
    /// The context is current.
    pub unsafe fn read(&self, gl: &glow::Context, width: u32, height: u32, out: &mut Vec<u8>) {
        out.resize((width * height * 4) as usize, 0);
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, Some(self.fbo));
        gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
        gl.read_pixels(
            0,
            0,
            width as i32,
            height as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(out)),
        );
        gl.bind_framebuffer(glow::READ_FRAMEBUFFER, None);
    }

    /// # Safety
    /// The context is current; the Vulkan side no longer uses the shared memory.
    pub unsafe fn destroy(self, gl: &glow::Context) {
        FBO.store(0, Ordering::Relaxed);
        gl.delete_framebuffer(self.fbo);
        gl.delete_renderbuffer(self.depth);
        gl.delete_texture(self.texture);
        if let Colour::Imported {
            ext,
            memory,
            gl_done,
            vk_done,
            ..
        } = &self.colour
        {
            let sems = [*gl_done, *vk_done];
            (ext.delete_semaphores)(2, sems.as_ptr());
            (ext.delete_memory_objects)(1, memory);
        }
        gl.finish();
    }
}

/// Loader check at start, so a missing library fails before the core is touched.
pub fn init_loader() -> Result<()> {
    loader().map(|_| ()).context("GL loader")
}
