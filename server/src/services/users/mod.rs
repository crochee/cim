use crate::Result;

use crate::{
    store::{users, Store},
    AppState,
};

pub async fn put(
    app: &AppState,
    id: &str,
    content: &users::Content,
) -> Result<()> {
    let found = app
        .store
        .user_exist(id, content.account_id.clone(), true)
        .await?;
    if found {
        return app
            .store
            .update_user(
                id,
                content.account_id.clone(),
                &users::Opts {
                    name: Some(content.name.clone()),
                    nick_name: content.nick_name.clone(),
                    desc: Some(content.desc.clone()),
                    email: content.email.clone(),
                    mobile: content.mobile.clone(),
                    sex: content.sex.clone(),
                    image: content.image.clone(),
                    password: Some(content.password.clone()),
                    unscoped: Some(true),
                },
            )
            .await;
    }
    app.store.create_user(Some(id.to_owned()), content).await?;
    Ok(())
}

pub trait UserSrv {
	async fn create(&self, user:users::User, opts metav1.CreateOptions) error
	async fn update(&self.Context, user *v1.User, opts metav1.UpdateOptions) error
	async fn delete(&self.Context, username string, opts metav1.DeleteOptions) error
	async fn deleteCollection(&self, usernames []string, opts metav1.DeleteOptions) error
	async fn get(&self, username string, opts metav1.GetOptions) (*v1.User, error)
	async fn list(&self, opts metav1.ListOptions) (*v1.UserList, error)
	async fn changePassword(&self, user *v1.User) error
}
