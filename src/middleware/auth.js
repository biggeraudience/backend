// backend/src/middleware/auth.js
import { verifyToken } from '../services/jwt.js';
import User from '../models/User.js';

export async function authenticateJWT(req, res, next) {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({ message: 'No token provided' });
  }

  const token = authHeader.split(' ')[1];
  const decoded = verifyToken(token);

  if (!decoded) {
    return res.status(403).json({ message: 'Invalid or expired token' });
  }

  const user = await User.findById(decoded.id);
  if (!user || user.status !== 'active') {
    return res.status(401).json({ message: 'User not found or inactive' });
  }

  req.user = user;
  next();
}
