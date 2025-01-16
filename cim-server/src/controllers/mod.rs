pub mod group_users;
pub mod groups;
pub mod oidc;
pub mod policies;
pub mod policy_bindings;
pub mod role_bindings;
pub mod roles;
pub mod users;

use std::convert::Infallible;

use async_stream::stream;
use axum::{
    extract::ws::Message,
    response::{sse::Event as SseEvent, IntoResponse, Response, Sse},
    Json,
};
use cim_slo::Result;
use cim_storage::{Event, List, WatchInterface};
use futures_util::{SinkExt, StreamExt};

use crate::{shutdown_signal, valid::ListWatch};

async fn list_watch<W, F>(
    store: W,
    list_params: ListWatch<W::L>,
    filter: F,
) -> Result<Response>
where
    W: WatchInterface + Send + 'static + Sync,
    F: Fn(&W::T, &W::L) -> bool + Sync + Send + 'static,
{
    match list_params {
        ListWatch::List(opts) => {
            let mut list = List::default();
            store.list(&opts, &mut list).await?;
            Ok(Json(list).into_response())
        }
        ListWatch::Watch(opts) => {
            let (tx, mut rx) =
                tokio::sync::mpsc::unbounded_channel::<Event<W::T>>();
            let remove = store.watch(0, move |event: Event<W::T>| {
                let result = event.get();
                if filter(result, &opts) {
                    return;
                }
                tx.send(event).unwrap();
            });
            let stream = stream! {
                remove.noop();
                loop{
                tokio::select! {
                    _ = shutdown_signal() => {
                            return;
                    },
                    result =rx.recv()=>{
                        if let Some(item) = result {
                            yield SseEvent::default().json_data(&item).unwrap();
                        }else{
                            return;
                        }
                    }
                }
                }
            };
            Ok(Sse::new(
                stream.map(|v| -> std::result::Result<_, Infallible> { Ok(v) }),
            )
            .into_response())
        }
        ListWatch::Ws((ws, opts)) => {
            Ok(ws.on_upgrade(move |socket| async move {
                let (wtx, wrx) = std::sync::mpsc::channel::<Event<W::T>>();
                let _remove = store.watch(0, move |event: Event<W::T>| {
                    let result = event.get();
                    if filter(result, &opts) {
                        return;
                    }
                    wtx.send(event).unwrap();
                });
                let (mut sender, mut receiver) = socket.split();
                let mut send_task = tokio::spawn(async move {
                    while let Ok(item) = wrx.recv() {
                        let data = serde_json::to_vec(&item).unwrap();
                        if sender
                            .send(Message::Binary(data.into()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });
                let mut recv_task = tokio::spawn(async move {
                    loop {
                        tokio::select! {
                        msg = receiver.next() => {
                        if let Some(msg) = msg {
                            if let Ok(msg) = msg {
                                if let Message::Close(_) = msg {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                        },
                        _ = shutdown_signal() => {
                            break;
                        }
                        }
                    }
                });
                tokio::select! {
                    _ = (&mut send_task) => {
                        recv_task.abort();
                    },
                    _ = (&mut recv_task) => {
                        send_task.abort();
                    }
                }
            }))
        }
    }
}
