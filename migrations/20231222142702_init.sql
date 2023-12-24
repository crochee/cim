-- Add migration script here
CREATE TABLE `policy` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT,
    `owner_id` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'owner id',
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

CREATE TABLE `user` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user id',
    `owner_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'owner id',
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

CREATE TABLE `role` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'role ID',
    `name` VARCHAR(255) NOT NULL COMMENT 'role name',
    `desc` VARCHAR(255) NOT NULL COMMENT 'role description',
    `owner_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'owner id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'role info';

CREATE TABLE `group` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_group ID',
    `name` VARCHAR(255) NOT NULL COMMENT 'group name',
    `desc` VARCHAR(255) NOT NULL COMMENT 'group description',
    `owner_id` BIGINT(20) UNSIGNED NOT NULL COMMENT 'owner id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'group info';
