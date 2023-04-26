use cim_core::Result;

use crate::{
    store::{roles, Store},
    AppState,
};

pub async fn put(
    app: &AppState,
    id: &str,
    content: &roles::Content,
) -> Result<()> {
    let found = app
        .store
        .role_exist(id, Some(content.account_id.clone()), true)
        .await?;
    if found {
        return app
            .store
            .update_role(
                id,
                Some(content.account_id.clone()),
                &roles::Opts {
                    name: Some(content.name.clone()),
                    desc: Some(content.desc.clone()),
                    unscoped: Some(true),
                },
            )
            .await;
    }
    app.store.create_role(Some(id.to_owned()), content).await?;
    Ok(())
}
