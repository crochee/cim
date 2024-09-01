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
    group_user::{Content, GroupUser, ListParams},
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
        .route("/group_users", get(list_group_user).post(create_group_user))
        .route(
            "/group_users/:id",
            get(get_group_user)
                .delete(delete_group_user)
                .put(put_group_user),
        )
        .with_state(state)
}

async fn create_group_user(
    _auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .group_user
        .create(&GroupUser {
            id: id.to_string(),
            group_id: input.group_id,
            user_id: input.user_id,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_group_user(
    _auth: Auth,
    app: AppState,
    list_watch: ListWatch<ListParams>,
) -> Result<Response> {
    match list_watch {
        ListWatch::List(filter) => {
            let mut list = List::default();
            app.store.group_user.list(&filter, &mut list).await?;
            Ok(Json(list).into_response())
        }
        ListWatch::Watch(filter) => {
            let (tx, rx) = std::sync::mpsc::channel::<Event<GroupUser>>();
            let _remove = app.store.group_user.watch(
                0,
                move |event: Event<GroupUser>| {
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
                let (wtx, wrx) = std::sync::mpsc::channel::<Event<GroupUser>>();
                let _remove = app.store.group_user.watch(
                    0,
                    move |event: Event<GroupUser>| {
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

async fn get_group_user(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<GroupUser>> {
    let mut result = GroupUser {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group_user.get(&mut result).await?;
    Ok(result.into())
}

async fn delete_group_user(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let result = GroupUser {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group_user.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_group_user(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = GroupUser {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group_user.get(&mut result).await?;

    result.user_id = content.user_id;
    result.group_id = content.group_id;
    app.store.group_user.put(&result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
