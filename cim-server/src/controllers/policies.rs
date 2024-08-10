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
    policy::{Content, ListParams, Policy},
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
        .put(
            &Policy {
                id: id.to_string(),
                account_id: Some(auth.user.account_id),
                desc: content.desc,
                version: content.version,
                statement: content.statement,
                ..Default::default()
            },
            0,
        )
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
        ListWatch::Ws((ws, filter)) => {
            Ok(ws.on_upgrade(move |socket| async move {
                let (wtx, wrx) = std::sync::mpsc::channel::<Event<Policy>>();
                let remove = app.store.policy.watch(
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
                        let result: EventData<Policy> = item.into();
                        if let Some(ref v) = filter.id {
                            if result.data.id.ne(v) {
                                continue;
                            }
                        }
                        if result.data.account_id != filter.account_id {
                            continue;
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

async fn get_policy(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Policy>> {
    let mut result = Policy::default();
    app.store.policy.get(&id, &mut result).await?;
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
    app.store.policy.get(&id, &mut result).await?;
    let mut opts = HashMap::new();
    if let Some(account_id) = &result.account_id {
        opts.insert("account_id".to_owned(), account_id.clone());
    }

    info.is_allow(&app.matcher, opts)?;
    app.store.policy.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_policy(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = Policy::default();
    app.store.policy.get(&id, &mut result).await?;
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
