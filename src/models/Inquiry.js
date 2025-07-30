// backend/src/models/Inquiry.js
import mongoose from 'mongoose';

const inquirySchema = new mongoose.Schema({
  name:      { type: String, required: true, trim: true },
  email:     { type: String, required: true, trim: true },
  subject:   { type: String, trim: true },
  message:   { type: String, required: true },
  status:    { type: String, enum: ['New','Read','Responded'], default: 'New' },
  response:  { type: String },
}, {
  timestamps: true
});

export default mongoose.model('Inquiry', inquirySchema);
