# changelog (AI compressed)

## 2025-12-25

editor-e2e:playwright-setup,editor.spec.ts(15tests),e2e-skill-update|run:`npm run e2e`(editor),`cargo run --e2e-test`(game)

worldgen:biome6(plain/desert/ocean/forest/mountain/wetland),block14(grass/sand/ore...),ore-dist(coal0-128,iron-16-72,copper-16-112,gold-64-32),miner-auto-detect|test:22new,175total

spec-audit:9impl-gap,4spec-conflict→issues.md|priority:dir-struct,achievement,profile-load

high-priority-systems:
- dual-lane-belt(factorio):ConveyorLane(L/R),side-load,alternate-split
- smart-splitter:filter(any/none/overflow/item),3-out
- power-control:CircuitBreaker(auto-trip),PowerSwitch(manual)
- overclock:1-250%,power×speed^1.6,powershard0-3
- quality:5-level(normal→legendary),module-t1-3(2.5-10%)
- alt-recipe:harddrive-research,3-choice
- logistics-robot:roboport,4-chest-type
- blueprint:mk1-32m³,mk2-64m³+hologram
- awesome-sink:item→point→coupon

infinite-world:perlin-noise,async-gen,render-dist4,unload512|test:5unit,11e2e

modeling-skill:compact-ref,templates/,json→script,_base.py-highlevel-parts

survival-physics:gravity32m/s²,jump9m/s(1.25block),collision-axis-separate,sneak1.31m/s,swim,ladder,coyote0.1s|block:water,lava,ladder

## 2025-12-24

blender-scripts:13cat,100+items,lowpoly-industrial|_base.py:primitive+material+export
pattern-apply:P1-tier,P3-multipath,R1-byproduct,R2-int-ratio,R3-depth5
items:ore8,wood3,stone5,ingot7,plate5,dust5,wire3,rod3,gear3,pipe3,processed5,component10,liquid6,gas5,food8,tool12,armor12,machine13
recipes:smelt11,crush7,press5,wire3,rod3,gear3,pipe3,wood2,alloy2,electronics3,mechpart7,food3,tool12,armor8,machine12

build-issue:rustc-sigsegv,llvm-crash,mem-pressure→workaround:job-limit,opt0

## 2025-12-23

e2e-test:F9-screenshot,F10-ui,F11-full,F12-dump,--e2e-test|token-opt:report.txt(1-2kb)<dump(5-10kb)<png
interaction-test:10phase(menu,move,hotbar,mouse,inv,pause,container,quick,combo,exit)
worldgen-ui:gamemode-select(survival/creative)
inv-ui:css-grid-auto-align,slot54px,gap4px
ui-rules:`.specify/memory/ui-design-rules.md`
world-save:pos,inv,gamemode,playtime
pause-menu:esc-toggle,resume/save-quit/mainmenu
profile-system:`profiles/*/profile.yaml`,resource-pack-per-profile
resource-pack:mc-style-override,priority-stack,hotreload

## 2025-12-22

impl:encryption(aes256gcm),accessibility(colorblind/scale/subtitle/input),sound(hierarchy/variation/spatial),ui-feedback
spec-update:52patterns→constitution,core-mechanics,editor|compress:patterns-compact,index-compact
research:10reports(mp,security,ui,mod,level,bevy,rust,a11y,sound,graphics)
arch:factory-data-types-crate,yaml-unified,ts-rs-typegen

## phase-done

1-core,2-logistics,3-power-multiblock,4-script-signal,5-optimize-mod,menu-save:all-complete
test:175+pass,clippy:0warn
