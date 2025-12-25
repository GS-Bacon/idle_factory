"""
Blender MCPサーバー自動起動スクリプト

使用方法:
  DISPLAY=:10 blender --python tools/start_blender_mcp.py

このスクリプトは:
1. Blenderを起動
2. MCPアドオンを有効化
3. MCPサーバーを自動起動（ポート9876）
4. _base.pyをロード
"""

import bpy
import sys
import os
import time

print("\n" + "=" * 60)
print("Blender MCP Auto-Start Script")
print("=" * 60)

# 1. アドオンの有効化を試みる
addon_enabled = False

# 方法1: bpy.opsで有効化
try:
    bpy.ops.preferences.addon_enable(module="blender_mcp")
    addon_enabled = True
    print("[OK] Blender MCP addon enabled via preferences")
except Exception as e:
    print(f"[INFO] addon_enable failed: {e}")

# 方法2: 直接ファイルをロード
if not addon_enabled:
    addon_paths = [
        os.path.expanduser("~/.config/blender/4.0/scripts/addons/blender_mcp.py"),
        os.path.expanduser("~/.config/blender/4.2/scripts/addons/blender_mcp.py"),
        os.path.expanduser("~/.config/blender/4.3/scripts/addons/blender_mcp.py"),
    ]

    for addon_path in addon_paths:
        if os.path.exists(addon_path):
            try:
                import importlib.util
                spec = importlib.util.spec_from_file_location("blender_mcp", addon_path)
                blender_mcp = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(blender_mcp)
                addon_enabled = True
                print(f"[OK] Addon loaded from: {addon_path}")
                break
            except Exception as e:
                print(f"[WARN] Failed to load {addon_path}: {e}")

if not addon_enabled:
    print("[ERROR] Could not enable Blender MCP addon")
    print("Please install from: https://github.com/ahujasid/blender-mcp")
    print("Or: pip install blender-mcp")

# 2. MCPサーバーの起動を試みる
server_started = False

# 方法1: オペレータで起動
try:
    bpy.ops.mcp.start_server()
    server_started = True
    print("[OK] MCP Server started via operator")
except Exception as e:
    print(f"[INFO] Operator start failed: {e}")

# 方法2: タイマーで遅延起動
if not server_started:
    def delayed_server_start():
        try:
            bpy.ops.mcp.start_server()
            print("[OK] MCP Server started (delayed)")
        except Exception as e:
            print(f"[WARN] Delayed start failed: {e}")
            print("[INFO] Please start manually: Sidebar > MCP > Start Server")
        return None  # タイマー停止

    bpy.app.timers.register(delayed_server_start, first_interval=2.0)
    print("[INFO] Scheduled delayed MCP server start...")

# 3. _base.py をロード
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

# 4. 状態サマリ
print("\n" + "=" * 60)
print("Blender MCP Status")
print("=" * 60)
print(f"  Blender Version: {bpy.app.version_string}")
print(f"  Addon Enabled: {addon_enabled}")
print(f"  MCP Port: 9876")
print("=" * 60)

if not bpy.app.background:
    print("\n[INFO] Running in interactive mode")
    print("  - MCP panel available in 3D View sidebar (N key)")
    print("  - If server not started, click 'Start Server' in MCP tab")
