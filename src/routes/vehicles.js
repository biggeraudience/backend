// backend/src/routes/vehicles.js
import express from 'express';
import Vehicle from '../models/Vehicle.js';
import { uploadImages } from '../services/cloudinary.js';
import { authenticateJWT } from '../middleware/auth.js';

const router = express.Router();

// @route   POST /api/vehicles
router.post('/', authenticateJWT, async (req, res, next) => {
  try {
    if (req.user.role !== 'admin') {
      return res.status(403).json({ message: 'Only admins can add vehicles' });
    }

    const imageUrls = await uploadImages(req.files || []);
    const vehicle = new Vehicle({ ...req.body, imageUrls });

    await vehicle.save();
    res.status(201).json(vehicle);
  } catch (err) {
    next(err);
  }
});

// @route   GET /api/vehicles
router.get('/', async (req, res, next) => {
  try {
    const vehicles = await Vehicle.find().sort({ createdAt: -1 });
    res.json(vehicles);
  } catch (err) {
    next(err);
  }
});

// @route   GET /api/vehicles/:id
router.get('/:id', async (req, res, next) => {
  try {
    const vehicle = await Vehicle.findById(req.params.id);
    if (!vehicle) {
      return res.status(404).json({ message: 'Vehicle not found' });
    }
    res.json(vehicle);
  } catch (err) {
    next(err);
  }
});

// @route   DELETE /api/vehicles/:id
router.delete('/:id', authenticateJWT, async (req, res, next) => {
  try {
    if (req.user.role !== 'admin') {
      return res.status(403).json({ message: 'Only admins can delete vehicles' });
    }

    await Vehicle.findByIdAndDelete(req.params.id);
    res.json({ message: 'Vehicle deleted' });
  } catch (err) {
    next(err);
  }
});

export default router;
