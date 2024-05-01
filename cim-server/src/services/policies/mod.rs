use cim_slo::Result;
use cim_storage::policies;

pub async fn put_policy<P: policies::PolicyStore>(
    policy_store: &P,
    id: &str,
    content: &policies::Content,
) -> Result<()> {
    let found = policy_store
        .policy_exist(id, content.account_id.clone(), true)
        .await?;
    if found {
        return policy_store
            .update_policy(
                id,
                content.account_id.clone(),
                &policies::UpdateOpts {
                    desc: Some(content.desc.clone()),
                    version: Some(content.version.clone()),
                    statement: Some(content.statement.clone()),
                    unscoped: Some(true),
                },
            )
            .await;
    }
    policy_store
        .create_policy(Some(id.to_owned()), content)
        .await?;
    Ok(())
}
