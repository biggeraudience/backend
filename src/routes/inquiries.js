// backend/src/routes/inquiries.js
import express from 'express';
import Inquiry from '../models/Inquiry.js';
import { authenticateJWT } from '../middleware/auth.js';

const router = express.Router();

// @route   POST /api/inquiries
router.post('/', authenticateJWT, async (req, res, next) => {
  try {
    const inquiry = new Inquiry({
      ...req.body,
      user: req.user._id,
    });

    await inquiry.save();
    res.status(201).json(inquiry);
  } catch (err) {
    next(err);
  }
});

// @route   GET /api/inquiries
router.get('/', authenticateJWT, async (req, res, next) => {
  try {
    if (req.user.role !== 'admin') {
      return res.status(403).json({ message: 'Admins only' });
    }

    const inquiries = await Inquiry.find().populate('user', 'username email');
    res.json(inquiries);
  } catch (err) {
    next(err);
  }
});

export default router;
