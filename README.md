# `bevy_wry_webview`
## A small library for embedding a `wry` web-view into Bevy apps

### What Works

* Syncing the web-view's position and size to a `bevy_ui` element.
* Input, transparency
  * Free thanks to `wry`
* MacOS, Windows, Linux (w/ X11)

### To-Do List

* [ ] Decent Documentation
* [x] Allow Despawning of WebViews
* [ ] Better Modularity
  * [ ] Possibly encompass into a larger library with different web-view backends
* [ ] Off-screen Rendering
  * [Not possible on Windows](https://github.com/MicrosoftEdge/WebView2Feedback/issues/547)
    * May add for other platforms anyway behind a feature flag
* [ ] Better Platform Support
  * [ ] Mobile
  * [ ] Web support (ironic right)
  * [ ] Wayland
    * Seemingly impossible 
* [ ] Occlusion by other `bevy_ui` elements
  * This works using overlay windows, so occlusion would likely involve a lot of jank
* [ ] IPC Support
  * [ ] Basic, message based IPC 
    * [ ] Crossplatform Support
        * [x] Mac, Windows
        * [ ] Linux
    * [ ] Graceful Error-handling
  * [ ] A full IPC to allow JS to access the bevy `World`
    * [ ] Likely requires ["Fully dynamic term based queries and builder API"](https://github.com/bevyengine/bevy/pull/9774) to be merged
* General Refactoring
    * [ ] Split `IpcHandler` into read and write

### Credits/Thanks

[CrabNebula](https://crabnebula.dev) and the Wry team - Worked with us to get an embeddable web-view implemented, a surprisingly difficult but interesting task

[Nicopap](https://github.com/nicopap) - Served as the main source of Bevy knowledge through all of this, helped us understand the problem scope of the Bevy side a ton
