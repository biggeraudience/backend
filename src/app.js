import express from 'express';
import cors from 'cors';
import morgan from 'morgan';
import config from './config/index.js';
import connectMongo from './utils/db.js';
import { errorHandler } from './middleware/errorHandler.js';

import authRoutes from './routes/auth.js';
import userRoutes from './routes/users.js';
import vehicleRoutes from './routes/vehicles.js';
import auctionRoutes from './routes/auctions.js';
import inquiryRoutes from './routes/inquiries.js';
import { authenticateJWT } from './middleware/auth.js';

const app = express();

// Middleware
app.use(cors({ origin: true }));  // allow all origins
app.use(express.json({ limit: '10mb' }));
app.use(morgan('dev'));

// Public auth routes
app.use('/api/auth', authRoutes);

// Protect all routes below
app.use(authenticateJWT);

// Protected routes
app.use('/api/users', userRoutes);
app.use('/api/vehicles', vehicleRoutes);
app.use('/api/auctions', auctionRoutes);
app.use('/api/inquiries', inquiryRoutes);

// Error handler
app.use(errorHandler);

// Connect to MongoDB
(async () => {
  await connectMongo(config.mongoUri);
  console.log('✅ MongoDB connected');
})();

export default app;