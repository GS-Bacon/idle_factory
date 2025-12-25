# specs (AI compressed)

## game

type:3d-voxel-factory|stack:rust+bevy0.15|data:yaml-hotreload|script:lua5.4-sandbox|mp:lightyear
goal:space-station|no-survival|automation-only
chunk:32^3|greedy-mesh|lod:4lvl|world:inf-xy,y±256

### core-mechanics

player:hp10,no-hunger,die:fall/lava,respawn:anchor,inv:40slot,stack:999
conveyor:speed-tiers,roundrobin-split,zipper-merge,clog-detect
pipe:flow-tiers,no-mix,drain-valve,leak→machine-short
tank:10k-mb,multiblock-scale,hot-tank-required
shaft:1type,reverse→break,gear-ratio
power:torque(shaft)+electric(wire),steam→engine/turbine,heat-system
machine:no-recipe-store,auto-detect,dir-fixed,tier-upgrade,vibration3x3,module5x6
signal:1-16strength,wire-decay,wireless-device,lua-computer,robot4x4→5x10
quest:main-seq,sub-parallel,no-skip,priority-change
enchant:auto/manual(xp-gacha),tool-speed/durability/range/luck

### ui-hud

hud:minimap,hotbar10,hp|screen:fullmap,quest-list,task-tree
response:<0.1s|undo-required|info-hierarchy

## best-practices

### multiplayer
must:server-auth,input-validate,rate-limit|rec:client-predict,delta-compress,reconnect
lib:lightyear(predict)|renet(simple)

### security
must:server-auth,all-input-validate,save-encrypt(aes256gcm)|rec:anomaly-detect,audit-log,gradual-ban
indie:avoid-overkill(no-kernel-ac)

### ui
principle:info-hierarchy,all-feedback,undo-required
factory:zoom-density,flow-dir,alert-system|mc-style:slot-ui,minimal-hud

### sound
hierarchy:master>music/sfx/voice>sub|repeat-prevent:3+var,pitch±10%
spatial:distance-atten,limit-n-nearest|a11y:subtitle,visual-indicator

### graphics
target:60fps,drawcall<500|voxel:greedy-mesh,chunk-stream
mass-obj:instancing|shader:precompile,pipeline-cache

### bevy-ecs
component:small,single-resp,marker-use|system:chain-order,run_if
query:changed/added,parallel-aware

### rust
ownership:min-clone,ref/cow/arc|alloc:with_capacity,pool-reuse
error:no-unwrap(except-init),result/?

### mod-api
model:factorio-3stage(settings→data→runtime)|sandbox:disable-os/io/load/require
version:semver,deprecate→remove-path

### accessibility
visual:colorblind-modes,contrast4.5:1+,color+shape
audio:subtitle,visual-sound|motor:remap-all,hold/toggle
cognitive:hint,pause,difficulty

### localization
must:string-extern(fluent),30%-expand-margin|rec:rtl-ready,plural-support

## antipatterns

### critical
|issue|fix|
|-|-|
|client-trust|server-auth|
|plaintext-save|aes-encrypt|
|shader-stutter|precompile|
|same-sound-repeat|variation|
|no-response-op|0.1s-feedback|

### severe
|issue|fix|
|-|-|
|huge-component|split|
|excess-clone|use-ref|
|no-ui-test|snapshot|
|single-bottleneck|multi-path|

### factory-specific
|issue|fix|
|-|-|
|byproduct-clog|consumer/incinerator|
|conveyor-jam|branch/parallel|
|power-unstable|buffer/warning|
|scale-limit|lod/culling|

## editor

arch:tauri-external|profile-target|bevy-child-process
feature:block/recipe-edit,test-play,mod-export
ux:E1-instant-preview,E2-nondestructive(undo100+),E3-constraint-viz,E4-smart-default,E5-bulk-op,E6-ref-integrity

## test

|type|use|tool|
|-|-|-|
|unit|logic|cargo-test|
|snapshot|ui-regress|insta|
|fuzz|boundary|cargo-fuzz|
|property|invariant|proptest|
|sim|long-stable|accel-exec|

coverage:core90%+,other70%+

## gdd-summary

platform:12x12,48port(16-initial)|weather:day-night,rain→waterwheel+/outdoor-machine-|
biome:natural+resource-overlay|enchant:tool/machine|robot:lua,4x4-inv,move/break/place
multiblock:editor-define,wrench-confirm|quest:seq-main,parallel-sub,no-skip

## phase-status

|phase|status|
|-|-|
|1-core|done|
|2-logistics|done|
|3-power-multiblock|done|
|4-script-signal|done|
|5-optimize-mod|done|
|menu-save|done|
