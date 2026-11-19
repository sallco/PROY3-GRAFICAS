"""Ejecutar con: blender --background --python scripts/optimizar_nave.py"""
from pathlib import Path
import bpy

root = Path(__file__).resolve().parents[1]
bpy.ops.object.select_all(action="SELECT")
bpy.ops.object.delete(use_global=False)
bpy.ops.wm.obj_import(filepath=str(root / "assets/models/nave-original.obj"))
objects = [obj for obj in bpy.context.selected_objects if obj.type == "MESH"]
for obj in objects:
    bpy.context.view_layer.objects.active = obj
    modifier = obj.modifiers.new("Vuelo", "DECIMATE")
    modifier.ratio = 0.028
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    triangulate = obj.modifiers.new("Triangulos", "TRIANGULATE")
    bpy.ops.object.modifier_apply(modifier=triangulate.name)
# Exportación explícita: no se requieren MTL, imágenes ni funciones 3D de Raylib.
vertices, faces = [], []
for obj in objects:
    offset = len(vertices)
    vertices.extend(obj.matrix_world @ vertex.co for vertex in obj.data.vertices)
    faces.extend(tuple(offset + i + 1 for i in face.vertices) for face in obj.data.polygons)
used = sorted({index for face in faces for index in face})
remap = {old: new + 1 for new, old in enumerate(used)}
vertices = [vertices[index - 1] for index in used]
faces = [tuple(remap[index] for index in face) for face in faces]
with (root / "assets/models/nave-vuelo.obj").open("w") as output:
    output.write("# Nadir | nave de vuelo optimizada a partir del modelo original de sallco\n")
    for vertex in vertices:
        output.write(f"v {vertex.x:.7f} {vertex.y:.7f} {vertex.z:.7f}\n")
    for face in faces:
        output.write("f " + " ".join(map(str, face)) + "\n")
print(f"Nave optimizada: {len(vertices)} vertices; {len(faces)} triangulos")
