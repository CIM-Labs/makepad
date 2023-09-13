#[allow(unused)]
use {
    std::rc::Rc,
    std::cell::{RefCell},
    std::ffi::CString,
    std::os::raw::{c_void},
    std::time::Instant,
    std::sync::mpsc,
    self::super::{
        android_media::CxAndroidMedia,
        jni_sys::jobject,
        android_jni::{self, *},
        android_keycodes::android_to_makepad_key_code,
        super::egl_sys::{self, LibEgl},
        ndk_sys,
    },
    self::super::super::{
        gl_sys,
        //libc_sys,
    },
    crate::{
        cx_api::{CxOsOp, CxOsApi},
        makepad_math::*,
        thread::Signal,
        live_id::LiveId,
        event::{
            VirtualKeyboardEvent,
            NetworkResponseEvent,
            NetworkResponse,
            HttpResponse,
            TouchPoint,
            TouchUpdateEvent,
            WindowGeomChangeEvent,
            TimerEvent,
            TextInputEvent,
            TextClipboardEvent,
            KeyEvent,
            KeyModifiers,
            KeyCode,
            Event,
            WindowGeom,
        },
        window::CxWindowPool,
        pass::CxPassParent,
        cx::{Cx, OsType, AndroidParams},
        gpu_info::GpuPerformance,
        os::cx_native::EventFlow,
        pass::{PassClearColor, PassClearDepth, PassId},
    }
};

impl Cx {
    pub fn main_loop(&mut self, from_java_rx: mpsc::Receiver<FromJavaMessage>) {
        
        self.android_load_dependencies();
        self.gpu_info.performance = GpuPerformance::Tier1;
        
        self.call_event_handler(&Event::Construct);
        self.redraw_all();
        
        while !self.os.quit {
            
            while let Ok(msg) = from_java_rx.try_recv() {
                match msg {
                    FromJavaMessage::SurfaceCreated {window} => unsafe {
                        self.os.display.as_mut().unwrap().update_surface(window);
                    },
                    FromJavaMessage::SurfaceDestroyed => unsafe {
                        self.os.display.as_mut().unwrap().destroy_surface();
                    },
                    FromJavaMessage::SurfaceChanged {
                        window,
                        width,
                        height,
                    } => {
                        unsafe {
                            self.os.display.as_mut().unwrap().update_surface(window);
                        }
                        self.os.display_size = dvec2(width as f64, height as f64);
                        let window_id = CxWindowPool::id_zero();
                        let window = &mut self.windows[window_id];
                        let old_geom = window.window_geom.clone();
                        let dpi_factor = window.dpi_override.unwrap_or(self.os.dpi_factor);
                        let size = self.os.display_size / dpi_factor;
                        window.window_geom = WindowGeom {
                            dpi_factor,
                            can_fullscreen: false,
                            xr_is_presenting: false,
                            is_fullscreen: true,
                            is_topmost: true,
                            position: dvec2(0.0, 0.0),
                            inner_size: size,
                            outer_size: size,
                        };
                        let new_geom = window.window_geom.clone();
                        self.call_event_handler(&Event::WindowGeomChange(WindowGeomChangeEvent {
                            window_id,
                            new_geom,
                            old_geom
                        }));
                        if let Some(main_pass_id) = self.windows[window_id].main_pass_id {
                            self.redraw_pass_and_child_passes(main_pass_id);
                        }
                        self.redraw_all();
                        self.os.first_after_resize = true;
                        self.call_event_handler(&Event::ClearAtlasses);
                    }
                    FromJavaMessage::Touch(mut touches) => {
                        let time = self.os.time_now();
                        let window = &mut self.windows[CxWindowPool::id_zero()];
                        let dpi_factor = window.dpi_override.unwrap_or(self.os.dpi_factor);
                        for touch in &mut touches {
                            // When the software keyboard shifted the UI in the vertical axis,
                            //we need to make the math here to keep touch events positions synchronized.
                            //if self.os.keyboard_visible {touch.abs.y += self.os.keyboard_panning_offset as f64};
                            //crate::log!("{} {:?} {} {}", time, touch.state, touch.uid, touch.abs);
                            touch.abs /= dpi_factor;
                        }
                        self.fingers.process_touch_update_start(time, &touches);
                        let e = Event::TouchUpdate(
                            TouchUpdateEvent {
                                time,
                                window_id: CxWindowPool::id_zero(),
                                touches,
                                modifiers: Default::default()
                            }
                        );
                        self.call_event_handler(&e);
                        let e = if let Event::TouchUpdate(e) = e {e}else {panic!()};
                        self.fingers.process_touch_update_end(&e.touches);
                    }
                    FromJavaMessage::Character {character} => {
                        if let Some(character) = char::from_u32(character) {
                            let e = Event::TextInput(
                                TextInputEvent {
                                    input: character.to_string(),
                                    replace_last: false,
                                    was_paste: false,
                                }
                            );
                            self.call_event_handler(&e);
                        }
                    }
                    FromJavaMessage::KeyDown {keycode, meta_state} => {
                        let e: Event;
                        let makepad_keycode = android_to_makepad_key_code(keycode);
                        if !makepad_keycode.is_unknown() {
                            let control = meta_state & ANDROID_META_CTRL_MASK != 0;
                            let alt = meta_state & ANDROID_META_ALT_MASK != 0;
                            let shift = meta_state & ANDROID_META_SHIFT_MASK != 0;
                            let is_shortcut = control || alt;
                            if is_shortcut {
                                if makepad_keycode == KeyCode::KeyC {
                                    let response = Rc::new(RefCell::new(None));
                                    e = Event::TextCopy(TextClipboardEvent {
                                        response: response.clone()
                                    });
                                    self.call_event_handler(&e);
                                    // let response = response.borrow();
                                    // if let Some(response) = response.as_ref(){
                                    //     to_java.copy_to_clipboard(response);
                                    // }
                                } else if makepad_keycode == KeyCode::KeyX {
                                    let response = Rc::new(RefCell::new(None));
                                    let e = Event::TextCut(TextClipboardEvent {
                                        response: response.clone()
                                    });
                                    self.call_event_handler(&e);
                                    // let response = response.borrow();
                                    // if let Some(response) = response.as_ref(){
                                    //     to_java.copy_to_clipboard(response);
                                    // }
                                } else if makepad_keycode == KeyCode::KeyV {
                                    //to_java.paste_from_clipboard();
                                }
                            } else {
                                e = Event::KeyDown(
                                    KeyEvent {
                                        key_code: makepad_keycode,
                                        is_repeat: false,
                                        modifiers: KeyModifiers {shift, control, alt, ..Default::default()},
                                        time: self.os.time_now()
                                    }
                                );
                                self.call_event_handler(&e);
                            }
                        }
                    }
                    FromJavaMessage::KeyUp {keycode: _} => {
                        /*match keycode {
                            KeyCode::LeftShift | KeyCode::RightShift => self.keymods.shift = false,
                            KeyCode::LeftControl | KeyCode::RightControl => self.keymods.ctrl = false,
                            KeyCode::LeftAlt | KeyCode::RightAlt => self.keymods.alt = false,
                            KeyCode::LeftSuper | KeyCode::RightSuper => self.keymods.logo = false,
                            _ => {}
                        }
                        self.event_handler.key_up_event(keycode, self.keymods);*/
                    }
                    FromJavaMessage::ResizeTextIME {keyboard_height, is_open} => {
                        let keyboard_height = (keyboard_height as f64) / self.os.dpi_factor;
                        if !is_open{
                            self.os.keyboard_closed = keyboard_height;
                        }
                        if is_open {
                            self.call_event_handler(&Event::VirtualKeyboard(VirtualKeyboardEvent::DidShow {
                                height: keyboard_height - self.os.keyboard_closed,
                                time: self.os.time_now()
                            }))
                        } 
                        else {
                            self.call_event_handler(&Event::VirtualKeyboard(VirtualKeyboardEvent::DidHide {
                                time: self.os.time_now()
                            }))
                        }
                        //self.panning_adjust_for_text_ime(keyboard_height);
                    }
                    FromJavaMessage::HttpResponse {request_id, metadata_id, status_code, headers, body} => {
                        let e = Event::NetworkResponses(vec![
                            NetworkResponseEvent {
                                request_id: LiveId(request_id),
                                response: NetworkResponse::HttpResponse(HttpResponse::new(
                                    LiveId(metadata_id),
                                    status_code,
                                    headers,
                                    Some(body)
                                ))
                            }
                        ]);
                        self.call_event_handler(&e);
                    }
                    FromJavaMessage::HttpRequestError {request_id, error, ..} => {
                        let e = Event::NetworkResponses(vec![
                            NetworkResponseEvent {
                                request_id: LiveId(request_id),
                                response: NetworkResponse::HttpRequestError(error)
                            }
                        ]);
                        self.call_event_handler(&e);
                    }
                    FromJavaMessage::MidiDeviceOpened {name, midi_device} => {
                        self.os.media.android_midi().lock().unwrap().midi_device_opened(name, midi_device);
                    }
                    FromJavaMessage::Pause => {
                        self.call_event_handler(&Event::Pause);
                    }
                    FromJavaMessage::Stop => {
                        
                    }
                    FromJavaMessage::Resume => {
                        if self.os.fullscreen {
                            unsafe {
                                let env = attach_jni_env();
                                android_jni::to_java_set_full_screen(env, true);
                            }
                        }
                        self.redraw_all();
                        self.reinitialise_media();
                    }
                    FromJavaMessage::Destroy => {
                        self.os.quit = true;
                    }
                    FromJavaMessage::Init(_) => {
                        panic!()
                    }
                }
            }
            
            if Signal::check_and_clear_ui_signal() {
                self.handle_media_signals();
                self.call_event_handler(&Event::Signal);
            }
            
            self.handle_platform_ops();
            if self.any_passes_dirty() || self.need_redrawing() || self.new_next_frames.len() != 0 {
                if self.new_next_frames.len() != 0 {
                    self.call_next_frame_event(self.os.time_now());
                }
                if self.need_redrawing() {
                    self.call_draw_event();
                    self.opengl_compile_shaders();
                }
                
                if self.os.first_after_resize {
                    self.os.first_after_resize = false;
                    self.redraw_all();
                }
                
                self.handle_repaint();
            }
            else {
                std::thread::sleep(std::time::Duration::from_millis(8));
            }
        }
    }
    
    pub fn android_entry<F>(activity: *const std::ffi::c_void, startup: F) where F: FnOnce() -> Box<Cx> + Send + 'static {
        let (from_java_tx, from_java_rx) = mpsc::channel();
        
        unsafe {android_jni::jni_init_globals(activity, from_java_tx)};
        
        // lets start a thread
        std::thread::spawn(move || {
            
            let mut cx = startup();
            
            let mut libegl = LibEgl::try_load().expect("Cant load LibEGL");
            
            let window = loop {
                match from_java_rx.try_recv() {
                    Ok(FromJavaMessage::Init(params)) => {
                        cx.os.dpi_factor = params.density;
                        cx.os_type = OsType::Android(params);
                    }
                    Ok(FromJavaMessage::SurfaceChanged {
                        window,
                        width,
                        height,
                    }) => {
                        cx.os.display_size = dvec2(width as f64, height as f64);
                        break window;
                    }
                    _ => {}
                }
            };
            
            let (egl_context, egl_config, egl_display) = unsafe {egl_sys::create_egl_context(
                &mut libegl,
                std::ptr::null_mut(),/* EGL_DEFAULT_DISPLAY */
                false,
            ).expect("Cant create EGL context")};
            
            unsafe {gl_sys::load_with( | s | {
                let s = CString::new(s).unwrap();
                libegl.eglGetProcAddress.unwrap()(s.as_ptr())
            })};
            
            let surface = unsafe {(libegl.eglCreateWindowSurface.unwrap())(
                egl_display,
                egl_config,
                window as _,
                std::ptr::null_mut(),
            )};
            
            if unsafe {(libegl.eglMakeCurrent.unwrap())(egl_display, surface, surface, egl_context)} == 0 {
                panic!();
            }
            cx.os.display = Some(CxAndroidDisplay {
                libegl,
                egl_display,
                egl_config,
                egl_context,
                surface,
                window
            });
            cx.main_loop(from_java_rx);
            
            let display = cx.os.display.take().unwrap();
            
            unsafe {
                (display.libegl.eglMakeCurrent.unwrap())(
                    display.egl_display,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                );
                (display.libegl.eglDestroySurface.unwrap())(display.egl_display, display.surface);
                (display.libegl.eglDestroyContext.unwrap())(display.egl_display, display.egl_context);
                (display.libegl.eglTerminate.unwrap())(display.egl_display);
            }
        });
    }
    
    /*
    pub fn from_java_on_resize_text_ime(&mut self, ime_height: i32) {
        let dt = crate::profile_start();
        self.os.keyboard_visible = true;
        self.panning_adjust_for_text_ime(ime_height);
        self.redraw_all();
        self.after_every_event(&to_java);
        crate::profile_end!(dt);
    }
    
    pub fn from_java_on_paste_from_clipboard(&mut self, content: Option<String>, to_java: AndroidToJava) {
        if let Some(text) = content {
            let e = Event::TextInput(
                TextInputEvent {
                    input: text,
                    replace_last: false,
                    was_paste: true,
                }
            );
            self.call_event_handler(&e);
            self.after_every_event(&to_java);
        }
    }
    
    pub fn from_java_on_cut_to_clipboard(&mut self, to_java: AndroidToJava) {
        let e = Event::TextCut(
            TextClipboardEvent {
                response: Rc::new(RefCell::new(None))
            }
        );
        self.call_event_handler(&e);
        self.after_every_event(&to_java);
    }
   */
    
    pub fn android_load_dependencies(&mut self) {
        for (path, dep) in &mut self.dependencies {
            if let Some(data) = unsafe {to_java_load_asset(path)} {
                dep.data = Some(Ok(Rc::new(data)))
            }
            else {
                let message = format!("cannot load dependency {}", path);
                crate::makepad_error_log::error!("Android asset failed: {}", message);
                dep.data = Some(Err(message));
            }
        }
    }
    
    pub fn draw_pass_to_fullscreen(
        &mut self,
        pass_id: PassId,
    ) {
        let draw_list_id = self.passes[pass_id].main_draw_list_id.unwrap();
        
        self.setup_render_pass(pass_id);
        
        // keep repainting in a loop
        self.passes[pass_id].paint_dirty = false;
        //let panning_offset = if self.os.keyboard_visible {self.os.keyboard_panning_offset} else {0};
        
        unsafe {
            gl_sys::Viewport(0, 0, self.os.display_size.x as i32, self.os.display_size.y as i32);
        }
        
        let clear_color = if self.passes[pass_id].color_textures.len() == 0 {
            self.passes[pass_id].clear_color
        }
        else {
            match self.passes[pass_id].color_textures[0].clear_color {
                PassClearColor::InitWith(color) => color,
                PassClearColor::ClearWith(color) => color
            }
        };
        let clear_depth = match self.passes[pass_id].clear_depth {
            PassClearDepth::InitWith(depth) => depth,
            PassClearDepth::ClearWith(depth) => depth
        };
        
        if !self.passes[pass_id].dont_clear {
            unsafe {
                //gl_sys::BindFramebuffer(gl_sys::FRAMEBUFFER, 0);
                gl_sys::ClearDepthf(clear_depth as f32);
                gl_sys::ClearColor(clear_color.x, clear_color.y, clear_color.z, clear_color.w);
                gl_sys::Clear(gl_sys::COLOR_BUFFER_BIT | gl_sys::DEPTH_BUFFER_BIT);
            }
        }
        Self::set_default_depth_and_blend_mode();
        
        let mut zbias = 0.0;
        let zbias_step = self.passes[pass_id].zbias_step;
        
        self.render_view(
            pass_id,
            draw_list_id,
            &mut zbias,
            zbias_step,
        );
        
        //to_java.swap_buffers();
        //unsafe {
        //direct_app.drm.swap_buffers_and_wait(&direct_app.egl);
        //}
    }
    
    pub (crate) fn handle_repaint(&mut self) {
        //opengl_cx.make_current();
        let mut passes_todo = Vec::new();
        self.compute_pass_repaint_order(&mut passes_todo);
        self.repaint_id += 1;
        for pass_id in &passes_todo {
            match self.passes[*pass_id].parent.clone() {
                CxPassParent::Window(_) => {
                    //let window = &self.windows[window_id];
                    self.draw_pass_to_fullscreen(*pass_id);
                    unsafe {
                        if let Some(display) = &mut self.os.display {
                            (display.libegl.eglSwapBuffers.unwrap())(display.egl_display, display.surface);
                            
                        }
                    }
                }
                CxPassParent::Pass(_) => {
                    //let dpi_factor = self.get_delegated_dpi_factor(parent_pass_id);
                    self.draw_pass_to_texture(*pass_id);
                },
                CxPassParent::None => {
                    self.draw_pass_to_texture(*pass_id);
                }
            }
        }
        
        
    }
    /*
    fn panning_adjust_for_text_ime(&mut self, android_ime_height: u32) {
        self.os.keyboard_visible = true;
        
        let screen_height = (self.os.display_size.y / self.os.dpi_factor) as i32;
        let vertical_offset = self.os.keyboard_trigger_position.y as i32;
        let ime_height = (android_ime_height as f64 / self.os.dpi_factor) as i32;
        
        // Make sure there is some room between the software keyword and the text input or widget that triggered
        // the TextIME event
        let vertical_space = ime_height / 3;
        
        let should_be_panned = vertical_offset > screen_height - ime_height;
        if should_be_panned {
            let panning_offset = vertical_offset - (screen_height - ime_height) + vertical_space;
            self.os.keyboard_panning_offset = (panning_offset as f64 * self.os.dpi_factor) as i32;
        } else {
            self.os.keyboard_panning_offset = 0;
        }
    }*/
    
    fn handle_platform_ops(&mut self) -> EventFlow {
        while let Some(op) = self.platform_ops.pop() {
            match op {
                CxOsOp::CreateWindow(window_id) => {
                    let window = &mut self.windows[window_id];
                    let dpi_factor = window.dpi_override.unwrap_or(self.os.dpi_factor);
                    let size = self.os.display_size / dpi_factor;
                    window.window_geom = WindowGeom {
                        dpi_factor,
                        can_fullscreen: false,
                        xr_is_presenting: false,
                        is_fullscreen: true,
                        is_topmost: true,
                        position: dvec2(0.0, 0.0),
                        inner_size: size,
                        outer_size: size,
                    };
                    window.is_created = true;
                },
                CxOsOp::SetCursor(_cursor) => {
                    //xlib_app.set_mouse_cursor(cursor);
                },
                CxOsOp::StartTimer {timer_id: _, interval: _, repeats: _} => {
                    //android_app.start_timer(timer_id, interval, repeats);
                    //to_java.schedule_timeout(timer_id as i64, (interval / 1000.0) as i64);
                },
                CxOsOp::StopTimer(_timer_id) => {
                    //to_java.cancel_timeout(timer_id as i64);
                    //android_app.stop_timer(timer_id);
                },
                CxOsOp::ShowTextIME(_area, _pos) => {
                    //self.os.keyboard_trigger_position = area.get_clipped_rect(self).pos;
                    unsafe {android_jni::to_java_show_keyboard(true);}
                },
                CxOsOp::HideTextIME => {
                    //self.os.keyboard_visible = false;
                    unsafe {android_jni::to_java_show_keyboard(false);}
                },
                CxOsOp::ShowClipboardActions(_selected) => {
                    //to_java.show_clipboard_actions(selected.as_str());
                },
                CxOsOp::HttpRequest {request_id, request} => {
                    unsafe {android_jni::to_java_http_request(request_id, request);}
                },
                _ => ()
            }
        }
        EventFlow::Poll
    }
}

impl CxOsApi for Cx {
    fn init_cx_os(&mut self) {
        self.live_registry.borrow_mut().package_root = Some("makepad".to_string());
        self.live_expand();
        self.live_scan_dependencies();
    }
    
    fn spawn_thread<F>(&mut self, f: F) where F: FnOnce() + Send + 'static {
        std::thread::spawn(f);
    }
}

impl Default for CxOs {
    fn default() -> Self {
        Self {
            last_time: Instant::now(),
            first_after_resize: true,
            display_size: dvec2(100., 100.),
            dpi_factor: 1.5,
            time_start: Instant::now(),
            keyboard_closed: 0.0, 
            //keyboard_visible: false,
            //keyboard_trigger_position: DVec2::default(),
            //keyboard_panning_offset: 0,
            media: CxAndroidMedia::default(),
            display: None,
            quit: false,
            fullscreen: false
        }
    }
}

pub struct CxAndroidDisplay {
    libegl: LibEgl,
    egl_display: egl_sys::EGLDisplay,
    egl_config: egl_sys::EGLConfig,
    egl_context: egl_sys::EGLContext,
    surface: egl_sys::EGLSurface,
    window: *mut ndk_sys::ANativeWindow,
    //event_handler: Box<dyn EventHandler>,
}


pub struct CxOs {
    pub last_time: Instant,
    pub first_after_resize: bool,
    pub display_size: DVec2,
    pub dpi_factor: f64,
    pub time_start: Instant,
    pub keyboard_closed: f64,
    
    //pub keyboard_visible: bool,
    //pub keyboard_trigger_position: DVec2,
    //pub keyboard_panning_offset: i32,
    
    pub quit: bool,
    pub fullscreen: bool,
    pub (crate) display: Option<CxAndroidDisplay>,
    pub (crate) media: CxAndroidMedia,
}


impl CxAndroidDisplay {
    unsafe fn destroy_surface(&mut self) {
        (self.libegl.eglMakeCurrent.unwrap())(
            self.egl_display,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        (self.libegl.eglDestroySurface.unwrap())(self.egl_display, self.surface);
        self.surface = std::ptr::null_mut();
    }
    
    unsafe fn update_surface(&mut self, window: *mut ndk_sys::ANativeWindow) {
        if !self.window.is_null() {
            ndk_sys::ANativeWindow_release(self.window);
        }
        self.window = window;
        if self.surface.is_null() == false {
            self.destroy_surface();
        }
        
        self.surface = (self.libegl.eglCreateWindowSurface.unwrap())(
            self.egl_display,
            self.egl_config,
            window as _,
            std::ptr::null_mut(),
        );
        
        assert!(!self.surface.is_null());
        
        let res = (self.libegl.eglMakeCurrent.unwrap())(
            self.egl_display,
            self.surface,
            self.surface,
            self.egl_context,
        );
        
        assert!(res != 0);
    }
}
impl CxOs {
    pub fn time_now(&self) -> f64 {
        let time_now = Instant::now(); //unsafe {mach_absolute_time()};
        (time_now.duration_since(self.time_start)).as_micros() as f64 / 1_000_000.0
    }
}
