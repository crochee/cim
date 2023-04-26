pub mod authrequests;
pub mod groups;
pub mod keys;
pub mod policies;
pub mod pool;
pub mod providers;
pub mod roles;
pub mod users;

use async_trait::async_trait;
use mockall::automock;

use cim_core::Result;
use sqlx::MySqlPool;

use crate::models::{
    auth_request::AuthRequest,
    key::Keys,
    policy::Policy,
    provider::Provider,
    role::{Role, RoleBindings},
    user::User,
    usergroup::{UserGroup, UserGroupBindings},
    List, ID,
};

#[automock]
#[async_trait]
pub trait Store: Send + Sync {
    // users
    async fn create_user(
        &self,
        id: Option<String>,
        content: &users::Content,
    ) -> Result<ID>;
    async fn update_user(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &users::Opts,
    ) -> Result<()>;
    async fn get_user(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<User>;
    async fn delete_user(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
    async fn list_user(&self, filter: &users::Querys) -> Result<List<User>>;
    async fn user_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;
    async fn user_get_password(
        &self,
        value: &users::UserSubject,
    ) -> Result<users::Password>;
    // user groups
    async fn create_user_group(
        &self,
        id: Option<String>,
        content: &groups::Content,
    ) -> Result<ID>;
    async fn update_user_group(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &groups::Opts,
    ) -> Result<()>;
    async fn get_user_group(
        &self,
        id: &str,
        filter: &groups::Querys,
    ) -> Result<UserGroupBindings>;
    async fn delete_user_group(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
    async fn list_user_group(
        &self,
        filter: &groups::Querys,
    ) -> Result<List<UserGroup>>;
    async fn user_group_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;
    async fn add_user_to_user_group(
        &self,
        id: &str,
        account_id: &str,
        user_id: &str,
    ) -> Result<()>;
    async fn delete_user_from_user_group(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<()>;
    async fn add_role_to_user_group(
        &self,
        id: &str,
        account_id: &str,
        role_id: &str,
    ) -> Result<()>;
    async fn delete_role_from_user_group(
        &self,
        id: &str,
        role_id: &str,
    ) -> Result<()>;
    // roles
    async fn create_role(
        &self,
        id: Option<String>,
        content: &roles::Content,
    ) -> Result<ID>;
    async fn update_role(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &roles::Opts,
    ) -> Result<()>;
    async fn get_role(
        &self,
        id: &str,
        filter: &roles::Querys,
    ) -> Result<RoleBindings>;
    async fn delete_role(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
    async fn list_role(&self, filter: &roles::Querys) -> Result<List<Role>>;
    async fn role_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;
    async fn add_user_to_role(
        &self,
        id: &str,
        account_id: &str,
        user_id: &str,
    ) -> Result<()>;
    async fn delete_user_from_role(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<()>;
    async fn add_policy_to_role(
        &self,
        id: &str,
        account_id: &str,
        policy_id: &str,
    ) -> Result<()>;
    async fn delete_policy_from_role(
        &self,
        id: &str,
        policy_id: &str,
    ) -> Result<()>;
    // policies
    async fn create_policy(
        &self,
        id: Option<String>,
        content: &policies::Content,
    ) -> Result<ID>;
    async fn update_policy(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &policies::Opts,
    ) -> Result<()>;
    async fn get_policy(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Policy>;
    async fn delete_policy(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
    async fn list_policy(
        &self,
        filter: &policies::Querys,
    ) -> Result<List<Policy>>;
    async fn policy_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;
    async fn get_policy_by_user(
        &self,
        user_id_str: &str,
    ) -> Result<Vec<Policy>>;
    // providers
    async fn create_provider(&self, content: &providers::Content)
        -> Result<ID>;
    async fn get_provider(&self, id: &str) -> Result<Provider>;
    async fn list_provider(&self) -> Result<Vec<Provider>>;
    // key
    async fn get_key(&self) -> Result<Keys>;
    async fn update_key(&self, nk: &Keys) -> Result<()>;
    async fn create_key(&self, nk: &Keys) -> Result<()>;
    // auth request
    async fn create_authrequest(
        &self,
        content: &authrequests::Content,
    ) -> Result<ID>;
    async fn get_authrequests(&self, id: &str) -> Result<AuthRequest>;
    async fn update_authrequests(
        &self,
        id: &str,
        opts: &authrequests::UpdateOpts,
    ) -> Result<()>;
    async fn delete_authrequests(&self, id: &str) -> Result<()>;
}

#[derive(Clone)]
pub struct MariadbStore {
    pub pool: MySqlPool,
}

impl MariadbStore {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Store for MariadbStore {
    // users
    async fn create_user(
        &self,
        id: Option<String>,
        content: &users::Content,
    ) -> Result<ID> {
        users::create(&self.pool, id, content).await
    }
    async fn update_user(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &users::Opts,
    ) -> Result<()> {
        users::update(&self.pool, id, account_id, opts).await
    }
    async fn get_user(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<User> {
        users::get(&self.pool, id, account_id).await
    }
    async fn delete_user(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()> {
        users::delete(&self.pool, id, account_id).await
    }
    async fn list_user(&self, filter: &users::Querys) -> Result<List<User>> {
        users::list(&self.pool, filter).await
    }
    async fn user_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool> {
        users::exist(&self.pool, id, account_id, unscoped).await
    }
    async fn user_get_password(
        &self,
        value: &users::UserSubject,
    ) -> Result<users::Password> {
        users::get_password(&self.pool, value).await
    }
    // user groups
    async fn create_user_group(
        &self,
        id: Option<String>,
        content: &groups::Content,
    ) -> Result<ID> {
        groups::create(&self.pool, id, content).await
    }
    async fn update_user_group(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &groups::Opts,
    ) -> Result<()> {
        groups::update(&self.pool, id, account_id, opts).await
    }
    async fn get_user_group(
        &self,
        id: &str,
        filter: &groups::Querys,
    ) -> Result<UserGroupBindings> {
        groups::get(&self.pool, id, filter).await
    }
    async fn delete_user_group(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()> {
        groups::delete(&self.pool, id, account_id).await
    }
    async fn list_user_group(
        &self,
        filter: &groups::Querys,
    ) -> Result<List<UserGroup>> {
        groups::list(&self.pool, filter).await
    }
    async fn user_group_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool> {
        groups::exist(&self.pool, id, account_id, unscoped).await
    }
    async fn add_user_to_user_group(
        &self,
        id: &str,
        account_id: &str,
        user_id: &str,
    ) -> Result<()> {
        groups::add_user(&self.pool, id, account_id, user_id).await
    }
    async fn delete_user_from_user_group(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<()> {
        groups::delete_user(&self.pool, id, user_id).await
    }
    async fn add_role_to_user_group(
        &self,
        id: &str,
        account_id: &str,
        role_id: &str,
    ) -> Result<()> {
        groups::add_role(&self.pool, id, account_id, role_id).await
    }
    async fn delete_role_from_user_group(
        &self,
        id: &str,
        role_id: &str,
    ) -> Result<()> {
        groups::delete_role(&self.pool, id, role_id).await
    }
    // roles
    async fn create_role(
        &self,
        id: Option<String>,
        content: &roles::Content,
    ) -> Result<ID> {
        roles::create(&self.pool, id, content).await
    }
    async fn update_role(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &roles::Opts,
    ) -> Result<()> {
        roles::update(&self.pool, id, account_id, opts).await
    }
    async fn get_role(
        &self,
        id: &str,
        filter: &roles::Querys,
    ) -> Result<RoleBindings> {
        roles::get(&self.pool, id, filter).await
    }
    async fn delete_role(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()> {
        roles::delete(&self.pool, id, account_id).await
    }
    async fn list_role(&self, filter: &roles::Querys) -> Result<List<Role>> {
        roles::list(&self.pool, filter).await
    }
    async fn role_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool> {
        roles::exist(&self.pool, id, account_id, unscoped).await
    }
    async fn add_user_to_role(
        &self,
        id: &str,
        account_id: &str,
        user_id: &str,
    ) -> Result<()> {
        roles::add_user(&self.pool, id, account_id, user_id).await
    }
    async fn delete_user_from_role(
        &self,
        id: &str,
        user_id: &str,
    ) -> Result<()> {
        roles::delete_user(&self.pool, id, user_id).await
    }
    async fn add_policy_to_role(
        &self,
        id: &str,
        account_id: &str,
        policy_id: &str,
    ) -> Result<()> {
        roles::add_policy(&self.pool, id, account_id, policy_id).await
    }
    async fn delete_policy_from_role(
        &self,
        id: &str,
        policy_id: &str,
    ) -> Result<()> {
        roles::delete_policy(&self.pool, id, policy_id).await
    }
    // policies
    async fn create_policy(
        &self,
        id: Option<String>,
        content: &policies::Content,
    ) -> Result<ID> {
        policies::create(&self.pool, id, content).await
    }
    async fn update_policy(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &policies::Opts,
    ) -> Result<()> {
        policies::update(&self.pool, id, account_id, opts).await
    }
    async fn get_policy(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Policy> {
        policies::get(&self.pool, id, account_id).await
    }
    async fn delete_policy(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()> {
        policies::delete(&self.pool, id, account_id).await
    }
    async fn list_policy(
        &self,
        filter: &policies::Querys,
    ) -> Result<List<Policy>> {
        policies::list(&self.pool, filter).await
    }
    async fn policy_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool> {
        policies::exist(&self.pool, id, account_id, unscoped).await
    }
    async fn get_policy_by_user(
        &self,
        user_id_str: &str,
    ) -> Result<Vec<Policy>> {
        policies::get_by_user(&self.pool, user_id_str).await
    }
    // providers
    async fn create_provider(
        &self,
        content: &providers::Content,
    ) -> Result<ID> {
        providers::create(&self.pool, content).await
    }
    async fn get_provider(&self, id: &str) -> Result<Provider> {
        providers::get(&self.pool, id).await
    }
    async fn list_provider(&self) -> Result<Vec<Provider>> {
        providers::list(&self.pool).await
    }
    // key
    async fn get_key(&self) -> Result<Keys> {
        keys::get(&self.pool).await
    }
    async fn update_key(&self, nk: &Keys) -> Result<()> {
        keys::update(&self.pool, nk).await
    }
    async fn create_key(&self, nk: &Keys) -> Result<()> {
        keys::create(&self.pool, nk).await
    }
    // auth request
    async fn create_authrequest(
        &self,
        content: &authrequests::Content,
    ) -> Result<ID> {
        authrequests::create(&self.pool, content).await
    }
    async fn get_authrequests(&self, id: &str) -> Result<AuthRequest> {
        authrequests::get(&self.pool, id).await
    }
    async fn update_authrequests(
        &self,
        id: &str,
        opts: &authrequests::UpdateOpts,
    ) -> Result<()> {
        authrequests::update(&self.pool, id, opts).await
    }
    async fn delete_authrequests(&self, id: &str) -> Result<()> {
        authrequests::delete(&self.pool, id).await
    }
}
