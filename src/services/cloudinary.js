// backend/src/services/cloudinary.js
import { v2 as cloudinary } from 'cloudinary';
import config from '../config/index.js';

cloudinary.config({
  cloud_name: config.cloudinary.cloudName,
  api_key:    config.cloudinary.apiKey,
  api_secret: config.cloudinary.apiSecret,
});

/**
 * Uploads an array of files (from `req.files`) to Cloudinary.
 * Returns an array of secure URLs.
 */
export async function uploadImages(files = []) {
  const uploadPromises = files.map(f =>
    cloudinary.uploader.upload(f.path || f.buffer, {
      upload_preset: config.cloudinary.uploadPreset,
      folder: 'manga_vehicles',
    })
  );
  const results = await Promise.all(uploadPromises);
  return results.map(r => r.secure_url);
}
