// backend/src/middleware/errorHandler.js
export function errorHandler(err, req, res, next) {
  console.error('🔥 Error:', err.message || err);
  
  if (res.headersSent) {
    return next(err);
  }

  res.status(err.status || 500).json({
    error: err.message || 'Internal Server Error',
  });
}
