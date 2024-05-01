use cim_slo::Result;
use cim_storage::roles;

pub async fn put_role<R>(
    role_store: &R,
    id: &str,
    content: &roles::Content,
) -> Result<()>
where
    R: roles::RoleStore,
{
    let found = role_store
        .role_exist(id, Some(content.account_id.clone()), true)
        .await?;
    if found {
        return role_store
            .update_role(
                id,
                Some(content.account_id.clone()),
                &roles::UpdateOpts {
                    name: Some(content.name.clone()),
                    desc: Some(content.desc.clone()),
                    unscoped: Some(true),
                },
            )
            .await;
    }
    role_store.create_role(Some(id.to_owned()), content).await?;
    Ok(())
}
