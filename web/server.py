#!/usr/bin/env python3
"""
WASM用HTTPサーバー
Cross-Origin-Isolation ヘッダー付き
"""

import http.server
import socketserver
import json
from datetime import datetime

PORT = 8080
ERROR_LOG = '/tmp/wasm-client-errors.log'
GAME_LOG = '/tmp/wasm-game-sessions.jsonl'

class CORSRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        # Cross-Origin-Isolation headers (required for SharedArrayBuffer)
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Resource-Policy', 'same-origin')
        # Cache control for development
        self.send_header('Cache-Control', 'no-cache, no-store, must-revalidate')
        super().end_headers()

    def guess_type(self, path):
        if path.endswith('.wasm'):
            return 'application/wasm'
        return super().guess_type(path)

    def do_POST(self):
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length) if content_length > 0 else b''

        if self.path == '/error-log':
            try:
                data = json.loads(post_data)
                timestamp = datetime.now().isoformat()
                log_entry = f"\n=== {timestamp} ===\nUA: {data.get('ua', 'N/A')}\nError: {data.get('error', 'N/A')}\n"
                with open(ERROR_LOG, 'a') as f:
                    f.write(log_entry)
                print(f"[CLIENT ERROR] {data.get('error', 'N/A')[:100]}...")
            except Exception as e:
                print(f"Error logging: {e}")
            self._respond_ok()

        elif self.path == '/game-log':
            try:
                data = json.loads(post_data)
                data['server_time'] = datetime.now().isoformat()
                with open(GAME_LOG, 'a') as f:
                    f.write(json.dumps(data) + '\n')
                # Print summary
                sid = data.get('sessionId', '?')
                fps_list = data.get('fps', [])
                avg_fps = sum(f['fps'] for f in fps_list) / len(fps_list) if fps_list else 0
                events = len(data.get('events', []))
                errors = len(data.get('errors', []))
                print(f"[GAME LOG] sid={sid} avgFps={avg_fps:.0f} events={events} errors={errors}")
            except Exception as e:
                print(f"Game log error: {e}")
            self._respond_ok()

        else:
            self.send_response(404)
            self.end_headers()

    def _respond_ok(self):
        self.send_response(200)
        self.send_header('Content-Type', 'text/plain')
        self.end_headers()
        self.wfile.write(b'OK')

if __name__ == '__main__':
    print(f"Error log: {ERROR_LOG}")
    print(f"Game log:  {GAME_LOG}")
    with socketserver.TCPServer(("0.0.0.0", PORT), CORSRequestHandler) as httpd:
        print(f"Serving at http://0.0.0.0:{PORT}")
        print(f"Local:     http://10.13.1.1:{PORT}")
        print(f"Tailscale: http://100.84.170.32:{PORT}")
        httpd.serve_forever()
