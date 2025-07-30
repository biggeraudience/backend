// backend/src/routes/auth.js
import express from 'express';
import User from '../models/User.js';
import { signToken } from '../services/jwt.js';

const router = express.Router();

// @route   POST /api/auth/register
router.post('/register', async (req, res, next) => {
  try {
    const { username, email, password } = req.body;

    const exists = await User.findOne({ email });
    if (exists) {
      return res.status(400).json({ message: 'Email already in use' });
    }

    const user = new User({ username, email });
    user.password = password;
    await user.save();

    const token = signToken({ id: user._id });

    res.status(201).json({ token, user: { id: user._id, username, email } });
  } catch (err) {
    next(err);
  }
});

// @route   POST /api/auth/login
router.post('/login', async (req, res, next) => {
  try {
    const { email, password } = req.body;

    const user = await User.findOne({ email });
    if (!user || !user.verifyPassword(password)) {
      return res.status(401).json({ message: 'Invalid email or password' });
    }

    const token = signToken({ id: user._id });

    res.json({ token, user: { id: user._id, username: user.username, email } });
  } catch (err) {
    next(err);
  }
});

export default router;
