window.fetchMessage = function() {
    const req = new XMLHttpRequest();
    req.responseType = "arraybuffer";
    req.open("GET", isWindows ? "http://bevy.send" : "bevy://fetch", false);

    req.onload = function () {
        const blob = new Uint8Array(req.response);
        window.processMessage(msgpack.decode(blob));
    };

    req.send();
}

window.processMessage = function(item) {}

window.sendMessage = async function(msg) {
    const url = isWindows ? "http://bevy.send" : "bevy://send";
    try {
        await fetch(url, {
            method: 'POST',
            body: msgpack.encode(msg)
        });
    } catch (error) {
        console.error("Send error: " + error.message);
    }
}
