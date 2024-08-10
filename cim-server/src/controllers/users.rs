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
    user::{Content, ListParams, User},
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
    info!("list query {:#?}", input);
    let id = next_id().map_err(errors::any)?;
    let account_id = match &input.account_id {
        Some(v) => v.parse().map_err(|err| errors::bad_request(&err))?,
        None => id,
    };

    app.store
        .user
        .put(
            &User {
                id: id.to_string(),
                account_id: account_id.to_string(),
                desc: input.desc,
                claim: input.claim,
                secret: None,
                password: Some(input.password),
                ..Default::default()
            },
            0,
        )
        .await?;
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
        ListWatch::Ws((ws, filter)) => {
            Ok(ws.on_upgrade(move |socket| async move {
                let (wtx, wrx) = std::sync::mpsc::channel::<Event<User>>();
                let remove = app.store.user.watch(
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
                        let result: EventData<User> = item.into();
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

async fn get_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<User>> {
    let mut user_result = User::default();
    app.store.user.get(&id, &mut user_result).await?;
    if !info.is_allow(
        &app.matcher,
        HashMap::from([(
            "account_id".to_owned(),
            user_result.account_id.clone(),
        )]),
    ) {
        return Err(errors::unauthorized());
    }
    user_result.secret = None;
    user_result.password = None;
    Ok(user_result.into())
}

async fn delete_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut user_result = User::default();
    app.store.user.get(&id, &mut user_result).await?;
    if !info.is_allow(
        &app.matcher,
        HashMap::from([(
            "account_id".to_owned(),
            user_result.account_id.clone(),
        )]),
    ) {
        return Err(errors::unauthorized());
    }

    app.store.user.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut user_result = User::default();
    app.store.user.get(&id, &mut user_result).await?;
    if !info.is_allow(
        &app.matcher,
        HashMap::from([(
            "account_id".to_owned(),
            user_result.account_id.clone(),
        )]),
    ) {
        return Err(errors::unauthorized());
    }

    user_result.desc = content.desc;
    user_result.claim = content.claim;
    user_result.password = Some(content.password);
    app.store.user.put(&user_result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
