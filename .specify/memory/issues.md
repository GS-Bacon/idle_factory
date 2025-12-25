# issues (AI compressed)

## open

### worldgen
priority:mid|done:perlin,async,dist4|todo-h:spawn-opt,grass|todo-m:terrain-var,biome,cave,ore|todo-l:struct,tree

### blender-mcp
priority:mid|issue:func-lost(all-in-one),material-err(iterate-bsdf)
startup:`DISPLAY=:10 blender --python tools/blender_autostart_mcp.py &`
screenshot:use`render_preview()`or`f3d`(viewport-black-fixed)

### modeling-skill
priority:low|status:mostly-ok(25+models-done)
issues:
- func-lost:mcp-each-exec-independent→all-in-one-or-background-blender
- import-inconsistent:hammer.py(exec)vs-pickaxe.py(redefine)→unify-to-exec
- blender4-api:use_auto_smooth-removed→modifier-or-version-branch
- screenshot:add-render_preview-to-skill-def
- hyper3d:disabled(enable-via-panel-if-needed)
- validation:connection-face,material-color-check-missing
action:1.func-lost-warning(small),2.import-unify(mid),3.screenshot-doc(small)

### spec-impl-gap
priority:high|gap9:achievement,stats,dev-mode,profile-dir,editor-steam-tab,mod-semver,profile-switch,active_profile-load,profile-select-hardcode
priority-order:1.dir-fix,2.achievement-stats,3.profile-load,4.steam-tab,5.hotreload

### spec-ambiguity
priority:mid|gamemode-conflict,lua-sandbox-detail,profile-responsibility,mp-spec-missing

### mid-systems
priority:mid|pending:headlift,gen-tier,creature,color-train,global-signal
models:pump,coal/fuel-gen,nuclear,train,wagon,station,router,signal-tx/rx,creature*6
ref:feature-roadmap-from-research.md

### e2e-test
priority:mid|status:partial(setup-only)
issues:
- editor-assets-path:test-skip-if-not-configured→mock-assets-or-ci-fixture
- game-input-sim:TapKey-release-timing→next-frame-release-unreliable
- human-like-test:Think/WaitRandom/MouseMoveSmooth→impl-pending
- tauri-webdriver:playwright-only-web-view→native-dialog-untestable
- ci-integration:xvfb-required,display-setup
todo:
- impl:human-like-test-steps(Think,MouseMoveSmooth,ClickElement)
- impl:mock-assets-for-ci
- impl:editor-tab-specific-tests(recipes-node-drag,multiblock-3d-edit)
- add:visual-regression(screenshot-diff)

### perf-issues
priority:mid

high:
- assembler-multi-scan:93-120→hashmap-aggregate
- minimap-o(n²):190-201→on-pos-change-only
- debug-log:assembler→debug!

mid:
- conveyor-sort:145-155→on-change-only
- dup-filter:203-208→1-pass
- systemtime:128,106→global-counter
- itemslot-clone:172→take/ref

low:
- logistics-sort:228-229→binaryheap

## done

build-sigsegv:2025-12-24|cause:mem-pressure|fix:job-limit,opt0
