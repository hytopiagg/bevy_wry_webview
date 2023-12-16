use std::{marker::PhantomData, sync::Arc};

use bevy::prelude::{Component, Event, Plugin};
use serde::{Deserialize, Serialize};
use wry::{
    http::{Method, Request, Response},
    RequestAsyncResponder,
};

use crate::WebViewHandle;

pub(crate) struct WebViewIpcPlugin;

impl Plugin for WebViewIpcPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<FetchEvent>();
    }
}

#[derive(Component)]
pub struct IpcHandler<T, U>
where
    T: Serialize + Send + Sync,
    U: for<'a> Deserialize<'a> + Send + Sync,
{
    sender: crossbeam::Sender<Vec<u8>>,
    receiver: crossbeam::Receiver<Vec<u8>>,
    _phantom_data: PhantomData<(T, U)>,
}

#[derive(Component, Clone)]
pub struct TemporaryIpcStore {
    sender: crossbeam::Sender<Vec<u8>>,
    receiver: crossbeam::Receiver<Vec<u8>>,
}

#[derive(Event)]
pub struct FetchEvent(pub(crate) WebViewHandle);

impl TemporaryIpcStore {
    pub fn make_async_protocol(self) -> impl Fn(Request<Vec<u8>>, RequestAsyncResponder) + 'static {
        let func = move |req: Request<Vec<u8>>, res: RequestAsyncResponder| {
            if (req.uri() == "bevy://send" || req.uri() == "bevy://send/")
                && req.method() == Method::POST
            {
                let _ = self.sender.send(req.body().to_owned());
                res.respond(Response::builder().status(200).body(vec![]).unwrap());
            } else if (req.uri() == "bevy://fetch" || req.uri() == "bevy://fetch/")
                && req.method() == Method::GET
            {
                //let _ = fsender_cloned.send((WebViewHandle(Some(len)), x, data_tx.clone()));

                match self.receiver.recv() {
                    Ok(data) if !data.is_empty() => {
                        res.respond(Response::builder().status(200).body(data).unwrap())
                    }
                    _ => res.respond(Response::builder().status(404).body(vec![]).unwrap()),
                }
            } else {
                res.respond(Response::builder().status(404).body(vec![]).unwrap());
            }
        };

        return func;
    }
}

impl<T, U> IpcHandler<T, U>
where
    T: Serialize + Send + Sync,
    U: for<'a> Deserialize<'a> + Send + Sync,
{
    pub fn new() -> (Self, TemporaryIpcStore) {
        let (incoming_send, incoming_receive) = crossbeam::unbounded();
        let (outgoing_send, outgoing_receive) = crossbeam::unbounded();
        (
            IpcHandler {
                sender: outgoing_send,
                receiver: incoming_receive,
                _phantom_data: PhantomData,
            },
            TemporaryIpcStore {
                sender: incoming_send,
                receiver: outgoing_receive,
            },
        )
    }

    #[must_use]
    /// Generate message send event
    pub fn send(&self, handle: WebViewHandle, msg: T) -> FetchEvent {
        let _ = self.sender.send(rmp_serde::to_vec(&msg).unwrap());
        FetchEvent(handle)
    }
}

impl<T, U> Iterator for IpcHandler<T, U>
where
    T: Serialize + Send + Sync,
    U: for<'a> Deserialize<'a> + Send + Sync,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver
            .try_recv()
            .ok()
            .map(|x| rmp_serde::from_slice::<U>(&x).unwrap())
    }
}
