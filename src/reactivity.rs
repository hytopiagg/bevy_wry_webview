use crate::*;

pub struct WebViewReactivityPlugin;

impl Plugin for WebViewReactivityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::on_webview_resize,
                Self::on_webview_reposition,
                Self::on_webview_redirect,
                Self::on_window_resize,
            ),
        );
    }
}

impl WebViewReactivityPlugin {
    fn on_webview_resize(
        registry: NonSendMut<WebViewRegistry>,
        query: Query<(&WebViewHandle, &Node), (With<WebViewMarker>, Changed<Node>)>,
    ) {
        for (handle, size) in query.iter() {
            handle.map(|x| {
                registry
                    .get(x)
                    .map(|webview| webview.set_size((size.size().x as u32, size.size().y as u32)))
            });
        }
    }

    fn on_webview_reposition(
        registry: NonSendMut<WebViewRegistry>,
        query: Query<
            (&WebViewHandle, &GlobalTransform, &Node),
            (With<WebViewMarker>, Changed<GlobalTransform>),
        >,
    ) {
        for (handle, position, size) in query.iter() {
            let size = size.size();
            handle.map(|x| {
                registry.get(x).map(|webview| {
                    let final_position = (
                        (position.translation().x - size.x / 2.0) as i32,
                        (position.translation().y - size.y / 2.0) as i32,
                    );
                    webview.set_position(final_position)
                })
            });
        }
    }

    fn on_webview_redirect(
        registry: NonSendMut<WebViewRegistry>,
        query: Query<
            (&WebViewHandle, &WebViewLocation),
            (With<WebViewMarker>, Changed<WebViewLocation>),
        >,
    ) {
        for (handle, location) in query.iter() {
            handle.map(|x| {
                registry.get(x).map(|webview| match location {
                    WebViewLocation::Url(url) => webview.load_url(url),
                    WebViewLocation::Html(_html) => {
                        // TODO Implement HTML loading past builder
                    }
                })
            });
        }
    }

    fn on_window_resize(
        e: EventReader<WindowResized>,
        registry: NonSendMut<WebViewRegistry>,
        query: Query<(&WebViewHandle, &Node, &GlobalTransform), With<WebViewHandle>>,
    ) {
        if !e.is_empty() {
            for (handle, size, position) in &query {
                let size = size.size();
                let final_position = (
                    (position.translation().x - size.x / 2.0) as i32,
                    (position.translation().y - size.y / 2.0) as i32,
                );
                handle
                    .map(|x| registry.get(x))
                    .flatten()
                    .map(|webview| webview.set_position(final_position));
            }
        }
    }
}
