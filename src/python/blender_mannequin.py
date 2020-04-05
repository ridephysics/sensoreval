import os
import bpy


def robot_set_quat(q):
    set_bpy_quat(robot.rotation_quaternion, q)
    set_bpy_quat(robot_skeleton.rotation_quaternion, q)


# get data from rust
outdir = load_data()
prefix = load_data()
orientations = load_data()
videores = load_data()

# get scene data
robot = bpy.data.objects['robot']
robot_skeleton = bpy.data.objects['robot_skeleton']
scene = bpy.data.scenes['Scene']
camera_name = 'Camera'
camera = bpy.data.cameras[camera_name]
camera_obj = bpy.data.objects[camera_name]

# configure
apply_pose(robot_skeleton, 'sitting')
scene.cycles.samples = 200
scene.render.resolution_y = int(videores[1] / 2)
scene.render.resolution_x = scene.render.resolution_y
scene.render.tile_x = 16
scene.render.tile_y = 16
camera_obj.location[0] = 0.0
camera_obj.location[1] = -8.0
camera_obj.location[2] = 4.31

for (fid, q) in orientations:
    filename = '%s_%s.png' % (prefix, fid)
    path = os.path.join(outdir, filename)

    if not os.path.exists(path):
        robot_set_quat(q)
        render(path)
