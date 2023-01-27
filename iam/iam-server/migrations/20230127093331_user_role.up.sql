-- Add up migration script here
CREATE TABLE `user_role` (
    `id` BIGINT(20) UNSIGNED NOT NULL AUTO_INCREMENT COMMENT 'user_role ID',
    `user_id` VARCHAR(255) NOT NULL COMMENT 'user id',
    `role_id` VARCHAR(255) NOT NULL COMMENT 'role id',
    `deleted` BIGINT(20) UNSIGNED NOT NULL DEFAULT '0' COMMENT 'soft delete flag',
    `created_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT 'create time',
    `updated_at` DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT 'update time',
    `deleted_at` DATETIME(3) NULL DEFAULT NULL COMMENT 'delete time',
    PRIMARY KEY (`id`),
    UNIQUE `idx_user_id_role_id_deleted` (`user_id`, `role_id`, `deleted`) USING BTREE,
    INDEX `idx_deleted` (`deleted`) USING BTREE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_bin COMMENT = 'user_role info';