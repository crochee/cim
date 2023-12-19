use crate::Result;

use crate::{
    store::{policies, Store},
    AppState,
};

pub async fn put(
    app: &AppState,
    id: &str,
    content: &policies::Content,
) -> Result<()> {
    let found = app
        .store
        .policy_exist(id, content.account_id.clone(), true)
        .await?;
    if found {
        return app
            .store
            .update_policy(
                id,
                content.account_id.clone(),
                &policies::Opts {
                    desc: Some(content.desc.clone()),
                    version: Some(content.version.clone()),
                    statement: Some(content.statement.clone()),
                    unscoped: Some(true),
                },
            )
            .await;
    }
    app.store
        .create_policy(Some(id.to_owned()), content)
        .await?;
    Ok(())
}
