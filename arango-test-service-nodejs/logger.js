/**
 * Uvicorn-style logging (matches Python FastAPI / uvicorn console output).
 *
 * Example access line:
 *   INFO:     172.17.0.1:34270 - "GET /docs HTTP/1.1" 200 OK
 */

const LEVEL_SUFFIX = '     '; // five spaces after "INFO:" / "ERROR:"

function write(level, stream, message) {
  stream.write(`${level}:${LEVEL_SUFFIX}${message}\n`);
}

function logInfo(message) {
  write('INFO', process.stdout, message);
}

function logError(message) {
  const text =
    message instanceof Error ? message.message : String(message);
  write('ERROR', process.stderr, text);
}

function getClientAddress(req) {
  return req.ip || req.socket?.remoteAddress || '-';
}

function logAccess(req, res) {
  const client = getClientAddress(req);
  const path = req.originalUrl || req.url;
  const statusText = res.statusMessage || '';
  const statusSuffix = statusText ? ` ${statusText}` : '';
  logInfo(
    `${client} - "${req.method} ${path} HTTP/${req.httpVersion}" ${res.statusCode}${statusSuffix}`
  );
}

function accessLogMiddleware() {
  return (req, res, next) => {
    res.on('finish', () => logAccess(req, res));
    next();
  };
}

function logServerStartup(port) {
  const pid = process.pid;
  logInfo(`Started server process [${pid}]`);
  logInfo('Waiting for application startup.');
  logInfo('Application startup complete.');
  logInfo(`Service running on http://0.0.0.0:${port} (Press CTRL+C to quit)`);
}

function logServerShutdown() {
  const pid = process.pid;
  logInfo('Shutting down');
  logInfo('Waiting for application shutdown.');
  logInfo('Application shutdown complete.');
  logInfo(`Finished server process [${pid}]`);
}

function registerShutdownHandlers(server) {
  let shuttingDown = false;

  const shutdown = () => {
    if (shuttingDown) {
      return;
    }
    shuttingDown = true;
    logServerShutdown();
    server.close(() => process.exit(0));
    setTimeout(() => process.exit(0), 5000).unref();
  };

  process.on('SIGINT', shutdown);
  process.on('SIGTERM', shutdown);
}

module.exports = {
  logInfo,
  logError,
  accessLogMiddleware,
  logServerStartup,
  registerShutdownHandlers,
};
