use crate::renderer::RenderInstance;

pub struct MeshInstancer {
    object_handle: u32,
    instances: Vec<RenderInstance>,
}

impl MeshInstancer {
    pub fn new(object_handle: u32) -> Self {
        Self {
            object_handle,
            instances: vec![],
        }
    }

    pub fn add_instance(&mut self, instance: RenderInstance) {
        self.instances.push(instance);
    }
}
