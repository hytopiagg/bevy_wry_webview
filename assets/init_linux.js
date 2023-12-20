window.fetchMessage = function(item) {
    const obj = JSON.parse(item);
    window.processMessage(obj)
}

window.processMessage = function(item) {}

window.sendMessage = function(msg) {
    window.ipc.postMessage(JSON.stringify(msg))
}
