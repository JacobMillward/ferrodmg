use std::collections::HashMap;

use muda::{
    AboutMetadata, ContextMenu, IconMenuItem, Menu, MenuEvent, MenuId, PredefinedMenuItem, Submenu,
};
use winit::{
    dpi::LogicalSize,
    error::EventLoopError,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

#[cfg(target_os = "macos")]
use winit::platform::macos::{EventLoopBuilderExtMacOS, WindowExtMacOS};
#[cfg(target_os = "linux")]
use winit::platform::unix::WindowExtUnix;
#[cfg(target_os = "windows")]
use winit::platform::windows::EventLoopBuilderExtWindows;

type DispatchMap = HashMap<MenuId, Box<dyn Fn()>>;

pub struct AppBuilder {
    app_name: String,
    window_title: String,
    window_width: u32,
    window_height: u32,
}

impl AppBuilder {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            window_title: app_name.to_string(),
            window_width: 320,
            window_height: 240,
        }
    }

    pub fn with_window_title(mut self, window_title: &str) -> Self {
        self.window_title = window_title.to_string();
        self
    }

    pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
        self.window_width = width;
        self.window_height = height;
        self
    }

    pub fn build(self) -> Result<App, Box<dyn std::error::Error>> {
        App::new(
            self.app_name,
            self.window_title,
            self.window_width,
            self.window_height,
        )
    }
}

pub struct App {
    pub window: Window,
    menu_bar: Menu,
    context_menu: Submenu,
    menu_dispatch_map: DispatchMap,
    event_loop: Option<EventLoop<()>>,
}

impl App {
    /// Create new App with a menu bar.
    /// This function is platform-specific, and should only be called once.
    /// It should be called before any other menu-related functions.
    pub fn new(
        app_name: String,
        window_title: String,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut event_loop_builder = EventLoopBuilder::new();
        let mut menu_bar = Self::create_menu_bar(&mut event_loop_builder)?;
        let event_loop = event_loop_builder.build()?;

        let size = LogicalSize::new(width, height);
        let window = WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)?;

        let menu_dispatch_map = Self::create_menu_items(&mut menu_bar, &app_name)?;
        let context_menu = Submenu::with_items(
            "Context",
            true,
            &[&PredefinedMenuItem::close_window(Some("Exit"))],
        )?;

        let mut app = Self {
            window,
            menu_bar,
            context_menu,
            menu_dispatch_map,
            event_loop: Some(event_loop),
        };

        app.init()?;
        Ok(app)
    }

    fn create_menu_bar(event_loop_builder: &mut EventLoopBuilder<()>) -> Result<Menu, muda::Error> {
        let menu_bar = Menu::new();

        #[cfg(target_os = "windows")]
        {
            let menu_bar = menu_bar.clone();
            event_loop_builder.with_msg_hook(move |msg| {
                use windows_sys::Win32::UI::WindowsAndMessaging::{TranslateAcceleratorW, MSG};
                unsafe {
                    let msg = msg as *const MSG;
                    let translated = TranslateAcceleratorW((*msg).hwnd, menu_bar.haccel(), msg);
                    translated == 1
                }
            });
        }

        #[cfg(target_os = "macos")]
        event_loop_builder.with_default_menu(false);

        Ok(menu_bar)
    }

    /// Creates and adds menu items to the given menu.
    /// Returns a dispatch map for menu items for use in event handling.
    #[allow(clippy::type_complexity)]
    fn create_menu_items(
        menu: &mut Menu,
        app_name: &str,
    ) -> Result<HashMap<MenuId, Box<dyn Fn()>>, muda::Error> {
        let version = option_env!("CARGO_PKG_VERSION").map(|s| s.to_string());
        let authors = option_env!("CARGO_PKG_AUTHORS")
            .map(|s| s.split(':').map(|s| s.trim().to_string()).collect());

        let about = PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some(app_name.to_string()),
                version,
                authors,
                ..Default::default()
            }),
        );

        #[cfg(target_os = "macos")]
        {
            let app_m = Submenu::new("App", true);
            menu.append(&app_m);
            app_m.append_items(&[
                &about,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ]);
        }

        let path = "icon.png";
        let icon = load_icon(std::path::Path::new(path));
        let image_item = IconMenuItem::new("Image Custom 1", true, Some(icon), None);

        let file_m = Submenu::with_items(
            "&File",
            true,
            &[&image_item, &PredefinedMenuItem::close_window(Some("Exit"))],
        )?;
        let help_m = Submenu::with_items("&Help", true, &[&about])?;

        menu.append_items(&[&file_m, &help_m])?;

        // Create dispatch map
        let dispatch_map = HashMap::from_iter([(
            image_item.into_id(),
            Box::new(|| println!("Image Item Pressed")) as Box<dyn Fn()>,
        )]);

        Ok(dispatch_map)
    }

    /// Initialize the App
    /// This function sets up the menu bar for the given window.
    /// This function is platform-specific, and should only be called once.
    fn init(&mut self) -> Result<(), muda::Error> {
        #[cfg(target_os = "windows")]
        {
            use winit::raw_window_handle::*;
            if let RawWindowHandle::Win32(handle) = self.window.window_handle().unwrap().as_raw() {
                self.menu_bar.init_for_hwnd(handle.hwnd.get())?
            }
        }
        #[cfg(target_os = "macos")]
        {
            self.menu_bar.init_for_nsapp()?
        }
        #[cfg(target_os = "linux")]
        {
            let gtk_window = self.window.gtk_window();
            let vertical_gtk_box = self.window.default_vbox();
            self.menu_bar
                .init_for_gtk_window(&gtk_window, Some(&vertical_gtk_box))?
        }

        Ok(())
    }

    /// Show the context menu for the given window.
    pub fn show_context_menu(&self) {
        #[cfg(target_os = "windows")]
        {
            use winit::raw_window_handle::*;
            if let RawWindowHandle::Win32(handle) = self.window.window_handle().unwrap().as_raw() {
                self.context_menu
                    .show_context_menu_for_hwnd(handle.hwnd.get(), None);
            }
        }
        #[cfg(target_os = "macos")]
        {
            use winit::raw_window_handle::*;
            if let RawWindowHandle::AppKit(handle) = self.window.window_handle().unwrap().as_raw() {
                self.context_menu
                    .show_context_menu_for_nsview(handle.ns_view.as_ptr() as _, None);
            }
        }
        #[cfg(target_os = "linux")]
        {
            let gtk_window = self.window.gtk_window();
            let vertical_gtk_box = self.window.default_vbox();
            self.context_menu
                .show_context_menu_for_gtk_window(&gtk_window, vertical_gtk_box);
        }
    }

    /// A callback for handling window events.
    /// This function should be called from the event loop, for every event.
    fn handle_window_event(&self, event: &Event<()>, event_loop: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => event_loop.exit(),
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Right,
                        ..
                    },
                ..
            } => {
                self.show_context_menu();
            }

            _ => (),
        }

        // Handle menu events
        let menu_channel = MenuEvent::receiver();
        if let Ok(event) = menu_channel.try_recv() {
            if let Some(dispatch) = self.menu_dispatch_map.get(&event.id) {
                dispatch();
            }
        }
    }

    pub fn run<F>(&mut self, mut event_handler: F) -> Result<(), EventLoopError>
    where
        F: FnMut(Event<()>, &EventLoopWindowTarget<()>),
    {
        let event_loop = self.event_loop.take().expect("Event loop already consumed");

        event_loop.run(move |event, event_loop| {
            self.handle_window_event(&event, event_loop);
            event_handler(event, event_loop);
            self.window.request_redraw();
        })
    }
}

fn load_icon(path: &std::path::Path) -> muda::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    muda::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}
