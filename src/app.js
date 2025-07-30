// backend/src/app.js
import express from 'express';
import cors from 'cors';
import morgan from 'morgan';
import config from './config/index.js';
import connectMongo from './utils/db.js';
import authRoutes from './routes/auth.js';
import vehicleRoutes from './routes/vehicles.js';
import auctionRoutes from './routes/auctions.js';
import inquiryRoutes from './routes/inquiries.js';
import userRoutes from './routes/users.js';
import { errorHandler } from './middleware/errorHandler.js';
import { authenticateJWT } from './middleware/auth.js';

const app = express();

// Middleware
app.use(cors());
app.use(express.json({ limit: '10mb' }));
app.use(morgan('dev'));

// Mount public/auth routes
app.use('/auth', authRoutes);

// Protect all following routes
app.use(authenticateJWT);

// Mount resource routes
app.use('/vehicles', vehicleRoutes);
app.use('/auctions', auctionRoutes);
app.use('/admin/vehicles', vehicleRoutes);     // admin CRUD
app.use('/admin/auctions', auctionRoutes);     // admin CRUD
app.use('/admin/inquiries', inquiryRoutes);
app.use('/admin/users', userRoutes);

// Global error handler
app.use(errorHandler);

// Connect DB & export app
(async () => {
  await connectMongo(config.mongoUri);
  console.log('✅ MongoDB connected');
})();

export default app;
