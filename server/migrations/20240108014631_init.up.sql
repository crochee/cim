-- Add up migration script here
CREATE TABLE `user` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user id',
    `account_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'account id',
    `name` VARCHAR(255) NOT NULL COMMENT 'user name',
    `nick_name` VARCHAR(255) NOT NULL COMMENT 'user nick name',
    `desc` VARCHAR(255) NOT NULL COMMENT 'user description,admin,develop',
    `email` VARCHAR(255) NULL DEFAULT NULL COMMENT 'user email',
    `mobile` CHAR(11) NULL DEFAULT NULL COMMENT 'user mobile',
    `sex` VARCHAR(6) NULL DEFAULT NULL COMMENT 'user sex',
    `image` VARCHAR(255) NULL DEFAULT NULL COMMENT 'user image',
    `secret` CHAR(64) NULL DEFAULT NULL COMMENT 'user password secret',
    `password` TEXT(414) NOT NULL COMMENT 'user password',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'user info';

CREATE TABLE `policy` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT,
    `account_id` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'account id',
    `user_id` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'user id',
    `desc` VARCHAR(255) NOT NULL COMMENT 'policy description',
    `version` VARCHAR(255) NOT NULL COMMENT 'policy version',
    `content` LONGTEXT NOT NULL COMMENT 'policy content',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    CONSTRAINT `content` CHECK (json_valid(`content`)),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'policy info';

CREATE TABLE `role` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'role ID',
    `name` VARCHAR(255) NOT NULL COMMENT 'role name',
    `desc` VARCHAR(255) NOT NULL COMMENT 'role description',
    `account_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'account id',
    `user_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'user id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'role info';

CREATE TABLE `group` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_group ID',
    `name` VARCHAR(255) NOT NULL COMMENT 'user_group name',
    `desc` VARCHAR(255) NOT NULL COMMENT 'user_group description',
    `account_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'account id',
    `user_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'user id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'group info';

CREATE TABLE `role_policy` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'role_policy ID',
    `role_id` VARCHAR(255) NOT NULL COMMENT 'role id',
    `policy_id` VARCHAR(255) NOT NULL COMMENT 'policy id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_role_id_policy_id_deleted` (`role_id`, `policy_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'role_policy info';

CREATE TABLE `role_bindings` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_role ID',
    `resource_id` VARCHAR(255) NOT NULL COMMENT 'resource id',
    `resource_type` VARCHAR(255) NOT NULL COMMENT 'resource type',
    `role_id` VARCHAR(255) NOT NULL COMMENT 'role id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_user_id_role_id_deleted` (`user_id`, `role_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'role_bindings info';

CREATE TABLE `group_policy` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_group_role ID',
    `group_id` VARCHAR(255) NOT NULL COMMENT 'group id',
    `policy_id` VARCHAR(255) NOT NULL COMMENT 'policy id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_group_id_policy_id_deleted` (`group_id`, `policy_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'group_policy info';

CREATE TABLE `group_user` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_group_user ID',
    `group_id` VARCHAR(255) NOT NULL COMMENT 'group id',
    `user_id` VARCHAR(255) NOT NULL COMMENT 'user id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_group_id_user_id_deleted` (`group_id`, `user_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'group_user info';

CREATE TABLE `user_policy` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_policy ID',
    `user_id` VARCHAR(255) NOT NULL COMMENT 'user id',
    `policy_id` VARCHAR(255) NOT NULL COMMENT 'policy id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_group_id_policy_id_deleted` (`group_id`, `policy_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'user_policy info';

create table client (
				id text not null primary key,
				secret text not null,
				redirect_uris bytea not null, -- JSON array of strings
				trusted_peers bytea not null, -- JSON array of strings
				public boolean not null,
				name text not null,
				logo_url text not null
			);

create table auth_request (
				id text not null primary key,
				client_id text not null,
				response_types bytea not null, -- JSON array of strings
				scopes bytea not null,         -- JSON array of strings
				redirect_uri text not null,
                code_challenge text not null default '',
                code_challenge_method text not null default '',
				nonce text not null,
				state varchar(4096),
                hmac_key bytea,
				force_approval_prompt boolean not null,
                claims_preferred_username text not null default '',
				logged_in boolean not null,

				claims_user_id text not null,
				claims_username text not null,
				claims_email text not null,
				claims_email_verified boolean not null,
				claims_groups bytea not null, -- JSON array of strings

				connector_id text not null,
				connector_data bytea,

				expiry timestamptz not null
			);

            create table auth_code (
				id text not null primary key,
				client_id text not null,
				scopes bytea not null, -- JSON array of strings
				nonce text not null,
				redirect_uri text not null,
                claims_preferred_username text not null default '',
                code_challenge text not null default '',
                code_challenge_method text not null default '',
				claims_user_id text not null,
				claims_username text not null,
				claims_email text not null,
				claims_email_verified boolean not null,
				claims_groups bytea not null, -- JSON array of strings

				connector_id text not null,
				connector_data bytea,

				expiry timestamptz not null
			);

            create table refresh_token (
				id text not null primary key,
				client_id text not null,
				scopes bytea not null, -- JSON array of strings
				nonce text not null,
				token text not null default '',
                claims_preferred_username text not null default '',
				claims_user_id text not null,
				claims_username text not null,
				claims_email text not null,
				claims_email_verified boolean not null,
				claims_groups bytea not null, -- JSON array of strings

				connector_id text not null,
				connector_data bytea,
                created_at timestamptz not null default '0001-01-01 00:00:00 UTC',
                last_used timestamptz not null default '0001-01-01 00:00:00 UTC',
                obsolete_token text default '',
			);

            create table password (
				email text not null primary key,
				hash bytea not null,
				username text not null,
				user_id text not null
			);

            create table keys (
				id text not null primary key,
				verification_keys bytea not null, -- JSON array
				signing_key bytea not null,       -- JSON object
				signing_key_pub bytea not null,   -- JSON object
				next_rotation timestamptz not null
			);
            create table offline_session (
				user_id text not null,
				conn_id text not null,
				refresh bytea not null,
				PRIMARY KEY (user_id, conn_id)
			);
            create table connector (
				id text not null primary key,
				type text not null,
				name text not null,
				resource_version text not null,
				config bytea,
                connector_data bytea
			);
            create table device_request (
				user_code text not null primary key,
				device_code text not null,
				client_id text not null,
				client_secret text ,
				scopes bytea not null, -- JSON array of strings
				expiry timestamptz not null
			);
            create table device_token (
				device_code text not null primary key,
				status text not null,
				token bytea,
				expiry timestamptz not null,
				last_request timestamptz not null,
                poll_interval integer not null,
                code_challenge text not null default '',
                code_challenge_method text not null default ''
			);
