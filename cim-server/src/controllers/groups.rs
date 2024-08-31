use std::collections::HashMap;

use axum::{
    extract::{ws::Message, Path},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

use futures_util::{SinkExt, StreamExt};
use http::StatusCode;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    group::{Content, Group, ListParams},
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
        .route("/groups", get(list_group).post(create_group))
        .route(
            "/groups/:id",
            get(get_group).delete(delete_group).put(put_group),
        )
        .with_state(state)
}

async fn create_group(
    auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .group
        .create(&Group {
            id: id.to_string(),
            account_id: auth.user.account_id,
            name: input.name,
            desc: input.desc,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_group(
    _auth: Auth,
    app: AppState,
    list_watch: ListWatch<ListParams>,
) -> Result<Response> {
    match list_watch {
        ListWatch::List(filter) => {
            let mut list = List::default();
            app.store.group.list(&filter, &mut list).await?;
            Ok(Json(list).into_response())
        }
        ListWatch::Ws((ws, filter)) => {
            Ok(ws.on_upgrade(move |socket| async move {
                let (wtx, wrx) = std::sync::mpsc::channel::<Event<Group>>();
                let _remove =
                    app.store.group.watch(0, move |event: Event<Group>| {
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

async fn get_group(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Group>> {
    let mut result = Group::default();
    result.id = id;
    app.store.group.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    Ok(result.into())
}

async fn delete_group(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = Group::default();
    result.id = id;
    app.store.group.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    app.store.group.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_group(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = Group::default();
    result.id = id;
    app.store.group.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;

    result.name = content.name;
    result.desc = content.desc;
    app.store.group.put(&result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
