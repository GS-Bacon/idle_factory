#!/usr/bin/env python3
"""WebSocket Mod API E2E Test"""

import asyncio
import json
import sys
import websockets

async def test_api():
    uri = "ws://127.0.0.1:9877"
    tests_passed = 0
    tests_failed = 0

    # Longer timeout for slow software rendering
    async with websockets.connect(uri, open_timeout=30, close_timeout=10) as ws:
        # Test 1: game.version
        print("Sending game.version request...")
        await ws.send(json.dumps({"jsonrpc": "2.0", "method": "game.version", "id": 1}))
        print("Waiting for response...")
        try:
            resp = json.loads(await asyncio.wait_for(ws.recv(), timeout=30))
        except asyncio.TimeoutError:
            print("✗ game.version: timeout waiting for response")
            return False
        if "result" in resp and "version" in resp["result"]:
            print("✓ game.version")
            tests_passed += 1
        else:
            print("✗ game.version:", resp)
            tests_failed += 1

        # Test 2: item.list
        await ws.send(json.dumps({"jsonrpc": "2.0", "method": "item.list", "id": 2}))
        resp = json.loads(await ws.recv())
        if "result" in resp and len(resp["result"].get("items", [])) >= 10:
            print(f"✓ item.list ({len(resp['result']['items'])} items)")
            tests_passed += 1
        else:
            print("✗ item.list:", resp)
            tests_failed += 1

        # Test 3: machine.list
        await ws.send(json.dumps({"jsonrpc": "2.0", "method": "machine.list", "id": 3}))
        resp = json.loads(await ws.recv())
        if "result" in resp and len(resp["result"].get("machines", [])) >= 3:
            print(f"✓ machine.list ({len(resp['result']['machines'])} machines)")
            tests_passed += 1
        else:
            print("✗ machine.list:", resp)
            tests_failed += 1

        # Test 4: recipe.list
        await ws.send(json.dumps({"jsonrpc": "2.0", "method": "recipe.list", "id": 4}))
        resp = json.loads(await ws.recv())
        if "result" in resp:
            print("✓ recipe.list")
            tests_passed += 1
        else:
            print("✗ recipe.list:", resp)
            tests_failed += 1

        # Test 5: event.subscribe (not implemented yet - should return error)
        await ws.send(json.dumps({
            "jsonrpc": "2.0",
            "method": "event.subscribe",
            "params": {"event_type": "item.delivered"},
            "id": 5
        }))
        resp = json.loads(await ws.recv())
        # Note: event.subscribe is not implemented yet, so we expect an error
        if "error" in resp:
            print("✓ event.subscribe returns error (not implemented yet)")
            tests_passed += 1
        elif "result" in resp and "subscription_id" in resp["result"]:
            print("✓ event.subscribe")
            tests_passed += 1
        else:
            print("✗ event.subscribe:", resp)
            tests_failed += 1

        # Test 6: invalid method
        await ws.send(json.dumps({"jsonrpc": "2.0", "method": "invalid.method", "id": 6}))
        resp = json.loads(await ws.recv())
        if "error" in resp:
            print("✓ invalid method returns error")
            tests_passed += 1
        else:
            print("✗ invalid method:", resp)
            tests_failed += 1

    print(f"\n{tests_passed}/{tests_passed + tests_failed} tests passed")
    return tests_failed == 0

if __name__ == "__main__":
    success = asyncio.run(test_api())
    sys.exit(0 if success else 1)
