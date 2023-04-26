-- Add up migration script here
CREATE TABLE `auth_request` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'auth_request ID',
    `client_id` VARCHAR(255) NOT NULL COMMENT 'client_id',
    `response_type` VARCHAR(255) NOT NULL COMMENT 'response_type',
    `scope` VARCHAR(255) NOT NULL COMMENT 'scope',
    `redirect_url` VARCHAR(255) NOT NULL COMMENT 'redirect_url',
    `nonce` TEXT NOT NULL COMMENT 'nonce',
    `state` TEXT NOT NULL COMMENT 'state',
    `force_approval` TINYINT(1) NOT NULL COMMENT 'force_approval',
    `expiry` BIGINT(20) NOT NULL COMMENT 'expiry',
    `logged_in` TINYINT(1) NOT NULL COMMENT 'logged_in',
    `claims` TEXT NULL DEFAULT NULL COMMENT 'claims',
    `hmac_key` VARCHAR(255) NOT NULL COMMENT 'hmac_key',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    CONSTRAINT `claims` CHECK (json_valid(`claims`)),
    UNIQUE `idx_redirect_url_deleted` (`redirect_url`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'auth_request info';