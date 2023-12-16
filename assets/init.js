window.fetchMessage = function(messageId) {
    const req = new XMLHttpRequest();
    req.responseType = "arraybuffer";
    req.open("GET", "bevy://fetch/" + messageId, false);

    req.onload = function () {
        const blob = new Uint8Array(req.response);
        window.processMessage(blob);
    };

    req.send();
}
//window.processMessage = function(_bytes) {}

window.sendMessage = function(msg) {
    const req = new XMLHttpRequest();
    req.open("POST", "bevy://send", false);
    req.send(msgpack.encode(msg));
}
