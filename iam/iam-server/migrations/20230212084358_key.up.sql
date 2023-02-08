-- Add up migration script here
CREATE TABLE `key` (
    `signing_key` TEXT NOT NULL COMMENT 'signing_key',
    `verification_keys` TEXT NOT NULL COMMENT 'verification_keys',
    `next_rotation` BIGINT(20) NOT NULL COMMENT 'next_rotation',
    `enable` TINYINT(1) NOT NULL COMMENT 'enable',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    UNIQUE `idx_enable_deleted` (`enable`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'key info';