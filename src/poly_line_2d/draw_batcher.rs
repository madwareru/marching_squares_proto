use macroquad::prelude::*;

pub struct BufferedDrawBatcher {
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u16>,
}

impl BufferedDrawBatcher {
    pub fn new() -> Self {
        Self {
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
        }
    }

    pub fn clear_buffers(&mut self) {
        self.vertex_buffer.clear();
        self.index_buffer.clear();
    }

    pub fn extend(&mut self,
                  vertices: impl Iterator<Item=Vertex>,
                  indices: impl Iterator<Item=u16>
    ) {
        self.vertex_buffer.extend(vertices);
        self.index_buffer.extend(indices);
    }

    pub fn too_many_vertices_in_buffer(&self) -> bool {
        self.vertex_buffer.len() >= 500
    }

    pub fn renderize(&mut self, texture: Option<Texture2D>) {
        if self.vertex_buffer.len() == 0 {
            self.clear_buffers();
            return;
        }
        let quad_gl = unsafe {
            let InternalGlContext { quad_gl, .. } = get_internal_gl();
            quad_gl
        };

        quad_gl.texture(texture);
        quad_gl.draw_mode(DrawMode::Triangles);
        quad_gl.geometry(&self.vertex_buffer, &self.index_buffer);

        self.clear_buffers();
    }

    pub fn flush(&mut self) {
        unsafe {
            let mut gl = get_internal_gl();
            gl.flush();
        }
    }
}