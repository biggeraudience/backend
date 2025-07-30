// backend/src/routes/auctions.js
import express from 'express';
import Auction from '../models/Auction.js';
import { authenticateJWT } from '../middleware/auth.js';

const router = express.Router();

// @route   POST /api/auctions
router.post('/', authenticateJWT, async (req, res, next) => {
  try {
    if (req.user.role !== 'admin') {
      return res.status(403).json({ message: 'Only admins can create auctions' });
    }

    const auction = new Auction(req.body);
    await auction.save();

    res.status(201).json(auction);
  } catch (err) {
    next(err);
  }
});

// @route   GET /api/auctions
router.get('/', async (req, res, next) => {
  try {
    const auctions = await Auction.find().sort({ createdAt: -1 });
    res.json(auctions);
  } catch (err) {
    next(err);
  }
});

// @route   GET /api/auctions/:id
router.get('/:id', async (req, res, next) => {
  try {
    const auction = await Auction.findById(req.params.id);
    if (!auction) {
      return res.status(404).json({ message: 'Auction not found' });
    }
    res.json(auction);
  } catch (err) {
    next(err);
  }
});

export default router;
