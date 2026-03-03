const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const rateLimit = require('express-rate-limit');
require('dotenv').config();

const { testConnection } = require('./config/database');
const licenseRoutes = require('./routes/license');
const shopifyRoutes = require('./routes/shopify');
const adminRoutes = require('./routes/admin');
const usageRoutes = require('./routes/usage');
const versionRoutes = require('./routes/version');

const app = express();
const PORT = process.env.PORT || 3000;

// Trust proxy for Railway (needed for rate limiting and getting real IP)
app.set('trust proxy', 1);

// Security middleware
app.use(helmet());
app.use(cors());

// Capture raw body for Shopify webhook signature verification
app.use(express.json({
    verify: (req, res, buf) => {
        // Store raw body for webhook signature verification
        req.rawBody = buf;
    }
}));
app.use(express.urlencoded({ extended: true }));

// Rate limiting
const limiter = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 100, // Limit each IP to 100 requests per windowMs
    message: 'Too many requests from this IP, please try again later.'
});

app.use('/api/', limiter);

// Health check endpoint
app.get('/health', (req, res) => {
    res.json({ 
        status: 'ok', 
        timestamp: new Date().toISOString(),
        service: 'Receipt Extractor License API'
    });
});

// API Routes
app.use('/api/license', licenseRoutes);
app.use('/api/shopify', shopifyRoutes);
app.use('/api/admin', adminRoutes);
app.use('/api/usage', usageRoutes);
app.use('/api/version', versionRoutes);

// 404 handler
app.use((req, res) => {
    res.status(404).json({ error: 'Endpoint not found' });
});

// Error handler
app.use((err, req, res, next) => {
    console.error('Server error:', err);
    res.status(500).json({ error: 'Internal server error' });
});

// Start server
async function startServer() {
    try {
        console.log('🔄 Starting server...');
        console.log(`📍 Attempting to bind to 0.0.0.0:${PORT}`);

        // Test database connection
        const dbConnected = await testConnection();

        if (!dbConnected) {
            console.error('❌ Failed to connect to database. Please check your configuration.');
            console.log('\n💡 Make sure to:');
            console.log('   1. Copy .env.example to .env');
            console.log('   2. Update database credentials in .env');
            console.log('   3. Run: npm run init-db\n');
            process.exit(1);
        }

        console.log('🔄 Database connected, starting HTTP server...');

        const server = app.listen(PORT, '0.0.0.0', () => {
            const address = server.address();
            console.log('🔍 Server address:', address);
            console.log('\n╔════════════════════════════════════════════╗');
            console.log('║  Receipt Extractor License API Server     ║');
            console.log('╚════════════════════════════════════════════╝\n');
            console.log(`🚀 Server running on port ${PORT}`);
            console.log(`📍 Environment: ${process.env.NODE_ENV || 'development'}`);
            console.log(`🔗 Health check: http://localhost:${PORT}/health`);
            console.log('\n📋 Available Endpoints:');
            console.log('   POST   /api/license/validate');
            console.log('   POST   /api/shopify/order-created');
            console.log('   GET    /api/shopify/stats');
            console.log('   GET    /api/admin/stats');
            console.log('   GET    /api/admin/licenses');
            console.log('   GET    /api/admin/license/:key');
            console.log('   POST   /api/admin/license/revoke');
            console.log('   DELETE /api/admin/activation/:id');
            console.log('   GET    /api/usage');
            console.log('   POST   /api/usage/increment');
            console.log('   POST   /api/usage/check');
            console.log('   GET    /api/usage/plans');
            console.log('   GET    /api/version');
            console.log('   GET    /api/version/changelog');
            console.log('\n✅ Server ready to accept requests!\n');
        });

        server.on('error', (error) => {
            if (error.code === 'EADDRINUSE') {
                console.error(`❌ Port ${PORT} is already in use!`);
                console.log('💡 Try one of these:');
                console.log(`   1. Kill the process using port ${PORT}`);
                console.log('   2. Change PORT in .env file');
                console.log(`   3. Run: netstat -ano | findstr :${PORT}`);
            } else {
                console.error('❌ Server error:', error);
            }
            process.exit(1);
        });
    } catch (error) {
        console.error('Failed to start server:', error);
        process.exit(1);
    }
}

startServer();

module.exports = app;

