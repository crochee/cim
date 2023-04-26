-- Add up migration script here
CREATE TABLE `provider` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'provider ID',
    `secret` VARCHAR(255) NOT NULL COMMENT 'provider secret',
    `redirect_url` VARCHAR(255) NOT NULL COMMENT 'provider redirect_url',
    `name` VARCHAR(255) NOT NULL COMMENT 'provider name',
    `prompt` VARCHAR(255) NOT NULL COMMENT 'provider prompt',
    `logo_url` VARCHAR(255) NOT NULL COMMENT 'provider logo url',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_redirect_url_deleted` (`redirect_url`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'provider info';