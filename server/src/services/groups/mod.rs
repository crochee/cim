use slo::Result;
use storage::groups;

pub async fn put_group<G: groups::GroupStore>(
    group_store: &G,
    id: &str,
    content: &groups::Content,
) -> Result<()> {
    let found = group_store
        .group_exist(id, Some(content.account_id.clone()), true)
        .await?;
    if found {
        return group_store
            .update_group(
                id,
                Some(content.account_id.clone()),
                &groups::UpdateOpts {
                    name: Some(content.name.clone()),
                    desc: Some(content.desc.clone()),
                    unscoped: Some(true),
                },
            )
            .await;
    }
    group_store
        .create_group(Some(id.to_owned()), content)
        .await?;
    Ok(())
}
