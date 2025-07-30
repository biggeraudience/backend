import mongoose from 'mongoose';
import bcrypt from 'bcryptjs';

const userSchema = new mongoose.Schema({
  username:     { type: String, required: true, trim: true },
  email:        { type: String, required: true, unique: true, lowercase: true, trim: true },
  passwordHash: { type: String, required: true },
  role:         { type: String, enum: ['user','admin'], default: 'user' },
  status:       { type: String, enum: ['active','inactive'], default: 'active' },
}, { timestamps: true });

userSchema.virtual('password').set(function(pw) {
  this.passwordHash = bcrypt.hashSync(pw, 12);
});
userSchema.methods.verifyPassword = function(pw) {
  return bcrypt.compareSync(pw, this.passwordHash);
};

export default mongoose.model('User', userSchema);