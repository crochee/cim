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
    policy_binding::{Content, ListParams, PolicyBinding},
    Event, EventData, Interface, List, ID,
};
use tracing::info;

use crate::{
    shutdown_signal,
    valid::{Header, ListWatch, Valid},
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/policy_bindings",
            get(list_policy_binding).post(create_policy_binding),
        )
        .route(
            "/policy_bindings/:id",
            get(get_policy_binding)
                .delete(delete_policy_binding)
                .put(put_policy_binding),
        )
        .with_state(state)
}

async fn create_policy_binding(
    header: Header,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    if !header.is_allow(&app.matcher, HashMap::from([])) {
        return Err(errors::unauthorized());
    }
    let id = next_id().map_err(errors::any)?;
    app.store
        .policy_binding
        .put(
            &PolicyBinding {
                id: id.to_string(),
                policy_id: input.policy_id,
                bindings_type: input.bindings_type,
                bindings_id: input.bindings_id,
                ..Default::default()
            },
            0,
        )
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_policy_binding(
    header: Header,
    app: AppState,
    list_watch: ListWatch<ListParams>,
) -> Result<Response> {
    match list_watch {
        ListWatch::List(filter) => {
            if !header.is_allow(&app.matcher, HashMap::from([])) {
                return Err(errors::unauthorized());
            }
            let mut list = List::default();
            app.store.policy_binding.list(&filter, &mut list).await?;
            Ok(Json(list).into_response())
        }
        ListWatch::Ws((ws, filter)) => {
            if !header.is_allow(&app.matcher, HashMap::from([])) {
                return Err(errors::unauthorized());
            }

            Ok(ws.on_upgrade(move |socket| async move {
                let (wtx, wrx) =
                    std::sync::mpsc::channel::<Event<PolicyBinding>>();
                let remove = app.store.policy_binding.watch(
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
                        let result: EventData<PolicyBinding> = item.into();
                        if let Some(ref v) = filter.id {
                            if result.data.id.ne(v) {
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

async fn get_policy_binding(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<PolicyBinding>> {
    let mut result = PolicyBinding::default();
    app.store.policy_binding.get(&id, &mut result).await?;
    if !header.is_allow(&app.matcher, HashMap::from([])) {
        return Err(errors::unauthorized());
    }
    Ok(result.into())
}

async fn delete_policy_binding(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = PolicyBinding::default();
    app.store.policy_binding.get(&id, &mut result).await?;
    if !header.is_allow(&app.matcher, HashMap::from([])) {
        return Err(errors::unauthorized());
    }
    app.store.policy_binding.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_policy_binding(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = PolicyBinding::default();
    app.store.policy_binding.get(&id, &mut result).await?;
    if !header.is_allow(&app.matcher, HashMap::from([])) {
        return Err(errors::unauthorized());
    }
    result.policy_id = content.policy_id;
    result.bindings_type = content.bindings_type;
    result.bindings_id = content.bindings_id;
    app.store.policy_binding.put(&result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
