// backend/src/routes/users.js
import express from 'express';
import User from '../models/User.js';
import { authenticateJWT } from '../middleware/auth.js';

const router = express.Router();

// @route   GET /api/users/me
router.get('/me', authenticateJWT, async (req, res, next) => {
  try {
    const user = req.user;
    res.json({
      id: user._id,
      username: user.username,
      email: user.email,
      role: user.role,
      status: user.status,
    });
  } catch (err) {
    next(err);
  }
});

// @route   GET /api/users
router.get('/', authenticateJWT, async (req, res, next) => {
  try {
    if (req.user.role !== 'admin') {
      return res.status(403).json({ message: 'Forbidden: Admins only' });
    }

    const users = await User.find().select('-passwordHash');
    res.json(users);
  } catch (err) {
    next(err);
  }
});

export default router;
