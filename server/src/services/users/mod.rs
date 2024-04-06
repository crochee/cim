use slo::Result;
use storage::users;

pub async fn put_user<U>(
    user_store: &U,
    id: &str,
    content: &users::Content,
) -> Result<()>
where
    U: users::UserStore,
{
    let found = user_store
        .user_exist(id, content.account_id.clone(), true)
        .await?;
    if found {
        return user_store
            .update_user(
                id,
                content.account_id.clone(),
                &users::UpdateOpts {
                    desc: Some(content.desc.clone()),
                    claim: content.claim.clone(),
                    password: Some(content.password.clone()),
                    unscoped: Some(true),
                },
            )
            .await;
    }
    user_store.create_user(Some(id.to_owned()), content).await?;
    Ok(())
}
