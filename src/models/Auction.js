// backend/src/models/Auction.js
import mongoose from 'mongoose';

const auctionSchema = new mongoose.Schema({
  vehicleId:           { type: mongoose.Schema.Types.ObjectId, ref: 'Vehicle', required: true },
  startTime:           { type: Date, required: true },
  endTime:             { type: Date, required: true },
  startingBid:         { type: Number, required: true },
  currentHighestBid:   { type: Number, default: 0 },
  status:              { type: String, enum: ['pending','active','closed'], default: 'pending' },
}, {
  timestamps: true
});

export default mongoose.model('Auction', auctionSchema);
