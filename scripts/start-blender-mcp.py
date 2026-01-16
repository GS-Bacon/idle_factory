#!/usr/bin/env python3
"""
Blender MCP サーバー自動起動スクリプト

使い方:
  blender --python scripts/start-blender-mcp.py

または GUI 付きで:
  blender --python scripts/start-blender-mcp.py -- --gui
"""

import bpy
import sys

def main():
    # アドオンを有効化
    try:
        bpy.ops.preferences.addon_enable(module='blender_mcp_addon')
        print("[MCP] Addon enabled")
    except Exception as e:
        print(f"[MCP] Addon already enabled or error: {e}")

    # ポート設定
    bpy.context.scene.blendermcp_port = 9876

    # サーバー起動
    try:
        bpy.ops.blendermcp.start_server()
        print("[MCP] Server started on port 9876")
    except Exception as e:
        print(f"[MCP] Error starting server: {e}")
        # 手動でサーバー作成を試行
        try:
            from blender_mcp_addon import BlenderMCPServer
            if not hasattr(bpy.types, 'blendermcp_server') or bpy.types.blendermcp_server is None:
                bpy.types.blendermcp_server = BlenderMCPServer(port=9876)
            bpy.types.blendermcp_server.start()
            print("[MCP] Server started manually on port 9876")
        except Exception as e2:
            print(f"[MCP] Manual start failed: {e2}")

if __name__ == "__main__":
    main()

    # GUIモードでない場合はメインループを維持
    if "--background" in sys.argv or "-b" in sys.argv:
        print("[MCP] Running in background mode, keeping alive...")
        import time
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("[MCP] Shutting down...")
