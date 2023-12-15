use std::collections::HashMap;

use crossbeam::{channel, Receiver, Sender};

use bevy::{prelude::*, utils::Uuid};

use crate::{WebViewHandle, WebViewRegistry};

#[derive(Event)]
pub struct SendBytes(pub WebViewHandle, pub Vec<u8>);

#[derive(Event)]
pub struct WryMessage(pub WebViewHandle, pub Vec<u8>);

pub(super) struct WebViewIpcPlugin;

#[derive(Resource, Deref, DerefMut)]
pub(super) struct WryMessageSender(Sender<(WebViewHandle, Vec<u8>)>);

#[derive(Resource, Deref, DerefMut)]
struct WryMessageReceiver(Receiver<(WebViewHandle, Vec<u8>)>);

#[derive(Resource, Deref, DerefMut)]
pub(super) struct WryFetchSender(Sender<(WebViewHandle, u128, Sender<Vec<u8>>)>);

#[derive(Resource, Deref, DerefMut)]
struct WryFetchReceiver(Receiver<(WebViewHandle, u128, Sender<Vec<u8>>)>);

#[derive(Resource, Deref, DerefMut)]
struct WryMessageRegistry(HashMap<(WebViewHandle, u128), Vec<u8>>);

impl Plugin for WebViewIpcPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = channel::unbounded::<(WebViewHandle, Vec<u8>)>();
        let (fsender, freceiver) =
            channel::unbounded::<(WebViewHandle, u128, Sender<Vec<u8>>)>();
        app.add_event::<SendBytes>()
            .insert_resource(WryMessageSender(sender))
            .insert_resource(WryMessageReceiver(receiver))
            .insert_resource(WryFetchSender(fsender))
            .insert_resource(WryFetchReceiver(freceiver))
            .insert_resource(WryMessageRegistry(HashMap::new()))
            .add_event::<WryMessage>()
            .add_systems(
                Update,
                (
                    Self::receive_bytes_system,
                    Self::respond_to_message_fetch,
                    Self::send_bytes_system,
                ),
            );
    }
}

impl WebViewIpcPlugin {
    pub(super) const IPC_INIT_SCRIPT: &'static str = include_str!("../assets/init.js");

    fn send_bytes_system(
        registry: NonSend<WebViewRegistry>,
        mut reader: EventReader<SendBytes>,
        mut mregistry: ResMut<WryMessageRegistry>,
    ) {
        for SendBytes(handle, data) in reader.iter() {
            if let Some(webview) = handle.map(|x| registry.get(x)).flatten() {
                let queue_id = Uuid::new_v4().to_u128_le();
                mregistry.insert((handle.to_owned(), queue_id), data.to_owned());
                webview
                    .evaluate_script(&format!("window.fetchMessage('{}')", queue_id))
                    .unwrap();
            }
        }
    }

    fn respond_to_message_fetch(
        receiver: Res<WryFetchReceiver>,
        mut registry: ResMut<WryMessageRegistry>,
    ) {
        let cloned_registry = registry.clone();
        for (ref handle, uuid, responder) in receiver.try_iter() {
            match (&cloned_registry).get(&(handle.clone(), uuid)) {
                Some(data) => {
                    responder.send(data.clone()).unwrap();
                    registry.remove(&(handle.clone(), uuid));
                }
                None => responder.send(vec![]).unwrap(),
            }
        }
    }

    fn receive_bytes_system(
        receiver: Res<WryMessageReceiver>,
        mut writer: EventWriter<WryMessage>,
    ) {
        for i in receiver.try_iter() {
            writer.send(WryMessage(i.0, i.1));
        }
    }
}
