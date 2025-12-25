# lowpoly-style (AI compressed)

## games

unturned:256px-tex,closest-interp,1unit=1m,decimate-modifier
astroneer:textureless,material-color-only,triangle-facet(nature)/smooth(artificial),2-person-art-team
a-short-hike:low-res-render+lowpoly,flat-shade,soft-outline,ds-3ds-retro
tabs:wobbly-physics,ragdoll,bright-color,basic-shapes
superhot:3-color-only(white/black/red),extreme-minimal,high-contrast
valheim:low-res-tex(50-100px)→upscale(hard-edge),n64-ps1-retro
crossy-road:voxel-art,magicavoxel/qubicle,6x6px-tex

## principles

shape:build-from-primitives(cube,sphere,cylinder)
polygon:quad-preferred(subdiv-compat)
texture:textureless|vertex-paint|normal-map|tiny-tex(6-256px)
shade:flat|smooth-30deg|toon
palette:consistent+flat→all-important-objects-highlighted
lighting:flat/directional,no-complex-shadow
lod:5k→2k→500tris-by-distance

## tri-budget

char-simple:500-1k|char-detail:2.5k-3.5k|prop:50-200|env:100-500|structure:1k-5k

## tools

blender:decimate,shade-smooth+auto30,vertex-paint
magicavoxel/qubicle:voxel
blockbench:minecraft-style

## this-project

style:mc/unturned-block+astroneer-textureless
shade:flat+vertex-edge-dark
tri:tool50-200,machine200-1500
consider:astroneer-textureless(cost-reduce),ashorthike-pixelation(optional-filter)
