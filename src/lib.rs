use ipc::{WebViewIpcPlugin, WryFetchSender, WryMessageSender};
use reactivity::WebViewReactivityPlugin;
use wry::{
    http::{Method, Response},
    WebView, WebViewBuilder,
};

use bevy::{
    prelude::*,
    window::{RawHandleWrapper, WindowResized},
};
use raw_window_handle::{ActiveHandle, WindowHandle};

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

#[derive(Component, Clone, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct WebViewHandle(Option<usize>);

#[derive(Bundle)]
pub struct UiWebViewBundle {
    pub node_bundle: NodeBundle,
    pub location: WebViewLocation,
    pub handle: WebViewHandle,
    pub marker: WebViewMarker,
    // TODO Add IPC handler
}

impl Default for UiWebViewBundle {
    fn default() -> Self {
        UiWebViewBundle {
            node_bundle: default(),
            location: WebViewLocation::Html("".to_owned()),
            handle: WebViewHandle(None),
            marker: WebViewMarker,
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
        app.insert_non_send_resource(WebViewRegistry { webviews: vec![] })
            .add_plugins((WebViewReactivityPlugin, WebViewIpcPlugin))
            .add_systems(Update, Self::on_webview_spawn);
    }
}

impl WebViewPlugin {
    fn on_webview_spawn(
        mut registry: NonSendMut<WebViewRegistry>,
        window_handle: Query<&RawHandleWrapper>,
        mut query: Query<
            (
                &mut WebViewHandle,
                &WebViewLocation,
                &Node,
                &GlobalTransform,
            ),
            With<WebViewMarker>,
        >,
        sender: Res<WryMessageSender>,
        fsender: Res<WryFetchSender>,
    ) {
        if let Ok(window_handle) = window_handle.get_single().map(|x| x.window_handle) {
            for (mut handle, location, size, position) in
                query.iter_mut().filter(|(x, _, _, _)| x.is_none())
            // && v.is_visible())
            {
                let size = size.size();
                let final_position = (
                    (position.translation().x - size.x / 2.0) as i32,
                    (position.translation().y - size.y / 2.0) as i32,
                );

                *handle = WebViewHandle(Some(registry.len()));

                let sender_cloned = sender.clone();
                let fsender_cloned = fsender.clone();
                let len = registry.len();
                let (data_tx, data_rx) = crossbeam::channel::unbounded();
                let (protocol_tx, protocol_rx) = crossbeam::channel::unbounded();

                let borrowed_handle =
                    unsafe { &WindowHandle::borrow_raw(window_handle, ActiveHandle::new()) };
                let webview = WebViewBuilder::new_as_child(&borrowed_handle)
                    .with_position(final_position)
                    .with_transparent(true)
                    .with_size((size.x as u32, size.y as u32))
                    .with_initialization_script(WebViewIpcPlugin::IPC_INIT_SCRIPT)
                    .with_asynchronous_custom_protocol(
                        "bevy".to_owned(),
                        move |req, res| {
                            let _ = protocol_tx.send((req, res)).unwrap();
                        }, //WebViewIpcPlugin::handle_ipc,
                    );

                // async custom protocol thread
                std::thread::spawn(move || {
                    while let Ok((req, res)) = protocol_rx.recv() {
                        if req.uri() == "bevy://send"
                            || req.uri() == "bevy://send/" && req.method() == Method::POST
                        {
                            let _ = sender_cloned
                                .send((WebViewHandle(Some(len)), req.body().to_owned()));
                            res.respond(Response::builder().status(200).body(vec![]).unwrap());
                        } else if req.uri().to_string().starts_with("bevy://fetch/")
                            && req.method() == Method::GET
                        {
                            match req.uri().to_string().split_at(13).1.parse::<u128>() {
                                Ok(x) => {
                                    let _ = fsender_cloned.send((
                                        WebViewHandle(Some(len)),
                                        x,
                                        data_tx.clone(),
                                    ));

                                    match data_rx.recv() {
                                        Ok(data) if !data.is_empty() => res.respond(
                                            Response::builder().status(200).body(data).unwrap(),
                                        ),
                                        _ => res.respond(
                                            Response::builder().status(404).body(vec![]).unwrap(),
                                        ),
                                    }
                                }
                                Err(_) => {
                                    res.respond(
                                        Response::builder().status(409).body(vec![]).unwrap(),
                                    );
                                }
                            }
                        } else {
                            res.respond(Response::builder().status(404).body(vec![]).unwrap());
                        }
                    }
                });

                let webview = match location {
                    WebViewLocation::Url(url) => webview.with_url(url),
                    WebViewLocation::Html(html) => webview.with_html(html),
                }
                .unwrap()
                .build()
                .unwrap();

                registry.push(webview);
            }
        }
    }
}
