# constitution (AI compressed)

## core

type:3d-voxel-factory|mode:survival(future-hp/enemy),creative(default-pure-build)
stack:rust-stable,bevy0.15,lua5.4(mlua),yaml(serde)|arch:ecs-first,data-driven,deterministic
chunk:32³|mesh:greedy|lod:4lvl|sim:20tps-fixed

## code

naming:mod=snake,struct=Pascal,fn=snake,const=SCREAMING|doc:all-public,module-level,complex-inline
error:no-panic-prod,Result<T,E>,log-info/warn/error,graceful-degrade

## test

coverage:core90%+,other70%+|types:unit,integration,snapshot(insta),fuzz,property(proptest)
tdd:fail-first,regress-before-fix

## arch

```
src/core(config,registry,input)
   /rendering(voxel,mesh,model)
   /gameplay(machine,player,power)
   /ui
   /network(future)
```
rule:no-circular,core-independent,ui-use-events

## perf

target:60fps,drawcall<500,chunk-async,item-instancing,shader-precompile,frustum-cull
mem:no-clone-excess,with_capacity,pool-reuse

## ux

flow:MainMenu→ProfileSelect→ProfileSettings→SaveSelect→WorldGen→InGame↔PauseMenu
control:kb-first+mouse,full-remap,hold/toggle|ui:mc-slot,minimal-hud,tooltip,<0.1s-response
a11y:colorblind3,contrast4.5:1+,color+shape,subtitle,visual-sound,key-remap,hold/toggle,hint,pause
sound:master>music/sfx/voice,3+var,pitch±10%,spatial,distance-atten
i18n:fluent,30%-margin,rtl-ready,plural

## security

mem:rust-guarantee,no-unsafe(except-justified),cargo-audit
data:yaml-graceful-fail,save-aes256gcm,version-migrate,input-sanitize
mp-ac:server-auth,rate-limit,audit-log

## constraints

voxel:chunk32³,greedy-mesh,lod,frustum-cull
power:kinetic(speed+stress),network-bfs,overstress-graceful,deterministic
mp(future):server-auth,client-predict,input-validate,reconnect,delta-compress,20tps

## workflow

branch:feature→master|commit:ja,descriptive|ci:test-all,clippy0,fmt

## phase-status

1-core,2-logistics,3-power-multiblock,4-script-signal,5-optimize-mod,menu-save:done

## vision

goal:space-station-complete,blueprint,creative/survival,mod-ecosystem
philosophy:data-driven,player-empower,emergent-complexity,no-artificial-limit
