// backend/src/models/Vehicle.js
import mongoose from 'mongoose';

const vehicleSchema = new mongoose.Schema({
  make:          { type: String, required: true, trim: true },
  model:         { type: String, required: true, trim: true },
  year:          { type: Number, required: true },
  price:         { type: Number, required: true },
  mileage:       { type: String, required: true },
  exteriorColor: { type: String },
  interiorColor: { type: String },
  engine:        { type: String },
  transmission:  { type: String },
  fuelType:      { type: String },
  description:   { type: String },
  imageUrls:     [{ type: String }],
  engineSound:   { type: String },
  features:      [{ type: String }],
  status:        { type: String, enum: ['available','auctioning','sold','pending_inspection'], default: 'available' },
  isFeatured:    { type: Boolean, default: false },
}, {
  timestamps: { createdAt: 'createdAt', updatedAt: 'lastUpdated' }
});

export default mongoose.model('Vehicle', vehicleSchema);
