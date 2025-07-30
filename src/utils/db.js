// backend/src/utils/db.js
import mongoose from 'mongoose';

export default async function connectMongo(uri) {
  try {
    await mongoose.connect(uri, {
      useNewUrlParser: true,
      useUnifiedTopology: true,
    });
  } catch (err) {
    console.error('🔥 MongoDB connection error:', err);
    process.exit(1);
  }
}
