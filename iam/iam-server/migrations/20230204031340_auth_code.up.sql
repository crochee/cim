-- Add up migration script here
CREATE TABLE `auth_code` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'auth_code ID',
    `client_id` VARCHAR(255) NOT NULL COMMENT 'client_id',
    `redirect_url` TEXT NOT NULL COMMENT 'redirect_url',
    `scope` VARCHAR(255) NOT NULL COMMENT 'scope',
    `nonce` TEXT NOT NULL COMMENT 'nonce',
    `expiry` BIGINT(20) NOT NULL COMMENT 'expiry',
    `claims` TEXT NULL DEFAULT NULL COMMENT 'claims',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    CONSTRAINT `claims` CHECK (json_valid(`claims`)),
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'auth_code info';