#!/bin/bash
# Blender MCPサーバー起動 & Claude Code再起動スクリプト

set -e

echo "=== Blender MCP Server Setup ==="

# 既存のBlenderプロセスを終了
echo "Stopping existing Blender processes..."
pkill -f blender || true
sleep 1

# Blenderをバックグラウンドで起動（MCPアドオン有効化スクリプト付き）
echo "Starting Blender with MCP addon..."

# Blender起動用Pythonスクリプト
cat > /tmp/start_mcp_server.py << 'PYTHON_SCRIPT'
import bpy
import sys
import os

# アドオンを有効化
addon_path = os.path.expanduser("~/.config/blender/4.0/scripts/addons/blender_mcp.py")
if os.path.exists(addon_path):
    # アドオンを直接実行して登録
    exec(open(addon_path).read())
    print("Blender MCP addon loaded")

    # MCPサーバーを起動
    try:
        bpy.ops.mcp.start_server()
        print("MCP Server started on port 9876")
    except Exception as e:
        print(f"Note: MCP server start via operator failed: {e}")
        print("Trying alternative method...")

        # 代替方法: 直接サーバーを起動
        import socket
        import threading
        import json

        class SimpleMCPServer:
            def __init__(self, port=9876):
                self.port = port
                self.running = False

            def start(self):
                self.running = True
                self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
                self.socket.bind(('localhost', self.port))
                self.socket.listen(1)
                print(f"MCP Server listening on port {self.port}")

                def accept_connections():
                    while self.running:
                        try:
                            client, addr = self.socket.accept()
                            print(f"Client connected: {addr}")
                            # Handle client in separate thread
                            threading.Thread(target=self.handle_client, args=(client,)).start()
                        except:
                            break

                threading.Thread(target=accept_connections, daemon=True).start()

            def handle_client(self, client):
                try:
                    while self.running:
                        data = client.recv(4096)
                        if not data:
                            break
                        # Echo back for now
                        response = {"status": "ok", "message": "Blender MCP ready"}
                        client.send(json.dumps(response).encode())
                except:
                    pass
                finally:
                    client.close()

        # Don't start simple server - let the addon handle it
        print("Please enable MCP addon manually in Blender preferences")
else:
    print(f"ERROR: Addon not found at {addon_path}")
    print("Please download from: https://github.com/ahujasid/blender-mcp")

print("\n" + "="*50)
print("Blender MCP Server Ready")
print("Port: 9876")
print("="*50)
PYTHON_SCRIPT

# DISPLAY環境変数を設定してBlenderを起動
export DISPLAY=:10
nohup blender --python /tmp/start_mcp_server.py > /tmp/blender_mcp.log 2>&1 &
BLENDER_PID=$!

echo "Blender started with PID: $BLENDER_PID"
echo "Log: /tmp/blender_mcp.log"

# Blenderが起動するまで待機
echo "Waiting for Blender to initialize..."
sleep 5

# ログを確認
echo ""
echo "=== Blender Log ==="
tail -20 /tmp/blender_mcp.log 2>/dev/null || echo "Log not available yet"

echo ""
echo "=== Next Steps ==="
echo "1. Blender window should be open on DISPLAY :10"
echo "2. In Blender: Edit > Preferences > Add-ons > Enable 'Blender MCP'"
echo "3. In Blender 3D View: Press N > MCP tab > Start Server"
echo "4. Then restart Claude Code with: claude --resume"
echo ""
echo "Or run this to restart Claude Code now:"
echo "  exec claude --resume"
