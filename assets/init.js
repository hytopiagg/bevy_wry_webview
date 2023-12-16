window.fetchMessage = function() {
    const req = new XMLHttpRequest();
    req.responseType = "arraybuffer";
    req.open("GET", "bevy://fetch", false);

    req.onload = function () {
        const blob = new Uint8Array(req.response);
        window.processMessage(msgpack.decode(blob));
    };

    req.send();
}

window.processMessage = function(item) {}

window.sendMessage = function(msg) {
    const req = new XMLHttpRequest();
    req.open("POST", "bevy://send", false);
    req.send(msgpack.encode(msg));
}
