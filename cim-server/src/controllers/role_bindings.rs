use std::convert::Infallible;

use async_stream::stream;
use axum::{
    body::Body,
    extract::{ws::Message, Path},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use bytes::{BufMut, BytesMut};
use futures_util::{SinkExt, StreamExt};
use http::StatusCode;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    role_binding::{Content, ListParams, RoleBinding},
    Event, Interface, List, WatchInterface, ID,
};

use crate::{
    auth::Auth,
    shutdown_signal,
    valid::{ListWatch, Valid},
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/role_bindings",
            get(list_role_binding).post(create_role_binding),
        )
        .route(
            "/role_bindings/:id",
            get(get_role_binding)
                .delete(delete_role_binding)
                .put(put_role_binding),
        )
        .with_state(state)
}

async fn create_role_binding(
    _auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .role_binding
        .create(&RoleBinding {
            id: id.to_string(),
            role_id: input.role_id,
            user_type: input.user_type,
            user_id: input.user_id,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_role_binding(
    _auth: Auth,
    app: AppState,
    list_watch: ListWatch<ListParams>,
) -> Result<Response> {
    match list_watch {
        ListWatch::List(filter) => {
            let mut list = List::default();
            app.store.role_binding.list(&filter, &mut list).await?;
            Ok(Json(list).into_response())
        }
        ListWatch::Watch(filter) => {
            let (tx, rx) = std::sync::mpsc::channel::<Event<RoleBinding>>();
            let _remove = app.store.role_binding.watch(
                0,
                move |event: Event<RoleBinding>| {
                    let result = event.get();
                    if let Some(ref v) = filter.id {
                        if result.id.ne(v) {
                            return;
                        }
                    }
                    tx.send(event).unwrap();
                },
            );
            let stream = stream! {
                while let Ok(item) = rx.recv() {
                    let mut buffer = BytesMut::default();
                    if  serde_json::to_writer((&mut buffer).writer(), &item).is_ok(){
                        yield buffer.freeze();
                    }
                }
            };
            Ok(Body::from_stream(
                stream.map(|v| -> std::result::Result<_, Infallible> { Ok(v) }),
            )
            .into_response())
        }
        ListWatch::Ws((ws, filter)) => {
            Ok(ws.on_upgrade(move |socket| async move {
                let (wtx, wrx) =
                    std::sync::mpsc::channel::<Event<RoleBinding>>();
                let _remove = app.store.role_binding.watch(
                    0,
                    move |event: Event<RoleBinding>| {
                        let result = event.get();
                        if let Some(ref v) = filter.id {
                            if result.id.ne(v) {
                                return;
                            }
                        }
                        wtx.send(event).unwrap();
                    },
                );
                let (mut sender, mut receiver) = socket.split();
                let mut send_task = tokio::spawn(async move {
                    while let Ok(item) = wrx.recv() {
                        let data = serde_json::to_vec(&item).unwrap();
                        if sender.send(Message::Binary(data)).await.is_err() {
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

async fn get_role_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<RoleBinding>> {
    let mut result = RoleBinding::default();
    result.id = id;
    app.store.role_binding.get(&mut result).await?;
    Ok(result.into())
}

async fn delete_role_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = RoleBinding::default();
    result.id = id;
    app.store.role_binding.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_role_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = RoleBinding::default();
    result.id = id;
    app.store.role_binding.get(&mut result).await?;

    result.role_id = content.role_id;
    result.user_type = content.user_type;
    result.user_id = content.user_id;
    app.store.role_binding.put(&result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
