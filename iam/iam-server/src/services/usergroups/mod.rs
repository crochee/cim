use cim_core::Result;

use crate::{
    store::{usergroups, Store},
    AppState,
};

pub async fn put(
    app: &AppState,
    id: &str,
    content: &usergroups::Content,
) -> Result<()> {
    let found = app
        .store
        .user_group_exist(id, Some(content.account_id.clone()), true)
        .await?;
    if found {
        return app
            .store
            .update_user_group(
                id,
                Some(content.account_id.clone()),
                &usergroups::Opts {
                    name: Some(content.name.clone()),
                    desc: Some(content.desc.clone()),
                    unscoped: Some(true),
                },
            )
            .await;
    }
    app.store
        .create_user_group(Some(id.to_owned()), content)
        .await?;
    Ok(())
}
