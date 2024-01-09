-- Add down migration script here
DROP TABLE IF EXISTS `user`;
DROP TABLE IF EXISTS `policy`;
DROP TABLE IF EXISTS `role`;
DROP TABLE IF EXISTS `group`;
DROP TABLE IF EXISTS `role_bindings`;
DROP TABLE IF EXISTS `policy_bindings`;
DROP TABLE IF EXISTS `group_user`;
DROP TABLE IF EXISTS `user_policy`;
DROP TABLE IF EXISTS `client`;
DROP TABLE IF EXISTS `auth_request`;
DROP TABLE IF EXISTS `auth_code`;
DROP TABLE IF EXISTS `refresh_token`;
DROP TABLE IF EXISTS `key`;
DROP TABLE IF EXISTS `offline_session`;
DROP TABLE IF EXISTS `connector`;
