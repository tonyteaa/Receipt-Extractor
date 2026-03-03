-- Run this in MySQL Workbench or MySQL Command Line Client
-- This creates a new user for the Receipt Extractor app

CREATE USER 'receipt_admin'@'localhost' IDENTIFIED BY 'ReceiptPass123!';
GRANT ALL PRIVILEGES ON receipt_extractor.* TO 'receipt_admin'@'localhost';
FLUSH PRIVILEGES;

-- Test the new user
SELECT User, Host FROM mysql.user WHERE User = 'receipt_admin';

