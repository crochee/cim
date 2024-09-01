use std::{collections::HashMap, convert::Infallible};

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

use cim_slo::Result;
use cim_storage::{
    user::{Content, ListParams, User},
    Event, Interface, List, WatchInterface, ID,
};

use crate::{
    auth::{Auth, Info},
    services::user,
    shutdown_signal,
    valid::{ListWatch, Valid},
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/users", get(list_user).post(create_user))
        .route(
            "/users/:id",
            get(get_user).delete(delete_user).put(put_user),
        )
        .with_state(state)
}

async fn create_user(
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = user::create(app, input).await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_user(
    _auth: Auth,
    app: AppState,
    list_watch: ListWatch<ListParams>,
) -> Result<Response> {
    match list_watch {
        ListWatch::List(filter) => {
            let mut list = List::default();
            app.store.user.list(&filter, &mut list).await?;
            Ok(Json(list).into_response())
        }
        ListWatch::Watch(filter) => {
            let (tx, rx) = std::sync::mpsc::channel::<Event<User>>();
            let _remove = app.store.user.watch(0, move |event: Event<User>| {
                let result = event.get();
                if let Some(ref v) = filter.id {
                    if result.id.ne(v) {
                        return;
                    }
                }
                if let Some(ref v) = filter.account_id {
                    if result.account_id.ne(v) {
                        return;
                    }
                }
                tx.send(event).unwrap();
            });
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
                let (wtx, wrx) = std::sync::mpsc::channel::<Event<User>>();
                let _remove =
                    app.store.user.watch(0, move |event: Event<User>| {
                        let result = event.get();
                        if let Some(ref v) = filter.id {
                            if result.id.ne(v) {
                                return;
                            }
                        }
                        if let Some(ref v) = filter.account_id {
                            if result.account_id.ne(v) {
                                return;
                            }
                        }

                        wtx.send(event).unwrap();
                    });
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

async fn get_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<User>> {
    let mut result = User {
        id: id.clone(),
        ..Default::default()
    };
    app.store.user.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    result.secret = None;
    result.password = None;
    Ok(result.into())
}

async fn delete_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = User {
        id: id.clone(),
        ..Default::default()
    };
    app.store.user.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    app.store.user.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut user = User {
        id: id.clone(),
        ..Default::default()
    };
    app.store.user.get(&mut user).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), user.account_id.clone())]),
    )?;
    user.desc = content.desc;
    user.claim = content.claim;
    user.password = Some(content.password);
    app.store.user.put(&user, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
