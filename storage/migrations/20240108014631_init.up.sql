-- Add up migration script here
CREATE TABLE `user` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user id',
    `account_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'account id',
    `desc` VARCHAR(255) NOT NULL COMMENT 'user description,admin,develop',
    `email` VARCHAR(255) NULL DEFAULT NULL COMMENT 'user email',
    `email_verified` BOOLEAN NULL DEFAULT NULL COMMENT 'email verified flag',
    `name` VARCHAR(255) NULL DEFAULT NULL COMMENT 'user name',
    `given_name` VARCHAR(255) NULL DEFAULT NULL COMMENT 'given name',
    `family_name` VARCHAR(255) NULL DEFAULT NULL COMMENT 'family name',
    `middle_name` VARCHAR(255) NULL DEFAULT NULL COMMENT 'middle name',
    `nickname` VARCHAR(255) NULL DEFAULT NULL COMMENT 'nickname',
    `preferred_username` VARCHAR(255) NULL DEFAULT NULL COMMENT 'preferred username',
    `profile` VARCHAR(255) NULL DEFAULT NULL COMMENT 'profile',
    `picture` VARCHAR(255) NULL DEFAULT NULL COMMENT 'picture',
    `website` VARCHAR(255) NULL DEFAULT NULL COMMENT 'website',
    `gender` VARCHAR(10) NULL DEFAULT NULL COMMENT 'gender',
    `birthday` VARCHAR(255) NULL DEFAULT NULL COMMENT 'birthday',
    `birthdate` VARCHAR(255) NULL DEFAULT NULL COMMENT 'birthdate',
    `zoneinfo` VARCHAR(255) NULL DEFAULT NULL COMMENT 'zoneinfo',
    `locale` VARCHAR(255) NULL DEFAULT NULL COMMENT 'locale',
    `phone_number` VARCHAR(255) NULL DEFAULT NULL COMMENT 'phone number',
    `phone_number_verified` BOOLEAN NULL DEFAULT NULL COMMENT 'phone number verified',
    `address` TEXT NULL DEFAULT NULL COMMENT 'address',
    `secret` CHAR(64) NOT NULL COMMENT 'password secret',
    `password` TEXT(417) NOT NULL COMMENT 'user password',
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
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'group info';

CREATE TABLE `role_bindings` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_role ID',
    `role_id` BIGINT(20) NOT NULL COMMENT 'role id',
    `user_type` TINYINT NOT NULL COMMENT 'user type',
    `user_id` VARCHAR(255) NOT NULL COMMENT 'any can use role',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_role_id_user_type_user_id_deleted` (`role_id`, `user_type`, `user_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'role_bindings info';

CREATE TABLE `policy_bindings` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_group_role ID',
    `policy_id` BIGINT(20) NOT NULL COMMENT 'policy id',
    `bindings_type` TINYINT NOT NULL COMMENT 'bindings type 1:user 2:group 3:role',
    `bindings_id` BIGINT(20) NOT NULL COMMENT 'be bindings object id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_policy_id_bindings_type_bindings_id_deleted` (`policy_id`, `bindings_type`, `bindings_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'policy bindings info';

CREATE TABLE `group_user` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_group_user ID',
    `group_id` BIGINT(20) NOT NULL COMMENT 'group id',
    `user_id` BIGINT(20) NOT NULL COMMENT 'user id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_group_id_user_id_deleted` (`group_id`, `user_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'group_user info';

CREATE TABLE `client` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'client ID',
    `secret` TEXT NOT NULL COMMENT 'client secret',
    `redirect_uris` TEXT NOT NULL, -- JSON array of strings
    `trusted_peers` TEXT NOT NULL, -- JSON array of strings
    `name` VARCHAR(255) NOT NULL COMMENT 'client name',
    `logo_url` TEXT NOT NULL COMMENT 'client logo url',
    `account_id` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'account id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_name_deleted` (`name`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'client info';

CREATE TABLE `auth_request` (
    `id` CHAR(36) NOT NULL COMMENT 'auth_request ID',
    `client_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'client id',
	`response_types` TEXT NOT NULL, -- JSON array of strings
	`scopes` TEXT NOT NULL, -- JSON array of strings
	`redirect_uri` TEXT NOT NULL,
    `code_challenge` TEXT NOT NULL DEFAULT '',
    `code_challenge_method` TEXT NOT NULL DEFAULT '',
	`nonce` TEXT NOT NULL,
	`state` VARCHAR(4096),
    `hmac_key` TEXT,
	`force_approval_prompt` BOOLEAN NOT NULL,
	`logged_in` BOOLEAN NOT NULL,

	`claim` TEXT NOT NULL,

	`connector_id` TEXT NOT NULL,
	`connector_data` TEXT,

	`expiry` BIGINT(20) NOT NULL,

    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'auth_request info';

CREATE TABLE `auth_code` (
    `id` CHAR(36) NOT NULL COMMENT 'auth_code ID',
    `client_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'client id',
	`scopes` TEXT NOT NULL, -- JSON array of strings
	`nonce` TEXT NOT NULL,
	`redirect_uri` TEXT NOT NULL,
    `code_challenge` TEXT NOT NULL DEFAULT '',
    `code_challenge_method` TEXT NOT NULL DEFAULT '',

	`claim` TEXT NOT NULL,

	`connector_id` TEXT NOT NULL,
	`connector_data` TEXT,

	`expiry` BIGINT(20) NOT NULL,

    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'auth_code info';

CREATE TABLE `refresh_token` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'refresh_token ID',
    `client_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'client id',
	`scopes` TEXT NOT NULL, -- JSON array of strings
	`nonce` TEXT NOT NULL,
	`token` TEXT NOT NULL DEFAULT '',
	`obsolete_token` TEXT NOT NULL DEFAULT '',

    `claim` TEXT NOT NULL,

	`connector_id` TEXT NOT NULL,
	`connector_data` TEXT,

    `last_used_at` DATETIME(3) NOT NULL COMMENT 'last used time',

    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'refresh_token info';

CREATE TABLE `key` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'key ID',
	`verification_keys` TEXT NOT NULL, -- JSON array
	`signing_key` TEXT NOT NULL,       -- JSON object
	`signing_key_pub` TEXT NOT NULL,   -- JSON object
	`next_rotation` BIGINT(20) UNSIGNED NOT NULL,

    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'keys info';

CREATE TABLE `offline_session` (
	`user_id` VARCHAR(255) NOT NULL,
	`conn_id` VARCHAR(255) NOT NULL,
	`refresh` TEXT NOT NULL,

    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
	PRIMARY KEY (`user_id`, `conn_id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'offline_session info';

CREATE TABLE `connector` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'connector ID',
	`type` VARCHAR(255) NOT NULL,
	`name` TEXT NOT NULL,
	`resource_version` TEXT NOT NULL,
	`config` TEXT,
    `connector_data` TEXT,

    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
	PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'connector info';
