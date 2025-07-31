import express from 'express';
import User from '../models/User.js';
import { signToken } from '../services/jwt.js';

const router = express.Router();

// POST /api/auth/register
router.post('/register', async (req, res, next) => {
  try {
    const { username, email, password, role } = req.body;
    if (!username || !email || !password) {
      return res.status(400).json({ message: 'Missing fields' });
    }
    const exists = await User.findOne({ email });
    if (exists) {
      return res.status(400).json({ message: 'Email already in use' });
    }
    // allow role override (default to 'user')
    const user = new User({ username, email, role: role || 'user' });
    user.password = password;
    await user.save();

    const token = signToken({ id: user._id });
    res.status(201).json({
      token,
      user: { id: user._id, username, email, role: user.role },
    });
  } catch (err) {
    next(err);
  }
});

// POST /api/auth/login
router.post('/login', async (req, res, next) => {
  try {
    const { email, password } = req.body;
    if (!email || !password) {
      return res.status(400).json({ message: 'Missing credentials' });
    }
    const user = await User.findOne({ email });
    if (!user || !user.verifyPassword(password)) {
      return res.status(401).json({ message: 'Invalid email or password' });
    }
    const token = signToken({ id: user._id });
    res.json({
      token,
      user: {
        id:       user._id,
        username: user.username,
        email,
        role:     user.role,
      },
    });
  } catch (err) {
    next(err);
  }
});

export default router;
