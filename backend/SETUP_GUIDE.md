# Backend Setup Guide

Complete step-by-step guide to set up the Receipt Extractor backend API.

## 📋 Prerequisites Checklist

- [ ] Windows 10/11 or Linux/macOS
- [ ] Node.js 16+ installed ([Download](https://nodejs.org/))
- [ ] MySQL 8.0+ installed ([Download](https://dev.mysql.com/downloads/mysql/))
- [ ] Git installed (optional)

## 🔧 Step 1: Install MySQL

### Windows

1. **Download MySQL Installer:**
   - Go to: https://dev.mysql.com/downloads/installer/
   - Download "Windows (x86, 32-bit), MSI Installer"

2. **Run Installer:**
   - Choose "Developer Default" or "Server only"
   - Set root password (remember this!)
   - Keep default port: 3306

3. **Verify Installation:**
   ```powershell
   mysql --version
   ```

### macOS

```bash
brew install mysql
brew services start mysql
mysql_secure_installation
```

### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install mysql-server
sudo mysql_secure_installation
```

## 🗄️ Step 2: Create Database User

1. **Login to MySQL as root:**
   ```bash
   mysql -u root -p
   ```

2. **Create database user:**
   ```sql
   CREATE USER 'receipt_admin'@'localhost' IDENTIFIED BY 'your_secure_password';
   GRANT ALL PRIVILEGES ON receipt_extractor.* TO 'receipt_admin'@'localhost';
   FLUSH PRIVILEGES;
   EXIT;
   ```

## 📦 Step 3: Install Backend

1. **Navigate to backend folder:**
   ```bash
   cd backend
   ```

2. **Install Node.js dependencies:**
   ```bash
   npm install
   ```

   This will install:
   - express (web framework)
   - mysql2 (database driver)
   - dotenv (environment variables)
   - cors, helmet (security)
   - express-rate-limit (rate limiting)

## ⚙️ Step 4: Configure Environment

1. **Copy example environment file:**
   ```bash
   cp .env.example .env
   ```

2. **Edit `.env` file:**
   
   Open `.env` in a text editor and update:

   ```env
   # Database (use credentials from Step 2)
   DB_HOST=localhost
   DB_PORT=3306
   DB_USER=receipt_admin
   DB_PASSWORD=your_secure_password
   DB_NAME=receipt_extractor

   # API Keys (already filled in)
   GROQ_API_KEY=your_groq_api_key_here
   OPENAI_API_KEY=your_openai_api_key_here

   # Leave these for now (will be set later)
   ADMIN_API_KEY=
   SHOPIFY_WEBHOOK_SECRET=
   ```

## 🏗️ Step 5: Initialize Database

1. **Run initialization script:**
   ```bash
   npm run init-db
   ```

2. **You'll see output like:**
   ```
   🚀 Initializing Receipt Extractor Database...
   ✅ Connected to MySQL server
   📝 Executing schema...
   ✅ Database schema created successfully
   ✅ Database initialization complete!

   📋 Admin API Key (save this!): a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6...
   ```

3. **Copy the Admin API Key** and add it to `.env`:
   ```env
   ADMIN_API_KEY=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6...
   ```

## 🔑 Step 6: Generate License Keys

1. **Generate Pro tier keys:**
   ```bash
   npm run generate-keys pro 100
   ```

2. **Generate Budget tier keys:**
   ```bash
   npm run generate-keys budget 100
   ```

3. **You'll see:**
   ```
   🔑 Generating 100 PRO license keys...

   1. PRO-ABCD-EFGH-IJKL ✅
   2. PRO-MNOP-QRST-UVWX ✅
   ...

   ✅ Successfully inserted 100/100 keys

   📊 Current Database Statistics:
   ─────────────────────────────────────
   PRO: 100 available / 100 total
   BUDGET: 100 available / 100 total
   ```

## 🚀 Step 7: Start the Server

1. **Start in development mode:**
   ```bash
   npm run dev
   ```

2. **Or start in production mode:**
   ```bash
   npm start
   ```

3. **You should see:**
   ```
   ╔════════════════════════════════════════════╗
   ║  Receipt Extractor License API Server     ║
   ╚════════════════════════════════════════════╝

   ✅ MySQL database connected successfully
   🚀 Server running on port 3000
   📍 Environment: development
   🔗 Health check: http://localhost:3000/health

   📋 Available Endpoints:
      POST   /api/license/validate
      POST   /api/shopify/order-created
      ...

   ✅ Server ready to accept requests!
   ```

## ✅ Step 8: Test the API

1. **Test health endpoint:**
   ```bash
   curl http://localhost:3000/health
   ```

2. **Test license validation:**
   ```bash
   curl -X POST http://localhost:3000/api/license/validate \
     -H "Content-Type: application/json" \
     -d '{"license_key": "PRO-ABCD-EFGH-IJKL"}'
   ```

   Replace `PRO-ABCD-EFGH-IJKL` with an actual key from Step 6.

3. **Expected response:**
   ```json
   {
     "valid": true,
     "tier": "pro",
     "api_keys": {
       "groq": "gsk_...",
       "openai": "sk-..."
     }
   }
   ```

## 🎉 Success!

Your backend is now running! Next steps:

- [ ] Deploy to production server (see DEPLOYMENT.md)
- [ ] Set up Shopify webhook
- [ ] Update Rust app to use backend API
- [ ] Test end-to-end flow

## 🔧 Common Issues

### "Cannot connect to MySQL"
- Check MySQL is running: `mysql -u root -p`
- Verify credentials in `.env`
- Check port 3306 is not blocked

### "EADDRINUSE: Port 3000 already in use"
- Change PORT in `.env` to 3001 or another port
- Or kill process using port 3000

### "No available license keys"
- Run: `npm run generate-keys pro 100`
- Check database: `mysql -u receipt_admin -p receipt_extractor`
  ```sql
  SELECT COUNT(*) FROM license_keys WHERE is_used = FALSE;
  ```

## 📞 Need Help?

Check the main README.md for API documentation and troubleshooting.

