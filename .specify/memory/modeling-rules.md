# modeling-rules (AI compressed)

ref:modeling-compact.md(detailed)|research:lowpoly-style-research.md

## style

base:mc/unturned-block+astroneer-textureless|shade:flat+vertex-edge-dark
palette:material-color-only(no-uv)|tri-budget:tool50-200,machine200-1500

## tool-ratio

handle:60-70%total|head-h:30-40%|head-w:4-6×handle-dia|grip:handle+15%
total-h:0.18-0.25|validate:print-ratios,check-connect

## primitives

use:octagon,chamfered-cube,hexagon,trapezoid|禁止:vertex-move
hierarchy:root-empty→parts-relative|origin:bottom-center(0,0,0)

## _base.py

primitive:create_octagon,octagonal_prism,chamfered_cube,hexagon,trapezoid
part:gear,shaft,pipe,bolt,piston,roller,conveyor_belt,support_leg
high-level:tool_handle,ingot,ore_chunk,plate,machine_frame,corner_bolts
material:iron,copper,brass,dark_steel,wood,stone
finish:finalize_model,export_gltf,validate_model

## connection-face

dir:front(+Z),back(-Z),left(-X),right(+X),top(+Y),bottom(-Y)
flange:pipe-dia×1.2-1.4,thick0.02-0.03,bolt4-6
pos:block-boundary(±0.5)|meta:extras-json

## done-models

item:pickaxe,hammer,axe,wrench,shovel,drill,pal_sphere
machine:furnace,conveyor,crusher,press,pump,tank,miner,assembler,mixer,centrifuge,generator,chemical_reactor,solar_panel,coal_gen,fuel_gen,nuclear,color_router,signal_tx/rx,creature_feeder/breed/transport,train_engine,cargo_wagon
structure:train_station,creature_pen

## todo

priority1:pipe-straight/L/T/cross
priority2:chest,shaft,gearbox
priority3:wire,cable,circuit_board

## workflow

1.subagent(general-purpose)+prompt:_base.py-read,model-name,category,principles
2.blender-mcp:execute→preview→fix
3.validate:ratio,connect,tri-count
4.export:assets/models/{cat}/{name}.gltf
