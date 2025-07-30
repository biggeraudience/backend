// backend/src/config/index.js
import dotenv from 'dotenv';

dotenv.config();

const {
  PORT = 8000,
  JWT_SECRET,
  MONGODB_URI,
  CLOUDINARY_CLOUD_NAME,
  CLOUDINARY_API_KEY,
  CLOUDINARY_API_SECRET,
  CLOUDINARY_UPLOAD_PRESET,
} = process.env;

if (!JWT_SECRET || !MONGODB_URI) {
  console.error('❌ Missing required env vars (JWT_SECRET, MONGODB_URI)');
  process.exit(1);
}

export default {
  port: Number(PORT),
  jwtSecret: JWT_SECRET,
  mongoUri: MONGODB_URI,
  cloudinary: {
    cloudName: CLOUDINARY_CLOUD_NAME,
    apiKey: CLOUDINARY_API_KEY,
    apiSecret: CLOUDINARY_API_SECRET,
    uploadPreset: CLOUDINARY_UPLOAD_PRESET,
  },
};
