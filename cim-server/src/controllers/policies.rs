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

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    policy::{Content, ListParams, Policy},
    Event, Interface, List, WatchInterface, ID,
};

use crate::{
    auth::{Auth, Info},
    shutdown_signal,
    valid::{ListWatch, Valid},
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/policies", get(list_policy).post(create_policy))
        .route(
            "/policies/:id",
            get(get_policy).delete(delete_policy).put(put_policy),
        )
        .with_state(state)
}

async fn create_policy(
    auth: Auth,
    app: AppState,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .policy
        .create(&Policy {
            id: id.to_string(),
            account_id: Some(auth.user.account_id),
            desc: content.desc,
            version: content.version,
            statement: content.statement,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_policy(
    _auth: Auth,
    app: AppState,
    list_watch: ListWatch<ListParams>,
) -> Result<Response> {
    match list_watch {
        ListWatch::List(filter) => {
            let mut list = List::default();
            app.store.policy.list(&filter, &mut list).await?;
            Ok(Json(list).into_response())
        }
        ListWatch::Watch(filter) => {
            let (tx, rx) = std::sync::mpsc::channel::<Event<Policy>>();
            let _remove =
                app.store.policy.watch(0, move |event: Event<Policy>| {
                    let result = event.get();
                    if let Some(ref v) = filter.id {
                        if result.id.ne(v) {
                            return;
                        }
                    }
                    if result.account_id != filter.account_id {
                        return;
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
                let (wtx, wrx) = std::sync::mpsc::channel::<Event<Policy>>();
                let _remove =
                    app.store.policy.watch(0, move |event: Event<Policy>| {
                        let result = event.get();
                        if let Some(ref v) = filter.id {
                            if result.id.ne(v) {
                                return;
                            }
                        }
                        if result.account_id != filter.account_id {
                            return;
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

async fn get_policy(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Policy>> {
    let mut result = Policy::default();
    result.id = id;
    app.store.policy.get(&mut result).await?;
    let mut opts = HashMap::new();
    if let Some(account_id) = &result.account_id {
        opts.insert("account_id".to_owned(), account_id.clone());
    }
    info.is_allow(&app.matcher, opts)?;
    Ok(result.into())
}

async fn delete_policy(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = Policy::default();
    result.id = id;
    app.store.policy.get(&mut result).await?;
    let mut opts = HashMap::new();
    if let Some(account_id) = &result.account_id {
        opts.insert("account_id".to_owned(), account_id.clone());
    }

    info.is_allow(&app.matcher, opts)?;
    app.store.policy.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_policy(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = Policy::default();
    result.id = id;
    app.store.policy.get(&mut result).await?;
    let mut opts = HashMap::new();
    if let Some(account_id) = &result.account_id {
        opts.insert("account_id".to_owned(), account_id.clone());
    }
    info.is_allow(&app.matcher, opts)?;

    result.desc = content.desc;
    result.version = content.version;
    result.statement = content.statement;
    app.store.policy.put(&result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
