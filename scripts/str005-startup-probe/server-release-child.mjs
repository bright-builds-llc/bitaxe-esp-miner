import { createServer } from 'node:net';
const server = createServer(socket => socket.end());
server.listen(Number(process.argv[2] ?? 0), '127.0.0.1', () => process.stdout.write(`${server.address().port}\n`));
process.stdin.setEncoding('utf8');
process.stdin.on('data', text => { if (text.trim() === 'finish') server.close(() => process.exit(0)); });
process.on('SIGTERM', () => server.close(() => process.exit(0)));
