use std::collections::HashMap;

use axum::{
    extract::{ws::Message, Path},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use http::StatusCode;
use tracing::info;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    role::{Content, ListParams, Role},
    Event, EventData, Interface, List, ID,
};

use crate::{
    auth::{Auth, Info},
    shutdown_signal,
    valid::{ListWatch, Valid},
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/roles", get(list_role).post(create_role))
        .route(
            "/roles/:id",
            get(get_role).delete(delete_role).put(put_role),
        )
        .with_state(state)
}

async fn create_role(
    auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    info!("list query {:#?}", input);
    let id = next_id().map_err(errors::any)?;
    app.store
        .role
        .put(
            &Role {
                id: id.to_string(),
                account_id: auth.user.account_id,
                name: input.name,
                desc: input.desc,
                ..Default::default()
            },
            0,
        )
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_role(
    _auth: Auth,
    app: AppState,
    list_watch: ListWatch<ListParams>,
) -> Result<Response> {
    match list_watch {
        ListWatch::List(filter) => {
            let mut list = List::default();
            app.store.role.list(&filter, &mut list).await?;
            Ok(Json(list).into_response())
        }
        ListWatch::Ws((ws, filter)) => {
            Ok(ws.on_upgrade(move |socket| async move {
                let (wtx, wrx) = std::sync::mpsc::channel::<Event<Role>>();
                let remove = app.store.role.watch(
                    move |event| {
                        wtx.send(event).unwrap();
                    },
                    move || {
                        info!("remove out 2");
                    },
                );
                let (mut sender, mut receiver) = socket.split();
                let mut send_task = tokio::spawn(async move {
                    while let Ok(item) = wrx.recv() {
                        let result: EventData<Role> = item.into();
                        if let Some(ref v) = filter.id {
                            if result.data.id.ne(v) {
                                continue;
                            }
                        }
                        if let Some(ref v) = filter.account_id {
                            if result.data.account_id.ne(v) {
                                continue;
                            }
                        }

                        let data = serde_json::to_vec(&result).unwrap();
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
                        remove();
                        recv_task.abort();
                    },
                    _ = (&mut recv_task) => {
                        remove();
                        send_task.abort();
                    }
                }
            }))
        }
    }
}

async fn get_role(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Role>> {
    let mut result = Role::default();
    app.store.role.get(&id, &mut result).await?;
    if !info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    ) {
        return Err(errors::unauthorized());
    }
    Ok(result.into())
}

async fn delete_role(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = Role::default();
    app.store.role.get(&id, &mut result).await?;
    if !info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    ) {
        return Err(errors::unauthorized());
    }
    app.store.role.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_role(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = Role::default();
    app.store.role.get(&id, &mut result).await?;
    if !info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    ) {
        return Err(errors::unauthorized());
    }
    result.name = content.name;
    result.desc = content.desc;
    app.store.role.put(&result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
