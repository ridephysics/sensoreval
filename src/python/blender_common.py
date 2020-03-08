import bpy

def get_pose_index(o, name):
    idx = 0

    for pm in o.pose_library.pose_markers:
        if name == pm.name:
            return idx

        idx += 1

    raise Exception('pose \'%s\' not found' %  (name))

def get_context_by_type(typename):
    for window in bpy.context.window_manager.windows:
        for area in window.screen.areas:
            if area.type == typename:
                for region in area.regions:
                    if region.type == 'WINDOW':
                        context = {
                            'window': window,
                            'screen': window.screen,
                            'area': area,
                            'region': region,
                            'scene': bpy.context.scene,
                            'edit_object': bpy.context.edit_object,
                            'active_object': bpy.context.active_object,
                            'selected_objects': bpy.context.selected_objects
                        }
                        return context
                    
    raise Exception("not found")

def apply_pose(o, name):
    bones = [pose_bone for pose_bone in o.pose.bones if pose_bone.bone.select]

    context = get_context_by_type('PROPERTIES')
    context['active_object'] = o
    context['selected_pose_bones'] = bones
    
    bpy.ops.poselib.apply_pose(context, pose_index=get_pose_index(o, name))

def set_bpy_quat(o, q):
    o[0] = q[0]
    o[1] = q[1]
    o[2] = q[2]
    o[3] = q[3]

def render(path):
    bpy.context.scene.render.filepath = path
    bpy.ops.render.render(write_still=True)

exec(load_data())