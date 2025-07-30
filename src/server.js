import http from 'http';
import app from './app.js';
import config from './config/index.js';

const server = http.createServer(app);

server.listen(config.port, () => {
  console.log(`🚀 Server listening on http://localhost:${config.port}`);
});

server.on('error', (err) => {
  console.error('Server error:', err);
  process.exit(1);
});