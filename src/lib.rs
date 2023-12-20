use ipc::{new_ipc_channel, FetchEvent, IpcQueue, IpcSender, TemporaryIpcStore, WebViewIpcPlugin};
use reactivity::WebViewReactivityPlugin;
use wry::{WebView, WebViewBuilder};

use bevy::{
    prelude::*,
    window::{RawHandleWrapper, WindowResized},
};
use raw_window_handle::{ActiveHandle, WindowHandle};
use serde::{Deserialize, Serialize};

pub mod ipc;
mod reactivity;

pub struct WebViewPlugin;

#[derive(Component, Debug)]
pub enum WebViewLocation {
    Url(String),
    Html(String),
}

#[derive(Component)]
pub struct WebViewMarker;

#[derive(DerefMut, Deref)]
pub struct WebViewRegistry {
    webviews: Vec<WebView>,
}

#[derive(Component, Clone, Copy, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct WebViewHandle(Option<usize>);

#[derive(Bundle)]
pub struct UiWebViewBundle<T, U>
where
    T: Serialize + Send + Sync + 'static,
    U: for<'a> Deserialize<'a> + Send + Sync + 'static,
{
    pub node_bundle: NodeBundle,
    pub location: WebViewLocation,
    pub handle: WebViewHandle,
    pub marker: WebViewMarker,
    pub ipc_sender: IpcSender<T>,
    pub ipc_queue: IpcQueue<U>,
    pub temporary_ipc_store: TemporaryIpcStore,
}

impl<T, U> Default for UiWebViewBundle<T, U>
where
    T: Serialize + Send + Sync + 'static,
    U: for<'a> Deserialize<'a> + Send + Sync + 'static,
{
    fn default() -> Self {
        let (ipc_sender, ipc_queue, temporary_ipc_store) = new_ipc_channel::<T, U>();
        Self {
            node_bundle: default(),
            location: WebViewLocation::Html("".to_owned()),
            handle: WebViewHandle(None),
            marker: WebViewMarker,
            ipc_sender,
            ipc_queue,
            temporary_ipc_store,
        }
    }
}

/**
 * A simple trait to emulate a custom command for despawning `UiWebViewBundle`s
 */
pub trait WebViewDespawning {
    /**
     * Despawns `UiWebViewBundle`s and cleans up the associated `wry` `WebView`
     */
    fn despawn_webview(&mut self, entity: Entity);
}

impl WebViewDespawning for Commands<'_, '_> {
    fn despawn_webview(&mut self, entity: Entity) {
        self.add(move |world: &mut World| {
            let registry = world
                .get_non_send_resource::<WebViewRegistry>()
                .unwrap_or_else(|| {
                    panic!("WebView Registry not found; have you loaded `WebViewPlugin`")
                });
            let handle = world.entity(entity).get::<WebViewHandle>().unwrap();
            // TODO close it here -- Waiting on Tauri/Wry folks
            handle.map(|x| registry.get(x)).map(drop);
            world.despawn(entity);
        })
    }
}

impl Plugin for WebViewPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        {
            use gtk::prelude::DisplayExtManual;

            gtk::init().unwrap();
            if gtk::gdk::Display::default().unwrap().backend().is_wayland() {
                panic!("This example doesn't support wayland!");
            }

            // we need to ignore this error here otherwise it will be catched by winit and will be
            // make the example crash
            winit::platform::x11::register_xlib_error_hook(Box::new(|_display, error| {
                let error = error as *mut x11_dl::xlib::XErrorEvent;
                (unsafe { (*error).error_code }) == 170
            }));
        }

        #[cfg(not(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        app.insert_non_send_resource(WebViewRegistry { webviews: vec![] })
            .add_plugins((WebViewReactivityPlugin, WebViewIpcPlugin))
            .add_systems(Update, (Self::on_webview_spawn, Self::handle_fetch));

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        app.insert_non_send_resource(WebViewRegistry { webviews: vec![] })
            .add_plugins((WebViewReactivityPlugin, WebViewIpcPlugin))
            .add_systems(
                Update,
                (
                    Self::on_webview_spawn,
                    Self::handle_fetch,
                    Self::forward_gtk,
                ),
            );
    }
}

impl WebViewPlugin {
    fn on_webview_spawn(
        mut commands: Commands,
        mut registry: NonSendMut<WebViewRegistry>,
        window_handle: Query<&RawHandleWrapper>,
        mut query: Query<
            (
                Entity,
                &mut WebViewHandle,
                &WebViewLocation,
                &Node,
                &GlobalTransform,
                &TemporaryIpcStore,
            ),
            With<WebViewMarker>,
        >,
    ) {
        if let Ok(window_handle) = window_handle.get_single().map(|x| x.window_handle) {
            for (entity, mut handle, location, size, position, tis) in
                query.iter_mut().filter(|(_, x, _, _, _, _)| x.is_none())
            // && v.is_visible())
            {
                let size = size.size();
                let final_position = (
                    (position.translation().x - size.x / 2.0) as i32,
                    (position.translation().y - size.y / 2.0) as i32,
                );

                *handle = WebViewHandle(Some(registry.len()));

                let borrowed_handle =
                    unsafe { &WindowHandle::borrow_raw(window_handle, ActiveHandle::new()) };

                #[cfg(not(any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd",
                )))]
                let webview = {
                    let func = tis.clone().make_async_protocol();
                    WebViewBuilder::new_as_child(&borrowed_handle)
                        .with_position(final_position)
                        .with_transparent(true)
                        .with_size((size.x as u32, size.y as u32))
                        .with_initialization_script(include_str!("../assets/msgpack.min.js"))
                        .with_initialization_script(include_str!("../assets/init.js"))
                        .with_asynchronous_custom_protocol(
                            "bevy".to_owned(),
                            func, //WebViewIpcPlugin::handle_ipc,
                        )
                };

                #[cfg(any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd",
                ))]
                let webview = {
                    let func = tis.clone().make_ipc_handler();
                    WebViewBuilder::new(&borrowed_handle)
                        .with_position(final_position)
                        .with_transparent(true)
                        .with_size((size.x as u32, size.y as u32))
                        .with_initialization_script(include_str!("../assets/init_linux.js"))
                        .with_ipc_handler(func)
                };

                let webview = match location {
                    WebViewLocation::Url(url) => webview.with_url(url),
                    WebViewLocation::Html(html) => webview.with_html(html),
                }
                .unwrap()
                .build()
                .unwrap();

                if let Some(mut x) = commands.get_entity(entity) {
                    x.remove::<TemporaryIpcStore>();
                }

                registry.push(webview);
            }
        }
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    )))]
    fn handle_fetch(registry: NonSendMut<WebViewRegistry>, mut reader: EventReader<FetchEvent>) {
        for &i in reader
            .read()
            .filter_map(|FetchEvent(WebViewHandle(i))| i.as_ref())
        {
            if let Some(wv) = registry.get(i) {
                let _ = wv.evaluate_script("window.fetchMessage()");
            }
        }
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    fn handle_fetch(registry: NonSendMut<WebViewRegistry>, mut reader: EventReader<FetchEvent>) {
        for (&i, j) in reader
            .read()
            .filter_map(|FetchEvent(WebViewHandle(i), j)| (i.as_ref(), j.clone()))
        {
            if let Some(wv) = registry.get(i) {
                let _ = wv.evaluate_script(format!(r#"window.fetchMessage(`{}`)"#, j));
            }
        }
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    fn forward_gtk(_: &mut World) {
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }
}
