-- Add up migration script here
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