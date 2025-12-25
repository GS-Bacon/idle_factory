"""
Blender MCP サーバー自動起動スクリプト

使用方法:
  DISPLAY=:10 blender --python tools/blender_autostart_mcp.py

このスクリプトは:
1. Blenderを起動
2. MCPアドオンを有効化
3. タイマーでMCPサーバーを自動起動
4. _base.pyをロード
"""

import bpy
import os
import sys

print("\n" + "=" * 60)
print("Blender MCP Auto-Start Script")
print("=" * 60)

# グローバル変数でサーバーインスタンスを保持
_mcp_server = None

def start_mcp_server_delayed():
    """MCPサーバーを遅延起動（Blender初期化完了後）"""
    global _mcp_server

    try:
        # アドオンから直接サーバークラスをインポート
        addon_path = os.path.expanduser("~/.config/blender/4.0/scripts/addons/blender_mcp.py")

        if os.path.exists(addon_path):
            import importlib.util
            spec = importlib.util.spec_from_file_location("blender_mcp_addon", addon_path)
            addon_module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(addon_module)

            # BlenderMCPServerクラスを使用
            if hasattr(addon_module, 'BlenderMCPServer'):
                _mcp_server = addon_module.BlenderMCPServer()
                _mcp_server.start()
                print("[OK] MCP Server started via BlenderMCPServer class")
                return None  # タイマー停止
            else:
                print("[WARN] BlenderMCPServer class not found in addon")
        else:
            print(f"[ERROR] Addon not found: {addon_path}")

    except Exception as e:
        print(f"[ERROR] Failed to start MCP server: {e}")
        import traceback
        traceback.print_exc()

    return None  # タイマー停止

# 2秒後にサーバー起動をスケジュール
bpy.app.timers.register(start_mcp_server_delayed, first_interval=2.0)
print("[INFO] Scheduled MCP server start in 2 seconds...")

# _base.py をロード
script_dir = os.path.dirname(os.path.abspath(__file__))
base_path = os.path.join(script_dir, "blender_scripts/_base.py")

if os.path.exists(base_path):
    try:
        exec(open(base_path).read())
        print(f"[OK] _base.py loaded")
    except Exception as e:
        print(f"[WARN] Failed to load _base.py: {e}")
else:
    print(f"[INFO] _base.py not found at {base_path}")

# 状態サマリ
print("\n" + "=" * 60)
print("Blender MCP Status")
print("=" * 60)
print(f"  Blender Version: {bpy.app.version_string}")
print(f"  MCP Port: 9876 (starting in 2 seconds)")
print("=" * 60)

if not bpy.app.background:
    print("\n[INFO] Running in interactive mode")
